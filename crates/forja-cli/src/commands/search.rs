use forja_core::error::Result;
use crate::output;
use forja_core::paths::ForjaPaths;
use forja_core::registry::catalog;
use forja_core::symlink::manager::load_installed_ids;
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
        output::print_tip("Try 'forja list --available' to browse all skills");
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

    println!();
    output::print_tip("Install a skill: forja install <skill-id>");

    Ok(())
}
