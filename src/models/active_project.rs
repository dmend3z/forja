use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Tracks which project currently owns the `~/.claude/agents/` symlinks.
/// Stored at `~/.forja/active_project.json` (always global).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveProject {
    pub project_name: String,
    pub project_root: PathBuf,
    pub synced_at: u64,
}

impl ActiveProject {
    pub fn new(project_name: String, project_root: PathBuf) -> Self {
        let synced_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            project_name,
            project_root,
            synced_at,
        }
    }
}

pub fn load_active_project(path: &Path) -> Option<ActiveProject> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_active_project(path: &Path, active: &ActiveProject) -> Result<()> {
    let json = serde_json::to_string_pretty(active)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn clear_active_project(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn save_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("active_project.json");

        let active = ActiveProject::new("my-app".to_string(), PathBuf::from("/tmp/my-app"));
        save_active_project(&path, &active).unwrap();

        let loaded = load_active_project(&path).unwrap();
        assert_eq!(loaded.project_name, "my-app");
        assert_eq!(loaded.project_root, PathBuf::from("/tmp/my-app"));
        assert!(loaded.synced_at > 0);
    }

    #[test]
    fn clear_removes_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("active_project.json");

        let active = ActiveProject::new("test".to_string(), PathBuf::from("/tmp/test"));
        save_active_project(&path, &active).unwrap();
        assert!(path.exists());

        clear_active_project(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn clear_nonexistent_is_ok() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nope.json");
        clear_active_project(&path).unwrap();
    }

    #[test]
    fn load_missing_returns_none() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("missing.json");
        assert!(load_active_project(&path).is_none());
    }
}
