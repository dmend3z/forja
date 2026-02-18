use colored::Colorize;
use dialoguer::Confirm;

use forja_core::error::{ForjaError, Result};
use forja_core::paths::ForjaPaths;
use forja_core::symlink::upgrade;

/// Reinstall skills that were modified since the last `forja update`.
pub fn run(skill_filter: Option<&str>, yes: bool) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    if paths.registry.is_symlink() {
        println!(
            "{}",
            "Registry is a local symlink — upgrade is not needed in dev mode.".yellow()
        );
        return Ok(());
    }

    let upgradable = upgrade::find_upgradable(&paths)?;

    if upgradable.is_empty() {
        println!("  All installed skills are up to date.");
        return Ok(());
    }

    // Filter to specific skill if requested
    let to_upgrade: Vec<&str> = match skill_filter {
        Some(filter) => {
            let matches: Vec<&str> = upgradable
                .iter()
                .filter(|id| id.contains(filter))
                .map(|s| s.as_str())
                .collect();
            if matches.is_empty() {
                println!(
                    "  No upgradable skills match '{}'. Available: {}",
                    filter,
                    upgradable.join(", ")
                );
                return Ok(());
            }
            matches
        }
        None => upgradable.iter().map(|s| s.as_str()).collect(),
    };

    println!("{}", "forja upgrade".bold());
    println!();
    println!("  {} skill(s) to upgrade:", to_upgrade.len());
    for id in &to_upgrade {
        println!("    {}", id.cyan());
    }
    println!();

    if !yes {
        let confirmed = Confirm::new()
            .with_prompt("Proceed with upgrade?")
            .default(true)
            .interact()
            .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

        if !confirmed {
            return Err(ForjaError::PromptCancelled);
        }
    }

    for id in &to_upgrade {
        match upgrade::reinstall_skill(&paths, id) {
            Ok(()) => println!("  {} {}", "upgraded".green(), id),
            Err(e) => println!("  {} {} — {}", "failed".red(), id, e),
        }
    }

    println!();
    println!("{} Upgrade complete.", "Done:".green().bold());

    Ok(())
}
