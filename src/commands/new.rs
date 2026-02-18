use std::fs;

use colored::Colorize;
use dialoguer::{Input, Select};

use crate::error::{ForjaError, Result};
use crate::models::phase::Phase;
use crate::paths::ForjaPaths;
use crate::templates;

/// Scaffold a new skill with the required directory structure and templates.
pub fn run(
    name: Option<&str>,
    phase: Option<&str>,
    tech: Option<&str>,
    no_wizard: bool,
) -> Result<()> {
    let paths = ForjaPaths::ensure_initialized()?;

    println!("{}", "forja new".bold());
    println!();

    let (selected_phase, selected_tech, skill_name) = if no_wizard {
        let p = phase
            .ok_or_else(|| ForjaError::Dialoguer("--phase is required with --no-wizard".into()))?;
        let t = tech
            .ok_or_else(|| ForjaError::Dialoguer("--tech is required with --no-wizard".into()))?;
        let n = name
            .ok_or_else(|| ForjaError::Dialoguer("name is required with --no-wizard".into()))?;
        (
            p.parse::<Phase>()
                .map_err(ForjaError::Dialoguer)?,
            t.to_string(),
            n.to_string(),
        )
    } else {
        let p = match phase {
            Some(s) => s
                .parse::<Phase>()
                .map_err(ForjaError::Dialoguer)?,
            None => select_phase()?,
        };

        let t = match tech {
            Some(s) => s.to_string(),
            None => prompt_tech(&paths, p)?,
        };

        let n = match name {
            Some(s) => s.to_string(),
            None => prompt_name()?,
        };

        (p, t, n)
    };

    // Validate name
    if !is_valid_kebab_case(&skill_name) {
        return Err(ForjaError::InvalidSkillName(skill_name));
    }

    // Build path
    let skill_dir = paths
        .registry
        .join("skills")
        .join(selected_phase.as_str())
        .join(&selected_tech)
        .join(&skill_name);

    if skill_dir.exists() {
        return Err(ForjaError::AlreadyInstalled(format!(
            "{}/{}/{}",
            selected_phase, selected_tech, skill_name
        )));
    }

    // Create directory structure
    let agents_dir = skill_dir.join("agents");
    fs::create_dir_all(&agents_dir)?;

    // Write templates
    let description = format!("TODO: describe {}", skill_name);
    fs::write(
        skill_dir.join("skill.json"),
        templates::skill_json(&skill_name, &description),
    )?;
    fs::write(
        agents_dir.join(format!("{}.md", skill_name)),
        templates::agent_md(&skill_name, selected_phase),
    )?;
    fs::write(
        skill_dir.join("README.md"),
        templates::readme_md(&skill_name, selected_phase, &selected_tech),
    )?;

    println!(
        "  {} Created skill: {}/{}/{}",
        "✓".green(),
        selected_phase.as_str().cyan(),
        selected_tech.cyan(),
        skill_name.cyan()
    );
    println!();
    println!("  Files created:");
    println!("    {}/skill.json", skill_dir.display());
    println!("    {}/agents/{}.md", skill_dir.display(), skill_name);
    println!("    {}/README.md", skill_dir.display());
    println!();
    println!(
        "  {} Edit the TODO placeholders, then run: {}",
        "Next:".green().bold(),
        format!(
            "forja lint {}",
            skill_dir.display()
        )
        .cyan()
    );

    Ok(())
}

fn select_phase() -> Result<Phase> {
    let phases: Vec<&Phase> = Phase::all()
        .iter()
        .filter(|p| **p != Phase::Teams)
        .collect();
    let labels: Vec<String> = phases
        .iter()
        .map(|p| format!("{} — {}", p.as_str(), p.description()))
        .collect();

    let selection = Select::new()
        .with_prompt("Select workflow phase")
        .items(&labels)
        .default(1) // Default to Code
        .interact()
        .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

    Ok(*phases[selection])
}

fn prompt_tech(paths: &ForjaPaths, phase: Phase) -> Result<String> {
    // Suggest existing tech categories from the registry
    let phase_dir = paths
        .registry
        .join("skills")
        .join(phase.as_str());

    let mut existing: Vec<String> = if phase_dir.exists() {
        fs::read_dir(&phase_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| !n.starts_with('.'))
            .collect()
    } else {
        Vec::new()
    };
    existing.sort();

    let prompt_msg = if existing.is_empty() {
        "Tech category (e.g. rust, nextjs, general)".to_string()
    } else {
        format!(
            "Tech category (existing: {})",
            existing.join(", ")
        )
    };

    let tech: String = Input::new()
        .with_prompt(&prompt_msg)
        .interact_text()
        .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

    Ok(tech.trim().to_lowercase())
}

fn prompt_name() -> Result<String> {
    let name: String = Input::new()
        .with_prompt("Skill name (kebab-case)")
        .interact_text()
        .map_err(|e| ForjaError::Dialoguer(e.to_string()))?;

    Ok(name.trim().to_string())
}

fn is_valid_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_kebab_names() {
        assert!(is_valid_kebab_case("feature"));
        assert!(is_valid_kebab_case("code-review"));
        assert!(is_valid_kebab_case("my-skill-v2"));
    }

    #[test]
    fn invalid_kebab_names() {
        assert!(!is_valid_kebab_case(""));
        assert!(!is_valid_kebab_case("CamelCase"));
        assert!(!is_valid_kebab_case("snake_case"));
        assert!(!is_valid_kebab_case("-leading"));
        assert!(!is_valid_kebab_case("trailing-"));
        assert!(!is_valid_kebab_case("double--dash"));
        assert!(!is_valid_kebab_case("has space"));
    }
}
