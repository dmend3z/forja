use dialoguer::Confirm;

use crate::error::{ForjaError, Result};
use crate::paths::ForjaPaths;
use crate::symlink::manager::{SymlinkManager, load_installed_ids, save_installed_ids};
use colored::Colorize;

/// Remove an installed skill by deleting its symlinks and updating state.
pub fn run(skill_path: &str, skip_confirm: bool) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    let mut installed_ids = load_installed_ids(&paths.state);
    if !installed_ids.contains(&skill_path.to_string()) {
        return Err(ForjaError::NotInstalled(skill_path.to_string()));
    }

    if !skip_confirm {
        let confirmed = Confirm::new()
            .with_prompt(format!("Uninstall skill '{}'?", skill_path))
            .default(true)
            .interact()
            .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

        if !confirmed {
            return Err(ForjaError::PromptCancelled);
        }
    }

    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    let removed = manager.uninstall(skill_path)?;

    installed_ids.retain(|id| id != skill_path);
    save_installed_ids(&paths.state, &installed_ids)?;

    println!("{} {}", "Uninstalled:".yellow().bold(), skill_path.bold());

    if !removed.is_empty() {
        println!("  Symlinks removed:");
        for link in &removed {
            println!("    {}", link.display().to_string().dimmed());
        }
    }

    Ok(())
}
