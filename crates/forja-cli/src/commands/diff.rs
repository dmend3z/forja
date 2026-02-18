use colored::Colorize;

use forja_core::error::Result;
use forja_core::paths::ForjaPaths;
use forja_core::registry::diff::{ChangeType, compute_diff, load_previous_head};
use forja_core::registry::git;
use forja_core::symlink::manager::load_installed_ids;

/// Show skill changes since the last `forja update`.
pub fn run() -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    if paths.registry.is_symlink() {
        println!(
            "{}",
            "Registry is a local symlink — diff is not available in dev mode.".yellow()
        );
        return Ok(());
    }

    let old_head = load_previous_head(&paths.forja_root)?;
    let new_head = git::head_sha(&paths.registry)?;

    if old_head == new_head {
        println!("  No changes since last update.");
        return Ok(());
    }

    let changes = compute_diff(&paths.registry, &old_head, &new_head)?;

    if changes.is_empty() {
        println!("  No skill changes since last update.");
        return Ok(());
    }

    let installed_ids = load_installed_ids(&paths.state);

    println!("{}", "forja diff".bold());
    println!();
    println!(
        "  Changes since last update ({} → {}):",
        old_head[..8].dimmed(),
        new_head[..8.min(new_head.len())].dimmed()
    );
    println!();

    let mut added = 0;
    let mut modified = 0;
    let mut removed = 0;

    for change in &changes {
        let is_installed = installed_ids.contains(&change.skill_id);
        let installed_marker = if is_installed {
            " (installed)".yellow().to_string()
        } else {
            String::new()
        };

        match change.change_type {
            ChangeType::Added => {
                added += 1;
                println!(
                    "    {} {}{}",
                    "+".green().bold(),
                    change.skill_id.green(),
                    installed_marker
                );
            }
            ChangeType::Modified => {
                modified += 1;
                println!(
                    "    {} {}{}",
                    "~".yellow().bold(),
                    change.skill_id.yellow(),
                    installed_marker
                );
            }
            ChangeType::Removed => {
                removed += 1;
                println!(
                    "    {} {}{}",
                    "-".red().bold(),
                    change.skill_id.red(),
                    installed_marker
                );
            }
        }
    }

    println!();
    println!(
        "  {} {} added, {} modified, {} removed",
        "Summary:".bold(),
        added.to_string().green(),
        modified.to_string().yellow(),
        removed.to_string().red(),
    );

    let modified_installed: Vec<&str> = changes
        .iter()
        .filter(|c| c.change_type == ChangeType::Modified && installed_ids.contains(&c.skill_id))
        .map(|c| c.skill_id.as_str())
        .collect();

    if !modified_installed.is_empty() {
        println!();
        println!(
            "  {} {} installed skill(s) were modified. Run: {}",
            "Tip:".cyan().bold(),
            modified_installed.len(),
            "forja upgrade".cyan()
        );
    }

    Ok(())
}
