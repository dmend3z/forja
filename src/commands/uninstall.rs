use dialoguer::Confirm;

use crate::error::{ForjaError, Result};
use crate::models::registry::ResolveResult;
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::symlink::manager::{SymlinkManager, load_installed_ids, save_installed_ids};
use colored::Colorize;

/// Remove an installed skill by deleting its symlinks and updating state.
pub fn run(skill_path: &str, skip_confirm: bool) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    let mut installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;

    // Resolve the identifier to a concrete skill ID
    let resolved_id = match registry.resolve(skill_path) {
        ResolveResult::Found(s) => s.id.clone(),
        ResolveResult::NotFound => {
            return Err(ForjaError::NotInstalled(skill_path.to_string()));
        }
        ResolveResult::Ambiguous(matches) => {
            println!(
                "{} '{}' matches multiple skills:",
                "Ambiguous:".yellow().bold(),
                skill_path
            );
            for s in &matches {
                println!("  {} — {}", s.id.cyan(), s.description.dimmed());
            }
            return Err(ForjaError::AmbiguousSkillName(skill_path.to_string()));
        }
    };

    if !installed_ids.contains(&resolved_id) {
        return Err(ForjaError::NotInstalled(resolved_id));
    }

    if !skip_confirm {
        let confirmed = Confirm::new()
            .with_prompt(format!("Uninstall skill '{}'?", resolved_id))
            .default(true)
            .interact()
            .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

        if !confirmed {
            return Err(ForjaError::PromptCancelled);
        }
    }

    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    let removed = manager.uninstall(&resolved_id)?;

    installed_ids.retain(|id| id != &resolved_id);
    save_installed_ids(&paths.state, &installed_ids)?;

    println!("{} {}", "Uninstalled:".yellow().bold(), resolved_id.bold());

    if !removed.is_empty() {
        println!("  Symlinks removed:");
        for link in &removed {
            println!("    {}", link.display().to_string().dimmed());
        }
    }

    Ok(())
}
