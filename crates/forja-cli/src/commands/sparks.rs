use std::fs;
use std::path::Path;
use std::process::Command;

use colored::Colorize;
use dialoguer::Select;

use forja_core::error::{ForjaError, Result};
use forja_core::models::plan::{
    self, PhaseStatus, PlanMetadata, PlanStatus,
    checkpoint_path, initialize_checkpoint, load_checkpoint, save_checkpoint, save_plan,
    workspace_dir,
};
use forja_core::models::spec::{self, SpecFile};
use forja_core::paths::ForjaPaths;
use forja_core::settings;
use forja_core::symlink::auto_install;

use crate::commands::execute as exec;
use crate::output;

const DEFAULT_SPECS_DIR: &str = "docs/specs";

fn specs_dir(path: Option<&str>) -> &Path {
    Path::new(path.unwrap_or(DEFAULT_SPECS_DIR))
}

pub fn list(path: Option<&str>) -> Result<()> {
    let dir = specs_dir(path);
    let specs = spec::discover_specs(dir)?;

    if specs.is_empty() {
        println!("{}", "No specs found.".dimmed());
        output::print_tip(&format!("Create a spec in {}/", dir.display()));
        return Ok(());
    }

    println!("{}", "Specs".bold());
    println!();

    let rows: Vec<Vec<String>> = specs
        .iter()
        .map(|s| {
            vec![
                s.id().to_string(),
                s.title().to_string(),
                s.frontmatter
                    .priority
                    .as_deref()
                    .unwrap_or("-")
                    .to_string(),
                s.status.as_str().to_string(),
            ]
        })
        .collect();

    output::print_table(&["ID", "Title", "Priority", "Status"], &rows);
    println!();
    output::print_tip("Show details: forja sparks show <spec-id>");

    Ok(())
}

pub fn show(spec_id: &str) -> Result<()> {
    let dir = specs_dir(None);
    let spec = spec::find_spec(dir, spec_id)?;

    println!("{}", spec.title().bold());
    println!("{}", spec.frontmatter.description);
    println!();
    println!(
        "  ID:       {}",
        spec.id().cyan()
    );
    println!(
        "  Priority: {}",
        spec.frontmatter
            .priority
            .as_deref()
            .unwrap_or("-")
            .cyan()
    );
    println!(
        "  Status:   {}",
        spec.status.as_str().cyan()
    );

    if !spec.frontmatter.tags.is_empty() {
        println!(
            "  Tags:     {}",
            spec.frontmatter.tags.join(", ").dimmed()
        );
    }

    if !spec.frontmatter.requirements.is_empty() {
        println!();
        println!("  {}:", "Requirements".bold());
        for req in &spec.frontmatter.requirements {
            println!("    {} {}", "-".dimmed(), req);
        }
    }

    if !spec.frontmatter.constraints.is_empty() {
        println!();
        println!("  {}:", "Constraints".bold());
        for c in &spec.frontmatter.constraints {
            println!("    {} {}", "-".dimmed(), c);
        }
    }

    if !spec.frontmatter.success_criteria.is_empty() {
        println!();
        println!("  {}:", "Success Criteria".bold());
        for sc in &spec.frontmatter.success_criteria {
            println!("    {} {}", "-".dimmed(), sc);
        }
    }

    if !spec.body.is_empty() {
        println!();
        println!("  {}", "── Body ──".dimmed());
        for line in spec.body.lines() {
            println!("  {}", line);
        }
    }

    Ok(())
}

pub fn plan(spec_id: &str) -> Result<()> {
    let dir = specs_dir(None);
    let spec = spec::find_spec(dir, spec_id)?;

    let paths = ForjaPaths::ensure_initialized()?;
    fs::create_dir_all(&paths.plans)?;

    // Build structured task description from spec fields
    let task_description = spec::build_task_description(&spec);

    // Load forja-plan template from registry
    let template_path = paths
        .registry
        .join("skills/research/planning/forja-plan/commands/forja-plan.md");
    let template = fs::read_to_string(&template_path)
        .map_err(|_| ForjaError::SkillNotFound("research/planning/forja-plan".into()))?;

    // Strip frontmatter and inject spec content as $ARGUMENTS
    let mut prompt = forja_core::frontmatter::strip_frontmatter(&template)
        .replace("$ARGUMENTS", &task_description);

    // Instruct Claude to link the plan back to this spec
    prompt.push_str(&format!(
        "\n\n## Additional Instructions\n\n\
         When saving the plan JSON in Step 5, include this field:\n\
         `\"source_spec\": \"{spec_id}\"`\n\n\
         This links the plan back to the source spec for `forja sparks execute`.\n"
    ));

    // Check claude CLI
    if Command::new("claude").arg("--version").output().is_err() {
        return Err(ForjaError::ClaudeCliNotFound);
    }

    println!("{}", "forja sparks plan".bold());
    println!();
    println!("  Spec:  {} — {}", spec.id().cyan(), spec.title());
    println!();
    println!("{}", "Launching Claude Code session...".bold());
    println!();

    Command::new("claude")
        .arg("--dangerously-skip-permissions")
        .arg("--")
        .arg(&prompt)
        .status()?;

    Ok(())
}

