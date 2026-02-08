use crate::error::Result;
use crate::models::state::load_state;
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::settings;
use crate::symlink::manager::{SymlinkManager, load_installed_ids};
use colored::Colorize;

/// Verify installation health: check paths, symlinks, env vars, and teams.
pub fn run() -> Result<()> {
    let paths = ForjaPaths::new()?;

    println!("{}", "forja health check".bold());
    println!();

    // Check ~/.forja/
    check("~/.forja/ exists", paths.forja_root.exists());

    // Check registry
    let registry_exists = paths.registry.exists();
    check("Registry linked/cloned", registry_exists);

    // Check config
    check("config.json exists", paths.config.exists());

    // Check state
    check("state.json exists", paths.state.exists());

    // Check ~/.claude/
    check("~/.claude/ exists", paths.claude_dir.exists());
    check("~/.claude/agents/ exists", paths.claude_agents.exists());

    // Check symlinks
    if paths.claude_agents.exists() {
        let manager =
            SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
        let (healthy, broken) = manager.verify()?;

        check(
            &format!("Symlinks healthy ({})", healthy.len()),
            broken.is_empty(),
        );

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

    // Check agent teams env var
    check(
        "Agent teams env var set",
        settings::has_teams_env_var(&paths.claude_dir),
    );

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

    println!();
    Ok(())
}

fn check(label: &str, ok: bool) {
    if ok {
        println!("  {} {label}", "PASS".green().bold());
    } else {
        println!("  {} {label}", "FAIL".red().bold());
    }
}
