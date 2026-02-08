use std::fs;
use std::path::Path;

use crate::error::Result;

const TEAMS_ENV_KEY: &str = "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS";

pub fn has_teams_env_var(claude_dir: &Path) -> bool {
    let settings_path = claude_dir.join("settings.json");
    fs::read_to_string(&settings_path)
        .ok()
        .map(|content| content.contains(TEAMS_ENV_KEY))
        .unwrap_or(false)
}

pub fn enable_teams_env_var(claude_dir: &Path) -> Result<()> {
    let settings_path = claude_dir.join("settings.json");

    let mut root: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        fs::create_dir_all(claude_dir)?;
        serde_json::json!({})
    };

    let env = root
        .as_object_mut()
        .expect("settings root must be an object")
        .entry("env")
        .or_insert_with(|| serde_json::json!({}));

    env.as_object_mut()
        .expect("env must be an object")
        .insert(TEAMS_ENV_KEY.to_string(), serde_json::json!("1"));

    let json = serde_json::to_string_pretty(&root)?;
    fs::write(&settings_path, json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn has_teams_env_var_missing_file() {
        let dir = TempDir::new().unwrap();
        assert!(!has_teams_env_var(dir.path()));
    }

    #[test]
    fn has_teams_env_var_present() {
        let dir = TempDir::new().unwrap();
        let content = r#"{ "env": { "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1" } }"#;
        fs::write(dir.path().join("settings.json"), content).unwrap();
        assert!(has_teams_env_var(dir.path()));
    }

    #[test]
    fn enable_creates_settings_from_scratch() {
        let dir = TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");

        enable_teams_env_var(&claude_dir).unwrap();

        assert!(has_teams_env_var(&claude_dir));

        let content = fs::read_to_string(claude_dir.join("settings.json")).unwrap();
        let val: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(val["env"]["CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS"], "1");
    }

    #[test]
    fn enable_preserves_existing_settings() {
        let dir = TempDir::new().unwrap();
        let existing = r#"{ "theme": "dark", "env": { "OTHER_VAR": "yes" } }"#;
        fs::write(dir.path().join("settings.json"), existing).unwrap();

        enable_teams_env_var(dir.path()).unwrap();

        let content = fs::read_to_string(dir.path().join("settings.json")).unwrap();
        let val: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(val["theme"], "dark");
        assert_eq!(val["env"]["OTHER_VAR"], "yes");
        assert_eq!(val["env"]["CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS"], "1");
    }
}