pub fn execute(spec_id: &str, profile: &str, resume: bool) -> Result<()> {
    let dir = specs_dir(None);
    let _spec = spec::find_spec(dir, spec_id)?;

    let paths = ForjaPaths::ensure_initialized()?;

    // Find linked plan
    let mut plan = plan::find_plan_for_spec(&paths.plans, spec_id)?;

    if plan.status == PlanStatus::Executed {
        println!(
            "{} Plan {} is already executed.",
            "Done:".green().bold(),
            plan.id.cyan()
        );
        return Ok(());
    }

    println!("{}", "forja sparks execute".bold());
    println!();
    println!("  Spec:  {}", spec_id.cyan());
    println!("  Plan:  {}", plan.id.cyan());
    println!("  Task:  {}", plan.task);
    println!("  Team:  {}", plan.team_size);
    println!();

    // Override profile if provided
    if profile != "balanced" {
        plan.profile = profile.to_string();
    }

    // Auto-install missing agents
    let skill_ids: Vec<&str> = plan.agents.iter().map(|a| a.skill_id.as_str()).collect();
    auto_install::auto_install_missing(&paths, &skill_ids)?;

    // Track analytics
    let analytics_path = forja_core::analytics::analytics_path(&paths.forja_root);
    for agent in &plan.agents {
        let _ = forja_core::analytics::track(&analytics_path, &agent.skill_id, "sparks-execute");
    }

    // Ensure agent teams env var
    let global_claude = ForjaPaths::global_claude_dir()?;
    if !settings::has_teams_env_var(&global_claude) {
        settings::enable_teams_env_var(&global_claude)?;
        println!(
            "  {} Agent teams env var enabled in settings.json",
            "NOTE:".yellow().bold()
        );
        println!();
    }

    // Read plan .md content
    let plan_md_path = paths.plans.join(format!("{}.md", plan.id));
    let plan_md = if plan_md_path.exists() {
        fs::read_to_string(&plan_md_path)?
    } else {
        format!("# Plan: {}\n\nNo detailed plan file found.", plan.task)
    };

    // Check claude CLI exists
    if Command::new("claude").arg("--version").output().is_err() {
        return Err(ForjaError::ClaudeCliNotFound);
    }

    if plan.phases.is_empty() {
        println!("{}", "No structured phases — running monolithic execution.".dimmed());
        println!();
        return exec_monolithic(&paths, &mut plan, &plan_md);
    }

    exec_phased(&paths, &mut plan, &plan_md, resume)
}

