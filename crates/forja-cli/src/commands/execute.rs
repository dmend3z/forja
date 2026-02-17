use std::fs;
use std::process::Command;

use colored::Colorize;

use forja_core::error::{ForjaError, Result};
use forja_core::models::plan::{
    ExecutionCheckpoint, PhaseStatus, PlanMetadata, PlanPhase, PlanStatus, checkpoint_path,
    find_latest_pending, initialize_checkpoint, load_checkpoint, load_plan, save_checkpoint,
    save_plan, workspace_dir,
};
use forja_core::paths::ForjaPaths;
use forja_core::settings;
use forja_core::symlink::auto_install;

/// Execute a previously created plan by launching Claude Code sessions.
/// Plans with phases run phase-by-phase with checkpoints.
/// Plans without phases run in legacy monolithic mode.
pub fn run(plan_id: Option<&str>, profile: &str, resume: bool) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    // 1. Find plan
    let mut plan = match plan_id {
        Some(id) => {
            let plan_path = paths.plans.join(format!("{id}.json"));
            if !plan_path.exists() {
                return Err(ForjaError::PlanNotFound(id.to_string()));
            }
            load_plan(&plan_path)?
        }
        None => find_latest_pending(&paths.plans)?,
    };

    println!("{}", "forja execute".bold());
    println!();
    println!("  Plan:  {}", plan.id.cyan());
    println!("  Task:  {}", plan.task);
    println!("  Team:  {}", plan.team_size);
    println!();

    // 2. Override profile if provided
    if profile != "balanced" {
        plan.profile = profile.to_string();
    }

    // 3. Auto-install missing agents and track analytics
    let skill_ids: Vec<&str> = plan.agents.iter().map(|a| a.skill_id.as_str()).collect();
    auto_install::auto_install_missing(&paths, &skill_ids)?;

    let analytics_path = forja_core::analytics::analytics_path(&paths.forja_root);
    for agent in &plan.agents {
        let _ = forja_core::analytics::track(&analytics_path, &agent.skill_id, "execute");
    }

    // 4. Ensure agent teams env var (always in ~/.claude/settings.json)
    let global_claude = forja_core::paths::ForjaPaths::global_claude_dir()?;
    if !settings::has_teams_env_var(&global_claude) {
        settings::enable_teams_env_var(&global_claude)?;
        println!(
            "  {} Agent teams env var enabled in settings.json",
            "NOTE:".yellow().bold()
        );
        println!();
    }

    // 5. Read plan .md content
    let plan_md_path = paths.plans.join(format!("{}.md", plan.id));
    let plan_md = if plan_md_path.exists() {
        fs::read_to_string(&plan_md_path)?
    } else {
        format!("# Plan: {}\n\nNo detailed plan file found.", plan.task)
    };

    // 6. Check claude CLI exists
    if Command::new("claude").arg("--version").output().is_err() {
        return Err(ForjaError::ClaudeCliNotFound);
    }

    // 7. Decide execution mode: phased (with checkpoints) or legacy (monolithic)
    if plan.phases.is_empty() {
        run_legacy(&paths, &mut plan, &plan_md)
    } else {
        run_phased(&paths, &mut plan, &plan_md, resume)
    }
}

/// Legacy monolithic execution — plans without phases.
fn run_legacy(paths: &ForjaPaths, plan: &mut PlanMetadata, plan_md: &str) -> Result<()> {
    let prompt = build_execution_prompt(plan, plan_md);

    println!("{}", "Launching Claude Code session...".bold());
    println!();

    let status = Command::new("claude")
        .arg("--dangerously-skip-permissions")
        .arg("--")
        .arg(&prompt)
        .status()?;

    if status.success() {
        plan.status = PlanStatus::Executed;
        let plan_json_path = paths.plans.join(format!("{}.json", plan.id));
        save_plan(&plan_json_path, plan)?;
        println!();
        println!(
            "{} Plan {} marked as executed.",
            "Done:".green().bold(),
            plan.id.cyan()
        );
    }

    Ok(())
}

