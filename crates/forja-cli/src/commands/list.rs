use forja_core::error::Result;
use forja_core::models::phase::Phase;
use crate::output;
use forja_core::paths::ForjaPaths;
use forja_core::registry::catalog;
use forja_core::symlink::manager::load_installed_ids;
use colored::Colorize;

/// List installed skills, or all available skills with `--available`.
pub fn run(available: bool) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;

    if available {
        println!("{}", "Available skills".bold());
        println!();

        for phase in Phase::all() {
            let phase_skills: Vec<_> = registry
                .skills
                .iter()
                .filter(|s| s.phase == *phase)
                .collect();

            if phase_skills.is_empty() {
                continue;
            }

            println!(
                "  {} ({})",
                phase.as_str().to_uppercase().cyan().bold(),
                phase_skills.len()
            );

            for skill in &phase_skills {
                let status = if skill.installed {
                    " [installed]".green().to_string()
                } else {
                    String::new()
                };
                println!(
                    "    {} {}{}",
                    skill.id.bold(),
                    skill.description.dimmed(),
                    status
                );
            }
            println!();
        }

        output::print_tip("Install a skill: forja install <skill-id>");
    } else if installed_ids.is_empty() {
        println!("{}", "No skills installed.".dimmed());
        println!(
            "Use {} to see available skills.",
            "forja list --available".cyan()
        );
        return Ok(());
    } else {
        println!("{}", "Installed skills".bold());
        println!();

        for id in &installed_ids {
            if let Some(skill) = registry.find_by_id(id) {
                println!("  {} {}", skill.id.bold(), skill.description.dimmed());
            } else {
                println!("  {} {}", id.bold(), "(not found in catalog)".red());
            }
        }
    }

    Ok(())
}