/// Monolithic execution for plans without phases.
fn exec_monolithic(paths: &ForjaPaths, plan: &mut PlanMetadata, plan_md: &str) -> Result<()> {
    let prompt = format!(
        "Execute this implementation plan.\n\n## Plan\n\n{plan_md}\n\n\
         ## Rules\n\n\
         - Read CLAUDE.md before starting\n\
         - Stop and report if blocked\n"
    );

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

/// Phase-by-phase execution with retry-then-ask and quality gates.
fn exec_phased(
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

    fs::create_dir_all(&ws_dir)?;
    save_checkpoint(&ckpt_path, &checkpoint)?;

    println!(
        "{} Executing {} phases...",
        "PHASES:".bold(),
        plan.phases.len()
    );
    println!();

    let mut all_completed = true;

    for i in 0..plan.phases.len() {
        let phase_ckpt = &checkpoint.phases[i];

        // Skip completed phases
        if phase_ckpt.status == PhaseStatus::Completed {
            println!(
                "  {} Phase {}: {} (already completed)",
                "✓".green(),
                i + 1,
                plan.phases[i].name
            );
            continue;
        }

        // Skip phases whose dependencies failed
        if exec::has_failed_dependency(&plan.phases[i], &plan.phases, &checkpoint) {
            checkpoint.phases[i].status = PhaseStatus::Skipped;
            checkpoint.last_updated = chrono::Utc::now().to_rfc3339();
            save_checkpoint(&ckpt_path, &checkpoint)?;
            println!(
                "  {} Phase {}: {} (dependency failed)",
                "⊘".yellow(),
                i + 1,
                plan.phases[i].name
            );
            all_completed = false;
            continue;
        }

        // Execute phase with retry-then-ask
        let succeeded = run_phase_with_retry(paths, plan, &mut checkpoint, i, plan_md, &ws_dir, &ckpt_path)?;

        if !succeeded {
            all_completed = false;
        }

        // Quality gates after successful phase
        if succeeded {
            run_quality_gates(i, &plan.phases[i].name);
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

/// Run a single phase, retrying once on failure, then prompting the user.
fn run_phase_with_retry(
    paths: &ForjaPaths,
    plan: &PlanMetadata,
    checkpoint: &mut plan::ExecutionCheckpoint,
    phase_index: usize,
    plan_md: &str,
    ws_dir: &Path,
    ckpt_path: &Path,
) -> Result<bool> {
    let phase_name = &plan.phases[phase_index].name;

    for attempt in 0..2 {
        if attempt == 1 {
            println!(
                "  {} Retrying phase {}...",
                "RETRY:".yellow().bold(),
                phase_index + 1
            );
        } else {
            println!("  {} Phase {}: {}", "▶".cyan(), phase_index + 1, phase_name.bold());
        }

        // Mark in-progress
        checkpoint.phases[phase_index].status = PhaseStatus::InProgress;
        checkpoint.phases[phase_index].started_at = Some(chrono::Utc::now().to_rfc3339());
        checkpoint.current_phase = Some(phase_index);
        checkpoint.last_updated = chrono::Utc::now().to_rfc3339();
        save_checkpoint(ckpt_path, checkpoint)?;

        // Build and run prompt
        let prompt = exec::build_phase_prompt(plan, phase_index, plan_md, ws_dir);
        let status = Command::new("claude")
            .arg("--dangerously-skip-permissions")
            .arg("--")
            .arg(&prompt)
            .status()?;

        let exit_code = status.code().unwrap_or(-1);

        if status.success() {
            checkpoint.phases[phase_index].status = PhaseStatus::Completed;
            checkpoint.phases[phase_index].completed_at = Some(chrono::Utc::now().to_rfc3339());
            checkpoint.phases[phase_index].exit_code = Some(exit_code);
            checkpoint.last_updated = chrono::Utc::now().to_rfc3339();
            save_checkpoint(ckpt_path, checkpoint)?;

            println!(
                "  {} Phase {}: {} completed",
                "✓".green(),
                phase_index + 1,
                phase_name
            );
            return Ok(true);
        }

        // First attempt failed — will retry
        if attempt == 0 {
            println!(
                "  {} Phase {}: {} failed (exit code {}) — retrying...",
                "✗".red(),
                phase_index + 1,
                phase_name,
                exit_code
            );
            continue;
        }

        // Second attempt also failed — prompt user
        checkpoint.phases[phase_index].status = PhaseStatus::Failed;
        checkpoint.phases[phase_index].completed_at = Some(chrono::Utc::now().to_rfc3339());
        checkpoint.phases[phase_index].exit_code = Some(exit_code);
        checkpoint.phases[phase_index].error_message =
            Some(format!("Failed after retry (exit code {exit_code})"));
        checkpoint.last_updated = chrono::Utc::now().to_rfc3339();
        save_checkpoint(ckpt_path, checkpoint)?;

        println!(
            "  {} Phase {}: {} failed after retry (exit code {})",
            "✗".red(),
            phase_index + 1,
            phase_name,
            exit_code
        );
        println!();

        return handle_phase_failure(paths, plan, checkpoint, phase_index, plan_md, ws_dir, ckpt_path);
    }

    unreachable!()
}

/// Prompt user after a phase fails twice: Retry / Skip / Abort.
fn handle_phase_failure(
    paths: &ForjaPaths,
    plan: &PlanMetadata,
    checkpoint: &mut plan::ExecutionCheckpoint,
    phase_index: usize,
    plan_md: &str,
    ws_dir: &Path,
    ckpt_path: &Path,
) -> Result<bool> {
    let items = &["Retry", "Skip this phase", "Abort execution"];

    let selection = Select::new()
        .with_prompt(format!(
            "Phase '{}' failed. What would you like to do?",
            plan.phases[phase_index].name
        ))
        .items(items)
        .default(0)
        .interact_opt()
        .unwrap_or(None);

    match selection {
        Some(0) => {
            // Retry — recursive call with fresh retry counter
            checkpoint.phases[phase_index].status = PhaseStatus::Pending;
            checkpoint.phases[phase_index].error_message = None;
            save_checkpoint(ckpt_path, checkpoint)?;
            run_phase_with_retry(paths, plan, checkpoint, phase_index, plan_md, ws_dir, ckpt_path)
        }
        Some(1) => {
            // Skip
            checkpoint.phases[phase_index].status = PhaseStatus::Skipped;
            checkpoint.last_updated = chrono::Utc::now().to_rfc3339();
            save_checkpoint(ckpt_path, checkpoint)?;
            println!(
                "  {} Phase {}: {} skipped",
                "⊘".yellow(),
                phase_index + 1,
                plan.phases[phase_index].name
            );
            Ok(false)
        }
        _ => {
            // Abort (including None = Esc/Ctrl+C)
            Err(ForjaError::PhaseExecutionFailed(format!(
                "Execution aborted at phase '{}'",
                plan.phases[phase_index].name
            )))
        }
    }
}

/// Run quality gates (cargo test + cargo clippy) after a completed phase.
fn run_quality_gates(phase_index: usize, phase_name: &str) {
    println!(
        "  {} Quality gates for phase {}...",
        "GATES:".bold(),
        phase_index + 1
    );

    // cargo test
    let test_result = Command::new("cargo")
        .args(["test", "--workspace"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match test_result {
        Ok(s) if s.success() => {
            println!("    {} cargo test", "✓".green());
        }
        Ok(s) => {
            let code = s.code().unwrap_or(-1);
            println!(
                "    {} cargo test (exit code {})",
                "✗".red(),
                code
            );
            println!(
                "      {} Tests failed after phase '{}' — review before continuing",
                "Warning:".yellow().bold(),
                phase_name
            );
        }
        Err(_) => {
            println!("    {} cargo test (not available)", "·".dimmed());
        }
    }

    // cargo clippy
    let clippy_result = Command::new("cargo")
        .args(["clippy", "--workspace", "--", "-D", "warnings"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match clippy_result {
        Ok(s) if s.success() => {
            println!("    {} cargo clippy", "✓".green());
        }
        Ok(s) => {
            let code = s.code().unwrap_or(-1);
            println!(
                "    {} cargo clippy (exit code {})",
                "✗".red(),
                code
            );
        }
        Err(_) => {
            println!("    {} cargo clippy (not available)", "·".dimmed());
        }
    }

    println!();
}

pub fn status(spec_id: Option<&str>) -> Result<()> {
    let dir = specs_dir(None);

    match spec_id {
        Some(id) => status_detail(dir, id),
        None => status_summary(dir),
    }
}

/// Summary table: all specs with derived status from linked plans.
fn status_summary(dir: &Path) -> Result<()> {
    let specs = spec::discover_specs(dir)?;

    if specs.is_empty() {
        println!("{}", "No specs found.".dimmed());
        output::print_tip(&format!("Create a spec in {}/", dir.display()));
        return Ok(());
    }

    // Try to load plans dir for status derivation
    let paths = ForjaPaths::resolve().ok();
    let plans_dir = paths.as_ref().map(|p| p.plans.as_path());

    println!("{}", "Spec Status".bold());
    println!();

    let rows: Vec<Vec<String>> = specs
        .iter()
        .map(|s| {
            let (status_label, plan_id) = derive_status(s, plans_dir);
            vec![
                s.id().to_string(),
                s.title().to_string(),
                status_label,
                plan_id.unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();

    output::print_table(&["ID", "Title", "Status", "Plan"], &rows);
    println!();
    output::print_tip("Details: forja sparks status <spec-id>");

    Ok(())
}

/// Detailed view: phase-by-phase progress for a single spec.
fn status_detail(dir: &Path, spec_id: &str) -> Result<()> {
    let spec = spec::find_spec(dir, spec_id)?;
    let paths = ForjaPaths::resolve().ok();
    let plans_dir = paths.as_ref().map(|p| p.plans.as_path());

    println!("{}", spec.title().bold());
    println!("{}", spec.frontmatter.description.dimmed());
    println!();

    // Find linked plan
    let plan = plans_dir.and_then(|pd| plan::find_plan_for_spec(pd, spec_id).ok());

    let Some(plan) = plan else {
        println!("  Status: {}", "draft".dimmed());
        println!();
        println!("  {}", "No plan generated yet.".dimmed());
        output::print_tip(&format!("Generate a plan: forja sparks plan {spec_id}"));
        return Ok(());
    };

    // Safety: if `plan` was found via `find_plan_for_spec`, then `plans_dir` is Some.
    let (status_label, _) = derive_status_from_plan(&plan, plans_dir.expect("plans_dir exists when plan was found"));

    println!("  Plan:   {}", plan.id.cyan());
    println!("  Team:   {}", plan.team_size);
    println!("  Status: {}", status_label);
    println!();

    // Show phases if they exist
    if plan.phases.is_empty() {
        println!("  {}", "No structured phases in this plan.".dimmed());
        return Ok(());
    }

    // Load checkpoint for phase progress
    let checkpoint = plans_dir
        .map(|pd| plan::checkpoint_path(pd, &plan.id))
        .filter(|p| p.exists())
        .and_then(|p| plan::load_checkpoint(&p).ok());

    println!("  {}:", "Phases".bold());
    println!();

    for (i, phase) in plan.phases.iter().enumerate() {
        let (icon, name_style) = if let Some(ref ckpt) = checkpoint {
            match &ckpt.phases[i].status {
                PhaseStatus::Completed => ("✓".green().to_string(), phase.name.green().to_string()),
                PhaseStatus::InProgress => {
                    ("▶".yellow().to_string(), phase.name.yellow().bold().to_string())
                }
                PhaseStatus::Failed => ("✗".red().to_string(), phase.name.red().to_string()),
                PhaseStatus::Skipped => ("⊘".yellow().to_string(), phase.name.dimmed().to_string()),
                PhaseStatus::Pending => ("·".dimmed().to_string(), phase.name.dimmed().to_string()),
            }
        } else {
            ("·".dimmed().to_string(), phase.name.dimmed().to_string())
        };

        println!(
            "  {} Phase {}: {} ({})",
            icon,
            i + 1,
            name_style,
            phase.agent_role.dimmed()
        );

        // Show error message for failed phases
        if let Some(ref ckpt) = checkpoint
            && ckpt.phases[i].status == PhaseStatus::Failed
            && let Some(ref err) = ckpt.phases[i].error_message
        {
            println!("      {}", err.red());
        }
    }

    // Quality gates
    if !plan.quality_gates.is_empty() {
        println!();
        println!("  {}:", "Quality Gates".bold());
        for gate in &plan.quality_gates {
            let is_complete = plan.status == PlanStatus::Executed;
            let icon = if is_complete {
                "✓".green().to_string()
            } else {
                "·".dimmed().to_string()
            };
            println!("  {} {}", icon, gate);
        }
    }

    println!();

    // Next action hint
    match plan.status {
        PlanStatus::Pending => {
            output::print_tip(&format!("Execute: forja sparks execute {spec_id}"));
        }
        PlanStatus::Executed => {
            println!("  {}", "All phases completed.".green().bold());
        }
        PlanStatus::Archived => {
            println!("  {}", "Plan archived.".dimmed());
        }
    }

    Ok(())
}

/// Derive a display status string and optional plan ID for a spec.
fn derive_status(spec: &SpecFile, plans_dir: Option<&Path>) -> (String, Option<String>) {
    let Some(plans_dir) = plans_dir else {
        return ("draft".dimmed().to_string(), None);
    };

    match plan::find_plan_for_spec(plans_dir, spec.id()) {
        Ok(plan) => {
            let (label, _) = derive_status_from_plan(&plan, plans_dir);
            (label, Some(plan.id.clone()))
        }
        Err(_) => ("draft".dimmed().to_string(), None),
    }
}

/// Derive display status from a plan and its checkpoint.
fn derive_status_from_plan(plan: &PlanMetadata, plans_dir: &Path) -> (String, Option<String>) {
    match plan.status {
        PlanStatus::Executed => ("complete".green().to_string(), None),
        PlanStatus::Archived => ("archived".dimmed().to_string(), None),
        PlanStatus::Pending => {
            // Check checkpoint for finer-grained status
            let ckpt_path = plan::checkpoint_path(plans_dir, &plan.id);
            if ckpt_path.exists()
                && let Ok(ckpt) = plan::load_checkpoint(&ckpt_path)
            {
                let has_failed = ckpt.phases.iter().any(|p| p.status == PhaseStatus::Failed);
                let has_in_progress =
                    ckpt.phases.iter().any(|p| p.status == PhaseStatus::InProgress);

                if has_failed {
                    return ("failed".red().to_string(), None);
                }
                if has_in_progress {
                    return ("executing".yellow().to_string(), None);
                }

                let completed = ckpt
                    .phases
                    .iter()
                    .filter(|p| p.status == PhaseStatus::Completed)
                    .count();
                if completed > 0 {
                    return (
                        format!("{}/{} phases", completed, ckpt.phases.len())
                            .yellow()
                            .to_string(),
                        None,
                    );
                }
            }
            ("ready".cyan().to_string(), None)
        }
    }
}