/// Phase-by-phase execution with checkpoint tracking.
fn run_phased(
    paths: &ForjaPaths,
    plan: &mut PlanMetadata,
    plan_md: &str,
    resume: bool,
) -> Result<()> {
    let ckpt_path = checkpoint_path(&paths.plans, &plan.id);
    let ws_dir = workspace_dir(&paths.plans, &plan.id);

    // Load or initialize checkpoint
    let mut checkpoint = if resume && ckpt_path.exists() {
        let ckpt = load_checkpoint(&ckpt_path)?;
        println!("  {} Resuming from checkpoint", "RESUME:".cyan().bold());
        let completed = ckpt
            .phases
            .iter()
            .filter(|p| p.status == PhaseStatus::Completed)
            .count();
        println!("  {}/{} phases completed", completed, ckpt.phases.len());
        println!();
        ckpt
    } else {
        initialize_checkpoint(plan)
    };

    // Ensure workspace directory exists
    fs::create_dir_all(&ws_dir)?;

    // Save initial checkpoint
    save_checkpoint(&ckpt_path, &checkpoint)?;

    println!(
        "{} Executing {} phases...",
        "PHASES:".bold(),
        plan.phases.len()
    );
    println!();

    let mut all_completed = true;

    for i in 0..plan.phases.len() {
        let phase = &plan.phases[i];
        let phase_ckpt = &checkpoint.phases[i];

        // Skip completed phases
        if phase_ckpt.status == PhaseStatus::Completed {
            println!(
                "  {} Phase {}: {} (already completed)",
                "✓".green(),
                i + 1,
                phase.name
            );
            continue;
        }

        // Skip phases whose dependencies failed
        if has_failed_dependency(phase, &plan.phases, &checkpoint) {
            checkpoint.phases[i].status = PhaseStatus::Skipped;
            checkpoint.last_updated = chrono::Utc::now().to_rfc3339();
            save_checkpoint(&ckpt_path, &checkpoint)?;
            println!(
                "  {} Phase {}: {} (dependency failed)",
                "⊘".yellow(),
                i + 1,
                phase.name
            );
            all_completed = false;
            continue;
        }

        // Mark in-progress
        println!("  {} Phase {}: {}", "▶".cyan(), i + 1, phase.name.bold());

        checkpoint.phases[i].status = PhaseStatus::InProgress;
        checkpoint.phases[i].started_at = Some(chrono::Utc::now().to_rfc3339());
        checkpoint.current_phase = Some(i);
        checkpoint.last_updated = chrono::Utc::now().to_rfc3339();
        save_checkpoint(&ckpt_path, &checkpoint)?;

        // Build phase-specific prompt
        let prompt = build_phase_prompt(plan, i, plan_md, &ws_dir);

        // Launch Claude for this phase
        let status = Command::new("claude")
            .arg("--dangerously-skip-permissions")
            .arg("--")
            .arg(&prompt)
            .status()?;

        let exit_code = status.code().unwrap_or(-1);

        if status.success() {
            checkpoint.phases[i].status = PhaseStatus::Completed;
            checkpoint.phases[i].completed_at = Some(chrono::Utc::now().to_rfc3339());
            checkpoint.phases[i].exit_code = Some(exit_code);
            checkpoint.last_updated = chrono::Utc::now().to_rfc3339();
            save_checkpoint(&ckpt_path, &checkpoint)?;

            println!(
                "  {} Phase {}: {} completed",
                "✓".green(),
                i + 1,
                phase.name
            );
        } else {
            checkpoint.phases[i].status = PhaseStatus::Failed;
            checkpoint.phases[i].completed_at = Some(chrono::Utc::now().to_rfc3339());
            checkpoint.phases[i].exit_code = Some(exit_code);
            checkpoint.phases[i].error_message =
                Some(format!("Process exited with code {exit_code}"));
            checkpoint.last_updated = chrono::Utc::now().to_rfc3339();
            save_checkpoint(&ckpt_path, &checkpoint)?;

            println!(
                "  {} Phase {}: {} failed (exit code {})",
                "✗".red(),
                i + 1,
                phase.name,
                exit_code
            );
            println!();

            return Err(ForjaError::PhaseExecutionFailed(format!(
                "Phase '{}' failed with exit code {exit_code}",
                phase.name
            )));
        }
    }

    // All phases done
    if all_completed {
        plan.status = PlanStatus::Executed;
        let plan_json_path = paths.plans.join(format!("{}.json", plan.id));
        save_plan(&plan_json_path, plan)?;
        println!();
        println!(
            "{} All {} phases completed. Plan {} marked as executed.",
            "Done:".green().bold(),
            plan.phases.len(),
            plan.id.cyan()
        );
    }

    Ok(())
}

