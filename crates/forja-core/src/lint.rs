use std::fs;
use std::path::Path;

use crate::models::lint::{LintIssue, LintLevel, LintResult};

const MANIFEST_FILE: &str = "skill.json";
const LEGACY_MANIFEST_DIR: &str = ".claude-plugin";
const LEGACY_MANIFEST_FILE: &str = "plugin.json";

/// Lint a single skill directory and return all issues found.
pub fn lint_skill(skill_path: &Path, skill_id: &str) -> LintResult {
    let mut issues = Vec::new();

    check_manifest(skill_path, &mut issues);
    check_agents(skill_path, &mut issues);
    check_readme(skill_path, &mut issues);
    check_name_convention(skill_id, &mut issues);

    LintResult {
        skill_path: skill_path.to_path_buf(),
        skill_id: skill_id.to_string(),
        issues,
    }
}

fn check_manifest(skill_path: &Path, issues: &mut Vec<LintIssue>) {
    let modern = skill_path.join(MANIFEST_FILE);
    let legacy = skill_path.join(LEGACY_MANIFEST_DIR).join(LEGACY_MANIFEST_FILE);

    let manifest_path = if modern.exists() {
        modern
    } else if legacy.exists() {
        legacy
    } else {
        issues.push(LintIssue {
            level: LintLevel::Error,
            rule: "manifest-missing".to_string(),
            message: format!(
                "No manifest found. Expected {} or {}/{}",
                MANIFEST_FILE, LEGACY_MANIFEST_DIR, LEGACY_MANIFEST_FILE
            ),
        });
        return;
    };

    let content = match fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => {
            issues.push(LintIssue {
                level: LintLevel::Error,
                rule: "manifest-unreadable".to_string(),
                message: "Could not read manifest file".to_string(),
            });
            return;
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            issues.push(LintIssue {
                level: LintLevel::Error,
                rule: "manifest-invalid-json".to_string(),
                message: format!("Malformed JSON: {e}"),
            });
            return;
        }
    };

    if json.get("name").and_then(|v| v.as_str()).is_none_or(|s| s.is_empty()) {
        issues.push(LintIssue {
            level: LintLevel::Error,
            rule: "manifest-name-missing".to_string(),
            message: "Missing or empty 'name' field in manifest".to_string(),
        });
    }

    if json
        .get("description")
        .and_then(|v| v.as_str())
        .is_none_or(|s| s.is_empty())
    {
        issues.push(LintIssue {
            level: LintLevel::Error,
            rule: "manifest-description-missing".to_string(),
            message: "Missing or empty 'description' field in manifest".to_string(),
        });
    }
}

fn check_agents(skill_path: &Path, issues: &mut Vec<LintIssue>) {
    let agents_dir = skill_path.join("agents");
    if !agents_dir.exists() {
        return;
    }

    let entries = match fs::read_dir(&agents_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if let Some(after_start) = content.strip_prefix("---")
            && !after_start.contains("---")
        {
            issues.push(LintIssue {
                level: LintLevel::Error,
                rule: "agent-frontmatter-unclosed".to_string(),
                message: format!(
                    "Unclosed YAML frontmatter in {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
            });
        }
    }
}

fn check_readme(skill_path: &Path, issues: &mut Vec<LintIssue>) {
    if !skill_path.join("README.md").exists() {
        issues.push(LintIssue {
            level: LintLevel::Warning,
            rule: "readme-missing".to_string(),
            message: "No README.md found".to_string(),
        });
    }
}

fn check_name_convention(skill_id: &str, issues: &mut Vec<LintIssue>) {
    let name = skill_id.rsplit('/').next().unwrap_or(skill_id);

    if !is_kebab_case(name) {
        issues.push(LintIssue {
            level: LintLevel::Warning,
            rule: "name-not-kebab-case".to_string(),
            message: format!("Skill name '{}' is not kebab-case", name),
        });
    }
}

fn is_kebab_case(s: &str) -> bool {
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
    use tempfile::TempDir;

    #[test]
    fn is_kebab_case_valid() {
        assert!(is_kebab_case("feature"));
        assert!(is_kebab_case("code-review"));
        assert!(is_kebab_case("my-skill-123"));
    }

    #[test]
    fn is_kebab_case_invalid() {
        assert!(!is_kebab_case(""));
        assert!(!is_kebab_case("CamelCase"));
        assert!(!is_kebab_case("snake_case"));
        assert!(!is_kebab_case("-leading"));
        assert!(!is_kebab_case("trailing-"));
        assert!(!is_kebab_case("double--dash"));
    }

    #[test]
    fn lint_missing_manifest() {
        let dir = TempDir::new().unwrap();
        let result = lint_skill(dir.path(), "code/test/skill");
        assert!(result.has_errors());
        assert!(result.issues.iter().any(|i| i.rule == "manifest-missing"));
    }

    #[test]
    fn lint_valid_skill() {
        let dir = TempDir::new().unwrap();
        let manifest = r#"{ "name": "my-skill", "description": "A test skill" }"#;
        fs::write(dir.path().join(MANIFEST_FILE), manifest).unwrap();
        fs::create_dir_all(dir.path().join("agents")).unwrap();
        fs::write(dir.path().join("README.md"), "# My Skill").unwrap();

        let result = lint_skill(dir.path(), "code/test/my-skill");
        assert!(!result.has_errors());
        assert_eq!(result.warning_count(), 0);
    }
}
