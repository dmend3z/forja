use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::Result;

/// Persistent state stored in `~/.forja/state.json`. Tracks installed skills and team configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForjaState {
    pub version: u32,
    pub installed: Vec<String>,
    #[serde(default)]
    pub teams: HashMap<String, TeamEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_profile: Option<String>,
}

/// A configured multi-agent team with its member list and model profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamEntry {
    pub members: Vec<TeamMember>,
    pub profile: String,
}

/// A single agent in a team, linking a skill to an agent name and model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub skill_id: String,
    pub agent_name: String,
    pub model: String,
}

impl Default for ForjaState {
    fn default() -> Self {
        Self {
            version: 2,
            installed: Vec::new(),
            teams: HashMap::new(),
            active_profile: None,
        }
    }
}

impl ForjaState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn load_state(state_path: &Path) -> ForjaState {
    let Ok(content) = fs::read_to_string(state_path) else {
        return ForjaState::new();
    };

    // Try new format first
    if let Ok(state) = serde_json::from_str::<ForjaState>(&content) {
        return state;
    }

    // Fallback: old Vec<String> format
    if let Ok(ids) = serde_json::from_str::<Vec<String>>(&content) {
        return ForjaState {
            version: 2,
            installed: ids,
            teams: HashMap::new(),
            active_profile: None,
        };
    }

    ForjaState::new()
}

pub fn save_state(state_path: &Path, state: &ForjaState) -> Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    fs::write(state_path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn load_empty_state_returns_default() {
        let path = std::path::PathBuf::from("/tmp/forja_test_nonexistent_state.json");
        let state = load_state(&path);
        assert_eq!(state.version, 2);
        assert!(state.installed.is_empty());
        assert!(state.teams.is_empty());
    }

    #[test]
    fn load_old_vec_format_migrates() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, r#"["code/general/feature","test/tdd/workflow"]"#).unwrap();

        let state = load_state(tmp.path());
        assert_eq!(state.version, 2);
        assert_eq!(state.installed.len(), 2);
        assert_eq!(state.installed[0], "code/general/feature");
        assert!(state.teams.is_empty());
    }

    #[test]
    fn load_new_format_roundtrips() {
        let mut state = ForjaState::new();
        state.installed = vec!["code/general/feature".to_string()];
        state.teams.insert(
            "my-team".to_string(),
            TeamEntry {
                members: vec![TeamMember {
                    skill_id: "code/general/feature".to_string(),
                    agent_name: "coder".to_string(),
                    model: "sonnet".to_string(),
                }],
                profile: "fast".to_string(),
            },
        );

        let tmp = NamedTempFile::new().unwrap();
        save_state(tmp.path(), &state).unwrap();

        let loaded = load_state(tmp.path());
        assert_eq!(loaded.version, 2);
        assert_eq!(loaded.installed, vec!["code/general/feature"]);
        assert!(loaded.teams.contains_key("my-team"));
        assert_eq!(loaded.teams["my-team"].members.len(), 1);
        assert_eq!(loaded.teams["my-team"].profile, "fast");
    }

    #[test]
    fn save_preserves_teams_when_updating_installed() {
        let mut state = ForjaState::new();
        state.teams.insert(
            "test-team".to_string(),
            TeamEntry {
                members: vec![],
                profile: "balanced".to_string(),
            },
        );

        let tmp = NamedTempFile::new().unwrap();
        save_state(tmp.path(), &state).unwrap();

        // Simulate what save_installed_ids does
        let mut reloaded = load_state(tmp.path());
        reloaded.installed = vec!["new/skill".to_string()];
        save_state(tmp.path(), &reloaded).unwrap();

        let final_state = load_state(tmp.path());
        assert_eq!(final_state.installed, vec!["new/skill"]);
        assert!(final_state.teams.contains_key("test-team"));
    }
}
