use crate::error::Result;
use crate::models::state::load_state;
use crate::output;
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::settings;
use crate::symlink::manager::{SymlinkManager, load_installed_ids};
use colored::Colorize;

struct CheckResult {
    label: String,
    passed: bool,
    remediation: Option<String>,
}

/// Verify installation health: check paths, symlinks, env vars, and teams.
pub fn run() -> Result<()> {
    let paths = ForjaPaths::new()?;

    println!("{}", "forja health check".bold());
    println!();

    // Show mode
    let mode_label = match paths.mode {
        crate::paths::ForjaMode::Project => {
            format!("project ({})", paths.display_name())
        }
        crate::paths::ForjaMode::Global => "global".to_string(),
    };
    println!("  {} {}", "Mode:".cyan().bold(), mode_label);

    let mut results: Vec<CheckResult> = Vec::new();

    // Check forja root
    let root_label = if paths.mode == crate::paths::ForjaMode::Project {
        format!(".forja/ exists ({})", paths.display_name())
    } else {
        "~/.forja/ exists".to_string()
    };
    results.push(CheckResult {
        label: root_label,
        passed: paths.forja_root.exists(),
        remediation: Some("Run: forja init".into()),
    });

    // Check registry
    let registry_exists = paths.registry.exists();
    results.push(CheckResult {
        label: "Registry linked/cloned".into(),
        passed: registry_exists,
        remediation: Some("Run: forja init".into()),
    });

    // Check config
    results.push(CheckResult {
        label: "config.json exists".into(),
        passed: paths.config.exists(),
        remediation: Some("Run: forja init".into()),
    });

    // Check state
    results.push(CheckResult {
        label: "state.json exists".into(),
        passed: paths.state.exists(),
        remediation: Some("Run: forja init".into()),
    });

    // Check ~/.claude/
    results.push(CheckResult {
        label: "~/.claude/ exists".into(),
        passed: paths.claude_dir.exists(),
        remediation: Some("Install Claude Code from https://claude.ai/download".into()),
    });

    results.push(CheckResult {
        label: "~/.claude/agents/ exists".into(),
        passed: paths.claude_agents.exists(),
        remediation: Some("Run: mkdir -p ~/.claude/agents".into()),
    });

    // Check git is available
    let git_ok = std::process::Command::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    results.push(CheckResult {
        label: "git is available".into(),
        passed: git_ok,
        remediation: Some("Install git: https://git-scm.com/downloads".into()),
    });

    // Check symlinks
    if paths.claude_agents.exists() {
        let manager =
            SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
        let (healthy, broken) = manager.verify()?;

        let symlinks_ok = broken.is_empty();
        results.push(CheckResult {
            label: format!("Symlinks healthy ({})", healthy.len()),
            passed: symlinks_ok,
            remediation: if symlinks_ok {
                None
            } else {
                Some("Run: forja install --all (to recreate broken symlinks)".into())
            },
        });

        if !broken.is_empty() {
            for link in &broken {
                println!(
                    "    {} {}",
                    "BROKEN:".red(),
                    link.display().to_string().dimmed()
                );
            }
        }
    }

    // Check catalog
    if registry_exists {
        let installed_ids = load_installed_ids(&paths.state);
        let registry = catalog::scan(&paths.registry, &installed_ids)?;

        println!(
            "  {} {} skills available, {} installed",
            "CATALOG:".cyan().bold(),
            registry.skills.len(),
            installed_ids.len()
        );
    }

    // Check state.json is valid JSON
    if paths.state.exists() {
        let state_valid = std::fs::read_to_string(&paths.state)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .is_some();
        results.push(CheckResult {
            label: "state.json is valid JSON".into(),
            passed: state_valid,
            remediation: Some("Delete and reinitialize: rm ~/.forja/state.json && forja init".into()),
        });
    }

    // Check agent teams env var
    let teams_env = settings::has_teams_env_var(&paths.claude_dir);
    results.push(CheckResult {
        label: "Agent teams env var set".into(),
        passed: teams_env,
        remediation: Some(
            "Add CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1 to ~/.claude/settings.json".into(),
        ),
    });

    // Print all results
    println!();
    for result in &results {
        if result.passed {
            println!("  {} {}", "PASS".green().bold(), result.label);
        } else {
            println!("  {} {}", "FAIL".red().bold(), result.label);
            if let Some(ref rem) = result.remediation {
                output::print_command_hint("Fix:", rem);
            }
        }
    }

    // Check configured teams
    let state = load_state(&paths.state);
    if !state.teams.is_empty() {
        println!();
        println!(
            "  {} {} team(s) configured:",
            "TEAMS:".cyan().bold(),
            state.teams.len()
        );
        for (name, entry) in &state.teams {
            let cmd_path = paths
                .claude_commands
                .join(format!("forja--team--{}.md", name));
            let status = if cmd_path.exists() {
                "OK".green().to_string()
            } else {
                "MISSING CMD".red().to_string()
            };
            println!(
                "    {} {} (profile: {}, agents: {}) [{}]",
                "●".cyan(),
                name,
                entry.profile,
                entry.members.len(),
                status
            );
        }
    }

    // Summary
    let pass_count = results.iter().filter(|r| r.passed).count();
    let fail_count = results.len() - pass_count;

    println!();
    if fail_count == 0 {
        output::print_success(&format!("All {} checks passed", pass_count));
    } else {
        output::print_warning(&format!(
            "{} passed, {} failed",
            pass_count, fail_count
        ));
    }
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_result_stores_remediation() {
        let result = CheckResult {
            label: "test check".into(),
            passed: false,
            remediation: Some("do this".into()),
        };
        assert!(!result.passed);
        assert_eq!(result.remediation.as_deref(), Some("do this"));
    }

    #[test]
    fn check_result_passed_no_remediation() {
        let result = CheckResult {
            label: "good check".into(),
            passed: true,
            remediation: None,
        };
        assert!(result.passed);
        assert!(result.remediation.is_none());
    }
}