/// Check if any dependency of a phase has failed.
fn has_failed_dependency(
    phase: &PlanPhase,
    all_phases: &[PlanPhase],
    checkpoint: &ExecutionCheckpoint,
) -> bool {
    for dep_name in &phase.depends_on {
        if let Some(dep_idx) = all_phases.iter().position(|p| &p.name == dep_name) {
            let dep_status = &checkpoint.phases[dep_idx].status;
            if *dep_status == PhaseStatus::Failed || *dep_status == PhaseStatus::Skipped {
                return true;
            }
        }
    }
    false
}

/// Build the prompt for a specific phase, including context from previous phases.
fn build_phase_prompt(
    plan: &PlanMetadata,
    phase_index: usize,
    plan_md: &str,
    workspace_dir: &std::path::Path,
) -> String {
    let phase = &plan.phases[phase_index];
    let mut prompt = String::new();

    prompt.push_str(&format!(
        "Execute Phase {} of {}: {}\n\n",
        phase_index + 1,
        plan.phases.len(),
        phase.name
    ));

    // Plan context
    prompt.push_str("## Task Context\n\n");
    prompt.push_str(&format!("Overall task: {}\n", plan.task));
    if let Some(ref stack) = plan.stack {
        prompt.push_str(&format!("Stack: {}", stack.language));
        if let Some(ref fw) = stack.framework {
            prompt.push_str(&format!(" + {fw}"));
        }
        prompt.push('\n');
    }
    prompt.push('\n');

    // Summary of completed previous phases
    if phase_index > 0 {
        prompt.push_str("## Previous Phases (completed)\n\n");
        for i in 0..phase_index {
            let prev = &plan.phases[i];
            prompt.push_str(&format!(
                "### Phase {}: {} — {}\n",
                i + 1,
                prev.name,
                prev.agent_role
            ));
            prompt.push_str(&format!("Instructions: {}\n", prev.instructions));

            // Include phase output summary if available
            let output_path = workspace_dir.join(format!("phase-{i}.md"));
            if output_path.exists()
                && let Ok(content) = fs::read_to_string(&output_path)
            {
                prompt.push_str(&format!("\nOutput summary:\n{content}\n"));
            }
            prompt.push('\n');
        }
    }

    // Current phase instructions
    prompt.push_str("## Current Phase\n\n");
    prompt.push_str(&format!(
        "**Phase {}: {}**\n\n",
        phase_index + 1,
        phase.name
    ));
    prompt.push_str(&format!("Agent role: {}\n", phase.agent_role));
    if !phase.files_to_create.is_empty() {
        prompt.push_str(&format!(
            "Files to create: {}\n",
            phase.files_to_create.join(", ")
        ));
    }
    if !phase.files_to_modify.is_empty() {
        prompt.push_str(&format!(
            "Files to modify: {}\n",
            phase.files_to_modify.join(", ")
        ));
    }
    prompt.push_str(&format!("\nInstructions: {}\n\n", phase.instructions));

    // Phase output file
    let output_file = workspace_dir.join(format!("phase-{phase_index}.md"));
    prompt.push_str("## Output\n\n");
    prompt.push_str(&format!(
        "After completing this phase, write a brief summary of what was done to:\n{}\n\n",
        output_file.display()
    ));

    // Full plan for reference
    prompt.push_str("## Full Plan Reference\n\n");
    prompt.push_str(plan_md);
    prompt.push('\n');

    // Execution rules
    prompt.push_str("\n## Rules\n\n");
    prompt.push_str("- Read CLAUDE.md before starting\n");
    prompt.push_str("- Focus only on this phase's scope\n");
    prompt.push_str("- Write the output summary file when done\n");
    prompt.push_str("- Stop and report if blocked\n");

    prompt
}

