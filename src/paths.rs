use crate::error::{ForjaError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Whether forja operates globally (`~/.forja/`) or per-project (`.forja/`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForjaMode {
    Global,
    Project,
}

/// Canonical paths used by forja.
///
/// In **global** mode, `forja_root` points to `~/.forja/`.
/// In **project** mode, `forja_root` points to `<project>/.forja/`.
/// In **project** mode, `claude_dir` points to `<project>/.claude/` (project-local agents).
/// In **global** mode, `claude_dir` points to `~/.claude/`.
pub struct ForjaPaths {
    pub mode: ForjaMode,
    pub project_root: Option<PathBuf>,
    pub forja_root: PathBuf,
    pub registry: PathBuf,
    pub config: PathBuf,
    pub state: PathBuf,
    pub plans: PathBuf,
    pub claude_dir: PathBuf,
    pub claude_agents: PathBuf,
    pub claude_commands: PathBuf,
}

impl ForjaPaths {
    /// Auto-detect: walk up from cwd looking for `.forja/config.json`, fallback to global.
    pub fn resolve() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        if let Some(project_root) = detect_project_root(&cwd) {
            return Self::from_project(project_root);
        }
        Self::global()
    }

    /// Force global mode (`~/.forja/`).
    pub fn global() -> Result<Self> {
        let home = dirs::home_dir().ok_or(ForjaError::NoHomeDir)?;
        let forja_root = home.join(".forja");
        let claude_dir = home.join(".claude");

        Ok(Self {
            mode: ForjaMode::Global,
            project_root: None,
            registry: forja_root.join("registry"),
            config: forja_root.join("config.json"),
            state: forja_root.join("state.json"),
            plans: forja_root.join("plans"),
            forja_root,
            claude_agents: claude_dir.join("agents"),
            claude_commands: claude_dir.join("commands"),
            claude_dir,
        })
    }

    /// Force project mode (`.forja/` inside given project root).
    pub fn from_project(project_root: PathBuf) -> Result<Self> {
        let forja_root = project_root.join(".forja");
        let claude_dir = project_root.join(".claude");

        Ok(Self {
            mode: ForjaMode::Project,
            project_root: Some(project_root),
            registry: forja_root.join("registry"),
            config: forja_root.join("config.json"),
            state: forja_root.join("state.json"),
            plans: forja_root.join("plans"),
            forja_root,
            claude_agents: claude_dir.join("agents"),
            claude_commands: claude_dir.join("commands"),
            claude_dir,
        })
    }

    /// Backward-compatible alias: resolves project-first, then global.
    pub fn new() -> Result<Self> {
        Self::resolve()
    }

    /// Resolve + check that forja_root exists.
    pub fn ensure_initialized() -> Result<Self> {
        let paths = Self::resolve()?;
        if !paths.forja_root.exists() {
            return Err(ForjaError::NotInitialized);
        }
        Ok(paths)
    }

    /// Human-readable name for the current context.
    pub fn display_name(&self) -> String {
        match &self.project_root {
            Some(root) => root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "project".to_string()),
            None => "global".to_string(),
        }
    }

    /// Always returns `~/.forja/` regardless of current mode.
    pub fn global_forja_root() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or(ForjaError::NoHomeDir)?;
        Ok(home.join(".forja"))
    }

    /// Always returns `~/.claude/` regardless of current mode.
    /// Used for global settings like `settings.json` which must always live in `~/.claude/`.
    pub fn global_claude_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or(ForjaError::NoHomeDir)?;
        Ok(home.join(".claude"))
    }
}

/// Walk up from `start` looking for `.forja/config.json`.
pub fn detect_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".forja").join("config.json").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detect_project_root_finds_config() {
        let dir = TempDir::new().unwrap();
        let forja_dir = dir.path().join(".forja");
        fs::create_dir_all(&forja_dir).unwrap();
        fs::write(forja_dir.join("config.json"), "{}").unwrap();

        let result = detect_project_root(dir.path());
        assert_eq!(result, Some(dir.path().to_path_buf()));
    }

    #[test]
    fn detect_project_root_walks_up() {
        let dir = TempDir::new().unwrap();
        let forja_dir = dir.path().join(".forja");
        fs::create_dir_all(&forja_dir).unwrap();
        fs::write(forja_dir.join("config.json"), "{}").unwrap();

        let subdir = dir.path().join("src").join("deep");
        fs::create_dir_all(&subdir).unwrap();

        let result = detect_project_root(&subdir);
        assert_eq!(result, Some(dir.path().to_path_buf()));
    }

    #[test]
    fn detect_project_root_returns_none_without_config() {
        let dir = TempDir::new().unwrap();
        let result = detect_project_root(dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn from_project_sets_mode_and_root() {
        let dir = TempDir::new().unwrap();
        let paths = ForjaPaths::from_project(dir.path().to_path_buf()).unwrap();
        assert_eq!(paths.mode, ForjaMode::Project);
        assert_eq!(paths.project_root.unwrap(), dir.path().to_path_buf());
        assert_eq!(paths.forja_root, dir.path().join(".forja"));
    }

    #[test]
    fn global_sets_mode_no_project_root() {
        let paths = ForjaPaths::global().unwrap();
        assert_eq!(paths.mode, ForjaMode::Global);
        assert!(paths.project_root.is_none());
    }

    #[test]
    fn display_name_for_project() {
        let dir = TempDir::new().unwrap();
        let paths = ForjaPaths::from_project(dir.path().to_path_buf()).unwrap();
        let name = paths.display_name();
        assert!(!name.is_empty());
    }

    #[test]
    fn display_name_for_global() {
        let paths = ForjaPaths::global().unwrap();
        assert_eq!(paths.display_name(), "global");
    }

    #[test]
    fn project_mode_claude_dir_is_local() {
        let dir = TempDir::new().unwrap();
        let project = ForjaPaths::from_project(dir.path().to_path_buf()).unwrap();

        assert_eq!(project.claude_dir, dir.path().join(".claude"));
        assert_eq!(project.claude_agents, dir.path().join(".claude").join("agents"));
        assert_eq!(project.claude_commands, dir.path().join(".claude").join("commands"));
    }

    #[test]
    fn global_mode_claude_dir_is_home() {
        let global = ForjaPaths::global().unwrap();
        let home = dirs::home_dir().unwrap();

        assert_eq!(global.claude_dir, home.join(".claude"));
        assert_eq!(global.claude_agents, home.join(".claude").join("agents"));
        assert_eq!(global.claude_commands, home.join(".claude").join("commands"));
    }

    #[test]
    fn global_claude_dir_always_home() {
        let dir = ForjaPaths::global_claude_dir().unwrap();
        let home = dirs::home_dir().unwrap();
        assert_eq!(dir, home.join(".claude"));
    }
}
