use crate::error::{ForjaError, Result};
use crate::models::phase::Phase;
use crate::models::plugin::PluginJson;
use crate::models::registry::Registry;
use crate::models::skill::{ContentType, Skill};
use std::fs;
use std::path::Path;

/// Scan the skills/ directory and build a Registry of all available skills.
/// Structure expected: skills/<phase>/<tech>/<skill-name>/.claude-plugin/plugin.json
pub fn scan(registry_path: &Path, installed_ids: &[String]) -> Result<Registry> {
    let skills_dir = registry_path.join("skills");
    if !skills_dir.exists() {
        return Ok(Registry::new(vec![]));
    }

    let mut skills = Vec::new();

    // Level 1: phases (research, code, test, review, deploy, teams)
    for phase_entry in read_dirs(&skills_dir)? {
        let phase_name = file_name(&phase_entry);
        let Ok(phase) = phase_name.parse::<Phase>() else {
            continue;
        };

        // Level 2: tech categories (nextjs, tdd, codebase, etc.)
        for tech_entry in read_dirs(&phase_entry)? {
            let tech = file_name(&tech_entry);

            // Level 3: individual skills
            for skill_entry in read_dirs(&tech_entry)? {
                let skill_name = file_name(&skill_entry);
                let id = format!("{phase_name}/{tech}/{skill_name}");

                if let Some(skill) = parse_skill(&skill_entry, &id, phase, &tech, installed_ids) {
                    skills.push(skill);
                }
            }
        }
    }

    Ok(Registry::new(skills))
}

fn parse_skill(
    path: &Path,
    id: &str,
    phase: Phase,
    tech: &str,
    installed_ids: &[String],
) -> Option<Skill> {
    let plugin_json_path = path.join(".claude-plugin").join("plugin.json");
    let plugin_json_str = fs::read_to_string(&plugin_json_path).ok()?;
    let plugin: PluginJson = serde_json::from_str(&plugin_json_str).ok()?;

    let mut content_types = Vec::new();
    if path.join("agents").exists() {
        content_types.push(ContentType::Agent);
    }
    if path.join("skills").exists() {
        content_types.push(ContentType::Skill);
    }
    if path.join("commands").exists() {
        content_types.push(ContentType::Command);
    }

    Some(Skill {
        id: id.to_string(),
        name: plugin.name,
        description: plugin.description,
        phase,
        tech: tech.to_string(),
        path: path.to_path_buf(),
        installed: installed_ids.contains(&id.to_string()),
        content_types,
    })
}

/// Read directory entries, filtering to directories only, sorted by name.
fn read_dirs(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(ForjaError::Io)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && !file_name(p).starts_with('.'))
        .collect();
    entries.sort();
    Ok(entries)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}
