use crate::error::Result;
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::symlink::manager::load_installed_ids;
use colored::Colorize;

/// Search the skill catalog by name, description, phase, or tech.
pub fn run(query: &str) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let results = registry.search(query);

    if results.is_empty() {
        println!("No skills matching \"{}\"", query.yellow());
        println!();
        println!(
            "  {} Try 'forja list --available' to browse all skills",
            "Tip:".cyan().bold()
        );
        return Ok(());
    }

    println!(
        "{} {} results for \"{}\"",
        "Found".bold(),
        results.len(),
        query.yellow()
    );
    println!();

    for skill in &results {
        let status = if skill.installed {
            " [installed]".green().to_string()
        } else {
            String::new()
        };
        println!(
            "  {} {}{}",
            skill.id.bold(),
            skill.description.dimmed(),
            status
        );
    }

    Ok(())
}
