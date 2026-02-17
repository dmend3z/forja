use crate::error::Result;
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::symlink::manager::{SymlinkManager, load_installed_ids};
use crate::symlink::sync;
use colored::Colorize;

/// Update the registry via `git pull` and re-verify installed symlinks.
pub fn run() -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    // Check if local development mode (symlink, not cloned)
    if paths.registry.is_symlink() {
        println!(
            "{}",
            "Registry is a local symlink — no update needed.".yellow()
        );
        println!(
            "  Source: {}",
            std::fs::read_link(&paths.registry)
                .unwrap_or_default()
                .display()
        );
    } else {
        // Save HEAD before pull for `forja diff`
        if let Ok(head) = crate::registry::git::head_sha(&paths.registry) {
            let last_update_path = paths.forja_root.join("last_update.json");
            let data = serde_json::json!({ "head_before": head });
            let _ = std::fs::write(&last_update_path, serde_json::to_string_pretty(&data).unwrap_or_default());
        }

        println!("Updating registry...");
        let output = crate::registry::git::pull(&paths.registry)?;
        println!("{output}");
    }

    // Sync symlinks after update to refresh any changed skill files
    sync::sync_symlinks(&paths)?;

    // Check symlink health
    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    let (healthy, broken) = manager.verify()?;

    // Show catalog stats
    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;

    println!();
    println!(
        "{} {} available, {} installed, {} symlinks healthy",
        "Status:".bold(),
        registry.skills.len().to_string().cyan(),
        installed_ids.len().to_string().green(),
        healthy.len().to_string().green(),
    );

    if registry.skills.len() > installed_ids.len() {
        println!(
            "  {} {} to install all",
            "Tip:".dimmed(),
            "forja install --all".cyan()
        );
    }

    if !broken.is_empty() {
        println!(
            "  {} {} broken symlinks found",
            "WARNING:".yellow().bold(),
            broken.len()
        );
        for link in &broken {
            println!("    {}", link.display().to_string().dimmed());
        }
    }

    Ok(())
}
