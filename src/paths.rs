use crate::error::{ForjaError, Result};
use std::path::PathBuf;

pub struct ForjaPaths {
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
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or(ForjaError::NoHomeDir)?;

        let forja_root = home.join(".forja");
        let claude_dir = home.join(".claude");

        Ok(Self {
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

    pub fn ensure_initialized() -> Result<Self> {
        let paths = Self::new()?;
        if !paths.forja_root.exists() {
            return Err(ForjaError::NotInitialized);
        }
        Ok(paths)
    }
}
