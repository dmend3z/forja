use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::error::{ForjaError, Result};

/// A skill that changed between two git revisions.
#[derive(Debug, Clone)]
pub struct SkillChange {
    pub skill_id: String,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
}

/// Load the pre-update HEAD SHA from last_update.json.
pub fn load_previous_head(forja_root: &Path) -> Result<String> {
    let path = forja_root.join("last_update.json");
    if !path.exists() {
        return Err(ForjaError::Git(
            "No previous update found. Run 'forja update' first.".to_string(),
        ));
    }

    let content = fs::read_to_string(&path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    json.get("head_before")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| ForjaError::Git("Invalid last_update.json format".to_string()))
}

/// Compute skill-level changes between two git revisions.
pub fn compute_diff(registry_path: &Path, old_head: &str, new_head: &str) -> Result<Vec<SkillChange>> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(registry_path)
        .args(["diff", "--name-status", &format!("{old_head}..{new_head}"), "--", "skills/"])
        .output()
        .map_err(ForjaError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ForjaError::Git(format!("git diff failed: {stderr}")));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_diff_output(&stdout)
}

fn parse_diff_output(output: &str) -> Result<Vec<SkillChange>> {
    let mut skill_changes: HashMap<String, ChangeType> = HashMap::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let status = parts[0];
        let file_path = parts[1];

        let segments: Vec<&str> = file_path.split('/').collect();
        if segments.len() < 4 || segments[0] != "skills" {
            continue;
        }

        let skill_id = format!("{}/{}/{}", segments[1], segments[2], segments[3]);

        let change_type = match status {
            "A" => ChangeType::Added,
            "D" => ChangeType::Removed,
            _ => ChangeType::Modified,
        };

        skill_changes
            .entry(skill_id)
            .and_modify(|existing| {
                if change_type == ChangeType::Added && *existing != ChangeType::Added {
                    *existing = ChangeType::Added;
                }
            })
            .or_insert(change_type);
    }

    let mut changes: Vec<SkillChange> = skill_changes
        .into_iter()
        .map(|(skill_id, change_type)| SkillChange {
            skill_id,
            change_type,
        })
        .collect();

    changes.sort_by(|a, b| a.skill_id.cmp(&b.skill_id));
    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_added_file() {
        let output = "A\tskills/code/rust/coder/skill.json\n";
        let changes = parse_diff_output(output).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].skill_id, "code/rust/coder");
        assert_eq!(changes[0].change_type, ChangeType::Added);
    }

    #[test]
    fn parse_modified_file() {
        let output = "M\tskills/code/rust/coder/agents/coder.md\n";
        let changes = parse_diff_output(output).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Modified);
    }

    #[test]
    fn parse_empty_output() {
        let changes = parse_diff_output("").unwrap();
        assert!(changes.is_empty());
    }
}
