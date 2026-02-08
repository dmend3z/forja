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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper: create the minimum directory structure for a valid skill.
    /// skills/<phase>/<tech>/<name>/.claude-plugin/plugin.json
    fn create_skill(root: &Path, phase: &str, tech: &str, name: &str, description: &str) {
        let skill_dir = root.join("skills").join(phase).join(tech).join(name);
        let plugin_dir = skill_dir.join(".claude-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();

        let plugin = format!(
            r#"{{ "name": "{}", "description": "{}" }}"#,
            name, description
        );
        fs::write(plugin_dir.join("plugin.json"), plugin).unwrap();

        // Create agents/ so content_types includes Agent
        fs::create_dir_all(skill_dir.join("agents")).unwrap();
    }

    #[test]
    fn scan_valid_skill_structure() {
        let dir = TempDir::new().unwrap();
        create_skill(dir.path(), "code", "general", "feature", "Writes features");

        let registry = scan(dir.path(), &[]).unwrap();

        assert_eq!(registry.skills.len(), 1);
        let skill = &registry.skills[0];
        assert_eq!(skill.id, "code/general/feature");
        assert_eq!(skill.name, "feature");
        assert_eq!(skill.description, "Writes features");
        assert_eq!(skill.phase, Phase::Code);
        assert_eq!(skill.tech, "general");
        assert!(!skill.installed);
        assert!(skill.content_types.contains(&ContentType::Agent));
    }

    #[test]
    fn scan_marks_installed_skill() {
        let dir = TempDir::new().unwrap();
        create_skill(dir.path(), "code", "general", "feature", "Writes features");

        let installed = vec!["code/general/feature".to_string()];
        let registry = scan(dir.path(), &installed).unwrap();

        assert_eq!(registry.skills.len(), 1);
        assert!(registry.skills[0].installed);
    }

    #[test]
    fn scan_missing_plugin_json_skips_skill() {
        let dir = TempDir::new().unwrap();
        // Create dir structure but no plugin.json
        let skill_dir = dir.path().join("skills/code/general/broken");
        fs::create_dir_all(&skill_dir).unwrap();

        let registry = scan(dir.path(), &[]).unwrap();
        assert!(registry.skills.is_empty());
    }

    #[test]
    fn scan_invalid_plugin_json_skips_skill() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("skills/code/general/broken");
        let plugin_dir = skill_dir.join(".claude-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("plugin.json"), "not valid json").unwrap();

        let registry = scan(dir.path(), &[]).unwrap();
        assert!(registry.skills.is_empty());
    }

    #[test]
    fn scan_empty_registry_returns_empty() {
        let dir = TempDir::new().unwrap();
        // No skills/ directory at all
        let registry = scan(dir.path(), &[]).unwrap();
        assert!(registry.skills.is_empty());
    }

    #[test]
    fn scan_unknown_phase_is_skipped() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("skills/unknown_phase/general/thing");
        let plugin_dir = skill_dir.join(".claude-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(
            plugin_dir.join("plugin.json"),
            r#"{ "name": "thing", "description": "test" }"#,
        )
        .unwrap();

        let registry = scan(dir.path(), &[]).unwrap();
        assert!(registry.skills.is_empty());
    }

    #[test]
    fn scan_multiple_skills_across_phases() {
        let dir = TempDir::new().unwrap();
        create_skill(dir.path(), "code", "general", "feature", "Code feature");
        create_skill(dir.path(), "test", "tdd", "workflow", "TDD workflow");
        create_skill(dir.path(), "review", "quality", "reviewer", "Code reviewer");

        let registry = scan(dir.path(), &[]).unwrap();
        assert_eq!(registry.skills.len(), 3);

        let ids: Vec<&str> = registry.skills.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"code/general/feature"));
        assert!(ids.contains(&"test/tdd/workflow"));
        assert!(ids.contains(&"review/quality/reviewer"));
    }

    #[test]
    fn scan_hidden_directories_are_ignored() {
        let dir = TempDir::new().unwrap();
        create_skill(dir.path(), "code", "general", "feature", "Visible");

        // Create a hidden directory at phase level
        let hidden = dir.path().join("skills/.hidden/general/secret");
        let plugin_dir = hidden.join(".claude-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(
            plugin_dir.join("plugin.json"),
            r#"{ "name": "secret", "description": "hidden" }"#,
        )
        .unwrap();

        let registry = scan(dir.path(), &[]).unwrap();
        assert_eq!(registry.skills.len(), 1);
        assert_eq!(registry.skills[0].id, "code/general/feature");
    }

    #[test]
    fn scan_detects_content_types() {
        let dir = TempDir::new().unwrap();
        create_skill(dir.path(), "code", "general", "full", "Full skill");
        let skill_dir = dir.path().join("skills/code/general/full");
        fs::create_dir_all(skill_dir.join("skills")).unwrap();
        fs::create_dir_all(skill_dir.join("commands")).unwrap();

        let registry = scan(dir.path(), &[]).unwrap();
        let skill = &registry.skills[0];
        assert!(skill.content_types.contains(&ContentType::Agent));
        assert!(skill.content_types.contains(&ContentType::Skill));
        assert!(skill.content_types.contains(&ContentType::Command));
    }
}
