use forja_core::error::{ForjaError, Result};
use forja_core::models::registry::ResolveResult;
use crate::output;
use forja_core::paths::ForjaPaths;
use forja_core::registry::catalog;
use forja_core::symlink::manager::{SymlinkManager, load_installed_ids, save_installed_ids};
use colored::Colorize;

/// Resolve paths based on --global flag: force global or auto-detect.
fn resolve_paths(force_global: bool) -> Result<ForjaPaths> {
    if force_global {
        let paths = ForjaPaths::global()?;
        if !paths.forja_root.exists() {
            return Err(ForjaError::NotInitialized);
        }
        Ok(paths)
    } else {
        ForjaPaths::ensure_initialized()
    }
}

struct InstallCounts {
    installed: usize,
    skipped: usize,
    failed: usize,
}

fn install_all_skills(paths: &ForjaPaths, verbose: bool) -> Result<InstallCounts> {
    let mut installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());

    let mut counts = InstallCounts {
        installed: 0,
        skipped: 0,
        failed: 0,
    };

    for skill in &registry.skills {
        if installed_ids.contains(&skill.id) {
            counts.skipped += 1;
            continue;
        }

        match manager.install(skill) {
            Ok(_) => {
                installed_ids.push(skill.id.clone());
                counts.installed += 1;
                if verbose {
                    println!("  {} {}", "✓".green(), skill.name);
                }
            }
            Err(e) => {
                counts.failed += 1;
                if verbose {
                    eprintln!("  {} {} — {}", "✗".red(), skill.name, e);
                }
            }
        }
    }

    save_installed_ids(&paths.state, &installed_ids)?;

    Ok(counts)
}

/// Install all available skills by creating symlinks for their agents and commands.
pub fn run_all(force_global: bool) -> Result<()> {
    let paths = resolve_paths(force_global)?;
    let counts = install_all_skills(&paths, true)?;

    println!();
    println!(
        "{} {} installed, {} already installed",
        "Done:".green().bold(),
        counts.installed,
        counts.skipped
    );

    output::print_tip(
        "Run 'forja doctor' to verify your setup, or 'forja guide' for a walkthrough",
    );

    Ok(())
}

/// Install skills filtered by workflow phases. Used by `forja init` with the wizard.
pub fn install_by_phases(
    paths: &ForjaPaths,
    phases: &[forja_core::models::phase::Phase],
) -> Result<(usize, usize)> {
    let mut installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());

    let mut installed = 0;
    let mut skipped = 0;

    for skill in &registry.skills {
        if !phases.contains(&skill.phase) {
            skipped += 1;
            continue;
        }
        if installed_ids.contains(&skill.id) {
            skipped += 1;
            continue;
        }
        match manager.install(skill) {
            Ok(_) => {
                installed_ids.push(skill.id.clone());
                installed += 1;
            }
            Err(_) => {
                skipped += 1;
            }
        }
    }

    save_installed_ids(&paths.state, &installed_ids)?;
    Ok((installed, skipped))
}

/// Install a single skill by creating symlinks for its agents and commands.
pub fn run(skill_path: &str, force_global: bool) -> Result<()> {
    let paths = resolve_paths(force_global)?;

    let mut installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;

    let skill = match registry.resolve(skill_path) {
        ResolveResult::Found(s) => s,
        ResolveResult::NotFound => {
            return Err(ForjaError::SkillNotFound(skill_path.to_string()));
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

    if installed_ids.contains(&skill.id) {
        return Err(ForjaError::AlreadyInstalled(skill.id.clone()));
    }

    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    let created = manager.install(skill)?;

    installed_ids.push(skill.id.clone());
    save_installed_ids(&paths.state, &installed_ids)?;

    println!("{} {}", "Installed:".green().bold(), skill.name.bold());
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

    println!();
    output::print_tip("Use 'forja task \"your task\"' to run a task with this skill");

    Ok(())
}
