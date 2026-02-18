use crate::error::{ForjaError, Result};
use crate::models::phase::Phase;
use crate::models::plugin::PluginJson;
use crate::models::registry::Registry;
use crate::models::skill::{ContentType, Skill};
use std::fs;
use std::path::Path;

const MANIFEST_FILE: &str = "skill.json";
const LEGACY_MANIFEST_FILE: &str = "plugin.json";
const LEGACY_MANIFEST_DIR: &str = ".claude-plugin";

/// Scan the skills/ directory and build a Registry of all available skills.
/// Structure expected: skills/<phase>/<tech>/<skill-name>/{skill.json|.claude-plugin/plugin.json}
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

/// Check whether `dir` looks like the forja skill registry.
/// Requires at least 3 phase-named subdirectories under `skills/`.
pub fn is_forja_registry(dir: &Path) -> bool {
    let skills_dir = dir.join("skills");
    if !skills_dir.is_dir() {
        return false;
    }
    let phase_names: Vec<&str> = Phase::all().iter().map(|p| p.as_str()).collect();
    let matches = fs::read_dir(&skills_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            phase_names.contains(&name.as_str())
        })
        .count();
    matches >= 3
}

fn parse_skill(
    path: &Path,
    id: &str,
    phase: Phase,
    tech: &str,
    installed_ids: &[String],
) -> Option<Skill> {
    let plugin = load_manifest(path)?;

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
        keywords: plugin.keywords.unwrap_or_default(),
    })
}

/// Load skill metadata from `skill.json` (preferred) or legacy `.claude-plugin/plugin.json`.
fn load_manifest(path: &Path) -> Option<PluginJson> {
    let candidates = [
        path.join(MANIFEST_FILE),
        path.join(LEGACY_MANIFEST_DIR).join(LEGACY_MANIFEST_FILE),
    ];

    candidates.iter().find_map(|candidate| {
        let manifest = fs::read_to_string(candidate).ok()?;
        serde_json::from_str::<PluginJson>(&manifest).ok()
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
    /// skills/<phase>/<tech>/<name>/skill.json
    fn create_skill(root: &Path, phase: &str, tech: &str, name: &str, description: &str) {
        create_skill_with_keywords(root, phase, tech, name, description, &[]);
    }

    fn create_skill_with_keywords(
        root: &Path,
        phase: &str,
        tech: &str,
        name: &str,
        description: &str,
        keywords: &[&str],
    ) {
        let skill_dir = root.join("skills").join(phase).join(tech).join(name);
        fs::create_dir_all(&skill_dir).unwrap();

        let keywords_json = if keywords.is_empty() {
            String::new()
        } else {
            let kw: Vec<String> = keywords.iter().map(|k| format!("\"{k}\"")).collect();
            format!(r#", "keywords": [{}]"#, kw.join(", "))
        };

        let plugin = format!(
            r#"{{ "name": "{}", "description": "{}"{} }}"#,
            name, description, keywords_json
        );
        fs::write(skill_dir.join(MANIFEST_FILE), plugin).unwrap();

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
    fn scan_missing_manifest_skips_skill() {
        let dir = TempDir::new().unwrap();
        // Create dir structure but no manifest
        let skill_dir = dir.path().join("skills/code/general/broken");
        fs::create_dir_all(&skill_dir).unwrap();

        let registry = scan(dir.path(), &[]).unwrap();
        assert!(registry.skills.is_empty());
    }

    #[test]
    fn scan_invalid_manifest_skips_skill() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("skills/code/general/broken");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join(MANIFEST_FILE), "not valid json").unwrap();

        let registry = scan(dir.path(), &[]).unwrap();
        assert!(registry.skills.is_empty());
    }

    #[test]
    fn scan_legacy_plugin_json_is_supported() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("skills/code/general/legacy");
        let plugin_dir = skill_dir.join(LEGACY_MANIFEST_DIR);
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(
            plugin_dir.join(LEGACY_MANIFEST_FILE),
            r#"{ "name": "legacy", "description": "Legacy manifest" }"#,
        )
        .unwrap();
        fs::create_dir_all(skill_dir.join("agents")).unwrap();

        let registry = scan(dir.path(), &[]).unwrap();
        assert_eq!(registry.skills.len(), 1);
        assert_eq!(registry.skills[0].id, "code/general/legacy");
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
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join(MANIFEST_FILE),
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
        fs::create_dir_all(&hidden).unwrap();
        fs::write(
            hidden.join(MANIFEST_FILE),
            r#"{ "name": "secret", "description": "hidden" }"#,
        )
        .unwrap();

        let registry = scan(dir.path(), &[]).unwrap();
        assert_eq!(registry.skills.len(), 1);
        assert_eq!(registry.skills[0].id, "code/general/feature");
    }

    #[test]
    fn is_forja_registry_with_valid_phases() {
        let dir = TempDir::new().unwrap();
        let skills = dir.path().join("skills");
        fs::create_dir_all(skills.join("code")).unwrap();
        fs::create_dir_all(skills.join("test")).unwrap();
        fs::create_dir_all(skills.join("review")).unwrap();

        assert!(is_forja_registry(dir.path()));
    }

    #[test]
    fn is_forja_registry_unrelated_skills_dir() {
        let dir = TempDir::new().unwrap();
        let skills = dir.path().join("skills");
        fs::create_dir_all(skills.join("auth")).unwrap();
        fs::create_dir_all(skills.join("users")).unwrap();
        fs::create_dir_all(skills.join("payments")).unwrap();

        assert!(!is_forja_registry(dir.path()));
    }

    #[test]
    fn is_forja_registry_no_skills_dir() {
        let dir = TempDir::new().unwrap();
        assert!(!is_forja_registry(dir.path()));
    }

    #[test]
    fn is_forja_registry_fewer_than_three_phases() {
        let dir = TempDir::new().unwrap();
        let skills = dir.path().join("skills");
        fs::create_dir_all(skills.join("code")).unwrap();
        fs::create_dir_all(skills.join("test")).unwrap();

        assert!(!is_forja_registry(dir.path()));
    }

    #[test]
    fn scan_reads_keywords_from_manifest() {
        let dir = TempDir::new().unwrap();
        create_skill_with_keywords(
            dir.path(),
            "code",
            "rust",
            "feature",
            "Rust feature coder",
            &["ownership", "borrow-checker", "traits"],
        );

        let registry = scan(dir.path(), &[]).unwrap();
        assert_eq!(registry.skills.len(), 1);
        assert_eq!(
            registry.skills[0].keywords,
            vec!["ownership", "borrow-checker", "traits"]
        );
    }

    #[test]
    fn scan_empty_keywords_when_absent() {
        let dir = TempDir::new().unwrap();
        create_skill(dir.path(), "code", "general", "feature", "Writes features");

        let registry = scan(dir.path(), &[]).unwrap();
        assert!(registry.skills[0].keywords.is_empty());
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
