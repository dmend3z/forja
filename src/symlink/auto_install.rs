use colored::Colorize;

use crate::error::Result;
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::symlink::manager::{SymlinkManager, load_installed_ids, save_installed_ids};

/// Auto-install missing agent symlinks for the given skill IDs.
///
/// Prints progress to stdout and persists the updated installed list.
/// Silently succeeds if all skills are already installed.
pub fn auto_install_missing(paths: &ForjaPaths, skill_ids: &[&str]) -> Result<()> {
    let installed_ids = load_installed_ids(&paths.state);
    let missing: Vec<&str> = skill_ids
        .iter()
        .copied()
        .filter(|id| !installed_ids.contains(&id.to_string()))
        .collect();

    if missing.is_empty() {
        return Ok(());
    }

    println!(
        "{} Installing {} missing agent(s)...",
        "AUTO-INSTALL:".yellow().bold(),
        missing.len()
    );

    let mut current_ids = installed_ids;
    let registry = catalog::scan(&paths.registry, &current_ids)?;
    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());

    for skill_id in &missing {
        match registry.find_by_id(skill_id) {
            Some(skill) => match manager.install(skill) {
                Ok(_) => {
                    current_ids.push(skill_id.to_string());
                    println!("  {} {}", "installed".green(), skill_id);
                }
                Err(e) => {
                    eprintln!("  {} {} — {}", "failed".red(), skill_id, e);
                }
            },
            None => {
                eprintln!(
                    "  {} {} — not found in catalog",
                    "skipped".yellow(),
                    skill_id
                );
            }
        }
    }

    save_installed_ids(&paths.state, &current_ids)?;
    println!();

    Ok(())
}