fn build_execution_prompt(plan: &PlanMetadata, plan_md: &str) -> String {
    let mut prompt = String::new();

    prompt.push_str("Execute this implementation plan. You are the team orchestrator.\n\n");

    // Structured phases (if available) or fallback to raw .md
    if plan.phases.is_empty() {
        prompt.push_str("## Plan\n\n");
        prompt.push_str(plan_md);
        prompt.push_str("\n\n");
    } else {
        prompt.push_str("## Implementation Phases\n\n");
        for (i, phase) in plan.phases.iter().enumerate() {
            prompt.push_str(&format!("### Phase {}: {}\n", i + 1, phase.name));
            prompt.push_str(&format!("- **Agent**: {}\n", phase.agent_role));
            if !phase.files_to_create.is_empty() {
                prompt.push_str(&format!(
                    "- **Files to create**: {}\n",
                    phase.files_to_create.join(", ")
                ));
            }
            if !phase.files_to_modify.is_empty() {
                prompt.push_str(&format!(
                    "- **Files to modify**: {}\n",
                    phase.files_to_modify.join(", ")
                ));
            }
            if !phase.depends_on.is_empty() {
                prompt.push_str(&format!(
                    "- **Depends on**: {}\n",
                    phase.depends_on.join(", ")
                ));
            }
            prompt.push_str(&format!("- **Instructions**: {}\n\n", phase.instructions));
        }

        prompt.push_str("## Full Plan Context\n\n");
        prompt.push_str(plan_md);
        prompt.push_str("\n\n");
    }

    // Quality gates
    if !plan.quality_gates.is_empty() {
        prompt.push_str("## Quality Gates\n\n");
        prompt.push_str("After all phases complete, verify these conditions:\n\n");
        for gate in &plan.quality_gates {
            prompt.push_str(&format!("- [ ] {gate}\n"));
        }
        prompt.push('\n');
    }

    // Team configuration
    prompt.push_str("## Team Configuration\n\n");
    prompt.push_str(&format!("Team size: {}\n", plan.team_size));
    prompt.push_str(&format!("Profile: {}\n\n", plan.profile));

    // Agent list with symlink names
    prompt.push_str("Spawn these agents as teammates:\n\n");
    for agent in &plan.agents {
        let symlink_name = format!("forja--{}", agent.skill_id.replace('/', "--"));
        prompt.push_str(&format!(
            "- **{}**: use the `{}` agent\n",
            agent.role, symlink_name
        ));
    }

    // Execution rules
    prompt.push_str("\n## Execution Rules\n\n");
    prompt.push_str("- Follow the implementation phases in order\n");
    prompt.push_str("- Pass results between phases as context\n");
    prompt.push_str("- Stop and report if an agent is blocked or encounters errors\n");
    prompt.push_str("- Use delegate mode for efficiency\n");
    prompt.push_str("- Read CLAUDE.md before starting implementation\n");
    if !plan.quality_gates.is_empty() {
        prompt.push_str("- After all phases complete, run quality gate checks\n");
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;
    use forja_core::models::plan::{PlanAgent, PlanPhase, PlanStack};

    fn test_plan() -> PlanMetadata {
        PlanMetadata {
            id: "20260208-143022-user-auth".to_string(),
            created: "2026-02-08T14:30:22Z".to_string(),
            status: PlanStatus::Pending,
            task: "Add user auth with JWT".to_string(),
            team_size: "full-product".to_string(),
            profile: "balanced".to_string(),
            agents: vec![
                PlanAgent {
                    skill_id: "research/codebase/explorer".to_string(),
                    role: "researcher".to_string(),
                },
                PlanAgent {
                    skill_id: "code/typescript/feature".to_string(),
                    role: "coder".to_string(),
                },
                PlanAgent {
                    skill_id: "test/tdd/workflow".to_string(),
                    role: "tester".to_string(),
                },
            ],
            stack: Some(PlanStack {
                language: "TypeScript".to_string(),
                framework: Some("Next.js".to_string()),
            }),
            quality_gates: vec![],
            phases: vec![],
        }
    }

    #[test]
    fn build_prompt_contains_plan_content() {
        let plan = test_plan();
        let plan_md = "# Plan: Add user auth\n\n## Phase 1\nSet up database schema";

        let prompt = build_execution_prompt(&plan, plan_md);

        assert!(prompt.contains("Execute this implementation plan"));
        assert!(prompt.contains("# Plan: Add user auth"));
        assert!(prompt.contains("Set up database schema"));
    }

    #[test]
    fn build_prompt_contains_team_config() {
        let plan = test_plan();
        let prompt = build_execution_prompt(&plan, "# Plan");

        assert!(prompt.contains("Team size: full-product"));
        assert!(prompt.contains("Profile: balanced"));
    }

    #[test]
    fn build_prompt_maps_agents_to_symlink_names() {
        let plan = test_plan();
        let prompt = build_execution_prompt(&plan, "# Plan");

        assert!(prompt.contains("forja--research--codebase--explorer"));
        assert!(prompt.contains("forja--code--typescript--feature"));
        assert!(prompt.contains("forja--test--tdd--workflow"));
    }

    #[test]
    fn build_prompt_includes_roles() {
        let plan = test_plan();
        let prompt = build_execution_prompt(&plan, "# Plan");

        assert!(prompt.contains("**researcher**"));
        assert!(prompt.contains("**coder**"));
        assert!(prompt.contains("**tester**"));
    }

    #[test]
    fn build_prompt_includes_execution_rules() {
        let plan = test_plan();
        let prompt = build_execution_prompt(&plan, "# Plan");

        assert!(prompt.contains("Follow the implementation phases in order"));
        assert!(prompt.contains("delegate mode"));
        assert!(prompt.contains("CLAUDE.md"));
    }

    #[test]
    fn build_prompt_with_structured_phases() {
        let mut plan = test_plan();
        plan.phases = vec![
            PlanPhase {
                name: "Database schema".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec!["migrations/001_users.sql".to_string()],
                files_to_modify: vec![],
                instructions: "Create users table".to_string(),
                depends_on: vec![],
            },
            PlanPhase {
                name: "Auth middleware".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec!["src/middleware/auth.ts".to_string()],
                files_to_modify: vec!["src/app.ts".to_string()],
                instructions: "Add JWT validation".to_string(),
                depends_on: vec!["Database schema".to_string()],
            },
        ];

        let prompt = build_execution_prompt(&plan, "# Full plan content");

        assert!(prompt.contains("## Implementation Phases"));
        assert!(prompt.contains("### Phase 1: Database schema"));
        assert!(prompt.contains("### Phase 2: Auth middleware"));
        assert!(prompt.contains("migrations/001_users.sql"));
        assert!(prompt.contains("src/middleware/auth.ts"));
        assert!(prompt.contains("src/app.ts"));
        assert!(prompt.contains("Create users table"));
        assert!(prompt.contains("Add JWT validation"));
        // Full plan embedded as context
        assert!(prompt.contains("## Full Plan Context"));
        assert!(prompt.contains("# Full plan content"));
    }

    #[test]
    fn build_prompt_with_quality_gates() {
        let mut plan = test_plan();
        plan.quality_gates = vec![
            "All tests must pass".to_string(),
            "No TypeScript errors".to_string(),
        ];

        let prompt = build_execution_prompt(&plan, "# Plan");

        assert!(prompt.contains("## Quality Gates"));
        assert!(prompt.contains("- [ ] All tests must pass"));
        assert!(prompt.contains("- [ ] No TypeScript errors"));
        assert!(prompt.contains("run quality gate checks"));
    }

    #[test]
    fn build_prompt_fallback_without_phases() {
        let plan = test_plan(); // phases is empty
        let prompt = build_execution_prompt(&plan, "# My plan\n\nSome details here");

        // Should use old behavior — embed .md as "## Plan"
        assert!(prompt.contains("## Plan\n\n# My plan"));
        assert!(!prompt.contains("## Implementation Phases"));
        assert!(!prompt.contains("## Full Plan Context"));
    }

    #[test]
    fn build_prompt_phases_include_dependencies() {
        let mut plan = test_plan();
        plan.phases = vec![PlanPhase {
            name: "API routes".to_string(),
            agent_role: "coder".to_string(),
            files_to_create: vec![],
            files_to_modify: vec!["src/routes.ts".to_string()],
            instructions: "Add auth routes".to_string(),
            depends_on: vec!["Database schema".to_string(), "Auth middleware".to_string()],
        }];

        let prompt = build_execution_prompt(&plan, "# Plan");

        assert!(prompt.contains("**Depends on**: Database schema, Auth middleware"));
    }

    // --- Phase prompt tests ---

    #[test]
    fn build_phase_prompt_includes_context() {
        let mut plan = test_plan();
        plan.phases = vec![
            PlanPhase {
                name: "Schema".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec!["db/schema.sql".to_string()],
                files_to_modify: vec![],
                instructions: "Create schema".to_string(),
                depends_on: vec![],
            },
            PlanPhase {
                name: "API".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec![],
                files_to_modify: vec!["src/api.ts".to_string()],
                instructions: "Build endpoints".to_string(),
                depends_on: vec!["Schema".to_string()],
            },
        ];

        let ws = tempfile::TempDir::new().unwrap();
        let prompt = build_phase_prompt(&plan, 1, "# Full plan", ws.path());

        assert!(prompt.contains("Execute Phase 2 of 2: API"));
        assert!(prompt.contains("Overall task: Add user auth with JWT"));
        assert!(prompt.contains("Stack: TypeScript + Next.js"));
        assert!(prompt.contains("## Previous Phases (completed)"));
        assert!(prompt.contains("Phase 1: Schema"));
        assert!(prompt.contains("Build endpoints"));
        assert!(prompt.contains("## Full Plan Reference"));
    }

    #[test]
    fn build_phase_prompt_first_phase_no_previous() {
        let mut plan = test_plan();
        plan.phases = vec![PlanPhase {
            name: "Setup".to_string(),
            agent_role: "coder".to_string(),
            files_to_create: vec![],
            files_to_modify: vec![],
            instructions: "Initial setup".to_string(),
            depends_on: vec![],
        }];

        let ws = tempfile::TempDir::new().unwrap();
        let prompt = build_phase_prompt(&plan, 0, "# Plan", ws.path());

        assert!(prompt.contains("Execute Phase 1 of 1: Setup"));
        assert!(!prompt.contains("## Previous Phases"));
    }

    #[test]
    fn build_phase_prompt_includes_previous_output() {
        let mut plan = test_plan();
        plan.phases = vec![
            PlanPhase {
                name: "Phase A".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec![],
                files_to_modify: vec![],
                instructions: "Do A".to_string(),
                depends_on: vec![],
            },
            PlanPhase {
                name: "Phase B".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec![],
                files_to_modify: vec![],
                instructions: "Do B".to_string(),
                depends_on: vec![],
            },
        ];

        let ws = tempfile::TempDir::new().unwrap();
        // Write a fake output for phase 0
        fs::write(
            ws.path().join("phase-0.md"),
            "Created users table successfully",
        )
        .unwrap();

        let prompt = build_phase_prompt(&plan, 1, "# Plan", ws.path());

        assert!(prompt.contains("Created users table successfully"));
    }

    // --- Dependency failure tests ---

    #[test]
    fn has_failed_dependency_detects_failure() {
        use forja_core::models::plan::initialize_checkpoint;

        let mut plan = test_plan();
        plan.phases = vec![
            PlanPhase {
                name: "Schema".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec![],
                files_to_modify: vec![],
                instructions: "Create schema".to_string(),
                depends_on: vec![],
            },
            PlanPhase {
                name: "API".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec![],
                files_to_modify: vec![],
                instructions: "Build API".to_string(),
                depends_on: vec!["Schema".to_string()],
            },
        ];

        let mut checkpoint = initialize_checkpoint(&plan);
        checkpoint.phases[0].status = PhaseStatus::Failed;

        assert!(has_failed_dependency(
            &plan.phases[1],
            &plan.phases,
            &checkpoint
        ));
    }

    #[test]
    fn has_failed_dependency_no_failure() {
        use forja_core::models::plan::initialize_checkpoint;

        let mut plan = test_plan();
        plan.phases = vec![
            PlanPhase {
                name: "Schema".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec![],
                files_to_modify: vec![],
                instructions: "Create schema".to_string(),
                depends_on: vec![],
            },
            PlanPhase {
                name: "API".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec![],
                files_to_modify: vec![],
                instructions: "Build API".to_string(),
                depends_on: vec!["Schema".to_string()],
            },
        ];

        let mut checkpoint = initialize_checkpoint(&plan);
        checkpoint.phases[0].status = PhaseStatus::Completed;

        assert!(!has_failed_dependency(
            &plan.phases[1],
            &plan.phases,
            &checkpoint
        ));
    }
}
