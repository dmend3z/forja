use crate::error::{ForjaError, Result};
use crate::paths::ForjaPaths;
use crate::symlink::manager::{load_installed_ids, save_installed_ids, SymlinkManager};
use colored::Colorize;

pub fn run(skill_path: &str) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    let mut installed_ids = load_installed_ids(&paths.state);
    if !installed_ids.contains(&skill_path.to_string()) {
        return Err(ForjaError::NotInstalled(skill_path.to_string()));
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
