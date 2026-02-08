use std::fs;
use std::process::Command;

use colored::Colorize;

use crate::error::{ForjaError, Result};
use crate::models::plan::{PlanMetadata, PlanStatus, find_latest_pending, load_plan, save_plan};
use crate::paths::ForjaPaths;
use crate::settings;
use crate::symlink::auto_install;

/// Execute a previously created plan by launching a Claude Code team session.
pub fn run(plan_id: Option<&str>, profile: &str) -> Result<()> {
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

    // 3. Auto-install missing agents
    let skill_ids: Vec<&str> = plan.agents.iter().map(|a| a.skill_id.as_str()).collect();
    auto_install::auto_install_missing(&paths, &skill_ids)?;

    // 4. Ensure agent teams env var
    if !settings::has_teams_env_var(&paths.claude_dir) {
        settings::enable_teams_env_var(&paths.claude_dir)?;
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

    // 6. Build the prompt
    let prompt = build_execution_prompt(&plan, &plan_md);

    // 7. Check claude CLI exists
    if Command::new("claude").arg("--version").output().is_err() {
        return Err(ForjaError::ClaudeCliNotFound);
    }

    // 8. Launch claude
    println!("{}", "Launching Claude Code session...".bold());
    println!();

    let status = Command::new("claude")
        .arg("--dangerously-skip-permissions")
        .arg("--")
        .arg(&prompt)
        .status()?;

    // 9. Update plan status
    if status.success() {
        plan.status = PlanStatus::Executed;
        let plan_json_path = paths.plans.join(format!("{}.json", plan.id));
        save_plan(&plan_json_path, &plan)?;
        println!();
        println!(
            "{} Plan {} marked as executed.",
            "Done:".green().bold(),
            plan.id.cyan()
        );
    }

    Ok(())
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
    use crate::models::plan::{PlanAgent, PlanPhase, PlanStack};

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
}
