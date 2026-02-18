use crate::error::Result;
use crate::paths::ForjaMode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

fn default_version() -> u32 {
    2
}
fn default_mode() -> ForjaMode {
    ForjaMode::Global
}
fn default_registry_url() -> String {
    "https://github.com/dmend3z/forja.git".to_string()
}

/// Persisted forja configuration (`.forja/config.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForjaConfig {
    #[serde(default = "default_version")]
    pub version: u32,

    #[serde(default = "default_mode")]
    pub mode: ForjaMode,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,

    #[serde(default = "default_registry_url")]
    pub registry_url: String,

    #[serde(default)]
    pub local: bool,
}

impl ForjaConfig {
    pub fn new(mode: ForjaMode, registry_url: String, local: bool) -> Self {
        let project_name = if mode == ForjaMode::Project {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        } else {
            None
        };

        Self {
            version: 2,
            mode,
            project_name,
            registry_url,
            local,
        }
    }
}

pub fn load_config(config_path: &Path) -> Option<ForjaConfig> {
    let content = fs::read_to_string(config_path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_config(config_path: &Path, config: &ForjaConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(config_path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");

        let config = ForjaConfig::new(
            ForjaMode::Global,
            "https://github.com/dmend3z/forja.git".to_string(),
            false,
        );
        save_config(&path, &config).unwrap();

        let loaded = load_config(&path).unwrap();
        assert_eq!(loaded.version, 2);
        assert_eq!(loaded.mode, ForjaMode::Global);
        assert!(!loaded.local);
    }

    #[test]
    fn backward_compat_old_format() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");

        // Old format: just registry_url + local, no version/mode
        fs::write(&path, r#"{"registry_url":"https://x.git","local":true}"#).unwrap();

        let loaded = load_config(&path).unwrap();
        assert_eq!(loaded.version, 2); // default
        assert_eq!(loaded.mode, ForjaMode::Global); // default
        assert!(loaded.local);
    }

    #[test]
    fn missing_config_returns_none() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("does-not-exist.json");
        assert!(load_config(&path).is_none());
    }
}
