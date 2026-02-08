use crate::error::{ForjaError, Result};
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::symlink::manager::{load_installed_ids, save_installed_ids, SymlinkManager};
use colored::Colorize;

pub fn run_all() -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    let mut installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());

    let mut installed_count = 0;
    let mut skipped_count = 0;

    for skill in &registry.skills {
        if installed_ids.contains(&skill.id) {
            skipped_count += 1;
            continue;
        }

        match manager.install(skill) {
            Ok(_) => {
                installed_ids.push(skill.id.clone());
                installed_count += 1;
                println!(
                    "  {} {}",
                    "✓".green(),
                    skill.name
                );
            }
            Err(e) => {
                eprintln!(
                    "  {} {} — {}",
                    "✗".red(),
                    skill.name,
                    e
                );
            }
        }
    }

    save_installed_ids(&paths.state, &installed_ids)?;

    println!();
    println!(
        "{} {} installed, {} already installed",
        "Done:".green().bold(),
        installed_count,
        skipped_count
    );

    Ok(())
}

/// Install all skills without per-skill output. Returns (installed, skipped) counts.
/// Used by `forja init` to auto-install everything in one go.
pub fn install_all_quiet(paths: &ForjaPaths) -> Result<(usize, usize)> {
    let mut installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());

    let mut installed_count = 0;
    let mut skipped_count = 0;

    for skill in &registry.skills {
        if installed_ids.contains(&skill.id) {
            skipped_count += 1;
            continue;
        }

        if manager.install(skill).is_ok() {
            installed_ids.push(skill.id.clone());
            installed_count += 1;
        }
    }

    save_installed_ids(&paths.state, &installed_ids)?;

    Ok((installed_count, skipped_count))
}

pub fn run(skill_path: &str) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    let mut installed_ids = load_installed_ids(&paths.state);
    if installed_ids.contains(&skill_path.to_string()) {
        return Err(ForjaError::AlreadyInstalled(skill_path.to_string()));
    }

    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let skill = registry
        .find_by_id(skill_path)
        .ok_or_else(|| ForjaError::SkillNotFound(skill_path.to_string()))?;

    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    let created = manager.install(skill)?;

    installed_ids.push(skill_path.to_string());
    save_installed_ids(&paths.state, &installed_ids)?;

    println!(
        "{} {}",
        "Installed:".green().bold(),
        skill.name.bold()
    );
    println!("  Phase: {}", skill.phase.as_str().cyan());
    println!("  Tech:  {}", skill.tech.cyan());
    println!("  {}", skill.description.dimmed());

    if !created.is_empty() {
        println!();
        println!("  Symlinks created:");
        for link in &created {
            println!("    {}", link.display().to_string().dimmed());
        }
    }

    let types: Vec<_> = skill.content_types.iter().map(|t| t.to_string()).collect();
    if !types.is_empty() {
        println!("  Content: {}", types.join(", "));
    }

    Ok(())
}
