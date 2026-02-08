use std::fs;
use std::path::Path;

use colored::Colorize;

use crate::error::{ForjaError, Result};
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::symlink::manager::load_installed_ids;

/// Display detailed information about a skill (description, phase, content types, files).
pub fn run(skill_path: &str) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;
    let skill = registry
        .find_by_id(skill_path)
        .ok_or_else(|| ForjaError::SkillNotFound(skill_path.to_string()))?;

    println!("{}", skill.name.bold());
    println!("{}", skill.description);
    println!();
    println!("  Phase:     {}", skill.phase.as_str().cyan());
    println!("  Tech:      {}", skill.tech.cyan());
    println!(
        "  Installed: {}",
        if skill.installed {
            "yes".green()
        } else {
            "no".dimmed()
        }
    );
    println!("  Path:      {}", skill.path.display().to_string().dimmed());

    let types: Vec<_> = skill.content_types.iter().map(|t| t.to_string()).collect();
    println!("  Content:   {}", types.join(", "));

    list_dir_entries(&skill.path.join("agents"), "Agents", |name| {
        if name.ends_with(".md") {
            Some(name.to_string())
        } else {
            None
        }
    });
    list_dir_entries(&skill.path.join("skills"), "Skills", |name| {
        Some(format!("/{name}"))
    });
    list_dir_entries(&skill.path.join("commands"), "Commands", |name| {
        name.strip_suffix(".md").map(|s| format!("/{s}"))
    });

    Ok(())
}

fn list_dir_entries(dir: &Path, label: &str, format_name: impl Fn(&str) -> Option<String>) {
    if !dir.exists() {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    println!();
    println!("  {}:", label.bold());
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(display) = format_name(&name) {
            println!("    {}", display.cyan());
        }
    }
}
