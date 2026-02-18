use std::process::Command;

use forja_core::error::Result;
use forja_core::models::state::load_state;
use forja_core::paths::ForjaPaths;
use forja_core::registry::catalog;
use forja_core::symlink::manager::{SymlinkManager, load_installed_ids};
use crate::tips;
use colored::Colorize;

/// Show installation status or welcome message (default command when no subcommand given).
pub fn run() -> Result<()> {
    let paths = ForjaPaths::new()?;

    if !paths.forja_root.exists() {
        print_welcome();
        return Ok(());
    }

    print_status(&paths)
}

fn print_welcome() {
    println!();
    println!("  {}", "forja".bold());
    println!("  Skills marketplace for Claude Code");
    println!();
    println!("  Curated skills for research, coding, testing,");
    println!("  code review, and deployment — installed in seconds.");
    println!();
    println!("  Get started:");
    println!("    {}", "forja init".cyan());
    println!();
    println!("  Run {} for all commands", "forja --help".dimmed());
    println!();
}

fn print_status(paths: &ForjaPaths) -> Result<()> {
    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let total = registry.skills.len();
    let installed = installed_ids.len();

    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    let (_healthy, broken) = manager.verify()?;

    let health = if broken.is_empty() {
        "all symlinks OK".green().to_string()
    } else {
        format!("{} broken symlink(s)", broken.len())
            .red()
            .to_string()
    };

    let state = load_state(&paths.state);

    let mode_label = match paths.mode {
        forja_core::paths::ForjaMode::Project => {
            format!("project ({})", paths.display_name())
        }
        forja_core::paths::ForjaMode::Global => "global".to_string(),
    };

    println!();
    println!("  {}", "forja".bold());
    println!();
    println!("  Mode:    {}", mode_label.cyan());
    println!("  Skills:  {}/{} installed", installed, total);
    println!("  Health:  {}", health);
    println!();

    // Git context
    let branch = git_branch();
    let changes_count = git_changes_count();
    let pending_plans = count_pending_plans(&paths.plans);

    if let Some(ref b) = branch {
        println!("  Branch:  {}", b.cyan());
    }
    if changes_count > 0 {
        println!(
            "  Changes: {}",
            format!("{} uncommitted file(s)", changes_count).yellow()
        );
    }
    if pending_plans > 0 {
        println!(
            "  Plans:   {}",
            format!("{} pending", pending_plans).cyan()
        );
    }
    println!();

    // Smart suggestion
    let suggestion = smart_suggestion(changes_count, pending_plans, &branch);
    println!("  {} {}", "Next:".green().bold(), suggestion.dimmed());
    println!();

    let ctx = tips::TipContext {
        installed_count: installed,
        total_skills: total,
        has_teams: !state.teams.is_empty(),
        is_initialized: true,
    };
    tips::print_random_tip(&ctx);
    println!();

    Ok(())
}

fn git_branch() -> Option<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        None
    } else {
        Some(branch)
    }
}

fn git_changes_count() -> usize {
    let output = Command::new("git")
        .args(["status", "--short"])
        .output()
        .ok();

    match output {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .count(),
        _ => 0,
    }
}

fn count_pending_plans(plans_dir: &std::path::Path) -> usize {
    if !plans_dir.exists() {
        return 0;
    }

    std::fs::read_dir(plans_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "json")
        })
        .filter(|e| {
            std::fs::read_to_string(e.path())
                .ok()
                .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                .and_then(|v| v.get("status").and_then(|s| s.as_str().map(String::from)))
                .is_some_and(|s| s == "pending")
        })
        .count()
}

fn smart_suggestion(changes: usize, plans: usize, branch: &Option<String>) -> &'static str {
    if changes > 0 {
        return "forja review  or  forja ship";
    }
    if plans > 0 {
        return "forja execute";
    }
    let is_feature_branch = branch
        .as_ref()
        .is_some_and(|b| b != "main" && b != "master");
    if is_feature_branch {
        return "forja ship";
    }
    "forja plan  or  forja task"
}
