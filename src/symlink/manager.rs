use crate::error::Result;
use crate::models::skill::Skill;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

const SYMLINK_PREFIX: &str = "forja--";

pub struct SymlinkManager {
    claude_agents_dir: PathBuf,
    claude_commands_dir: PathBuf,
}

impl SymlinkManager {
    pub fn new(claude_agents_dir: PathBuf, claude_commands_dir: PathBuf) -> Self {
        Self {
            claude_agents_dir,
            claude_commands_dir,
        }
    }

    /// Install a skill: symlink agents/*.md and commands/*.md into ~/.claude/
    pub fn install(&self, skill: &Skill) -> Result<Vec<PathBuf>> {
        let mut created = Vec::new();

        // Symlink agents/*.md → ~/.claude/agents/
        let agents_dir = skill.path.join("agents");
        if agents_dir.exists() {
            fs::create_dir_all(&self.claude_agents_dir)?;
            created.extend(self.symlink_dir(&agents_dir, &self.claude_agents_dir, &skill.id)?);
        }

        // Symlink commands/*.md → ~/.claude/commands/
        // (skip Teams-phase skills — their commands are managed by `forja team`)
        if skill.phase != crate::models::phase::Phase::Teams {
            let commands_dir = skill.path.join("commands");
            if commands_dir.exists() {
                fs::create_dir_all(&self.claude_commands_dir)?;
                created.extend(self.symlink_dir(
                    &commands_dir,
                    &self.claude_commands_dir,
                    &skill.id,
                )?);
            }
        }

        Ok(created)
    }

    /// Uninstall: remove all symlinks for a skill (agents + commands)
    pub fn uninstall(&self, skill_id: &str) -> Result<Vec<PathBuf>> {
        let prefix = format!("{SYMLINK_PREFIX}{}--", skill_id.replace('/', "--"));
        let mut removed = Vec::new();

        removed.extend(self.remove_matching_symlinks(&self.claude_agents_dir, &prefix)?);
        removed.extend(self.remove_matching_symlinks(&self.claude_commands_dir, &prefix)?);

        Ok(removed)
    }

    /// Verify all forja symlinks in both agents/ and commands/
    pub fn verify(&self) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        let mut healthy = Vec::new();
        let mut broken = Vec::new();

        for dir in [&self.claude_agents_dir, &self.claude_commands_dir] {
            let (h, b) = self.verify_dir(dir)?;
            healthy.extend(h);
            broken.extend(b);
        }

        Ok((healthy, broken))
    }

    fn symlink_dir(
        &self,
        source_dir: &Path,
        target_dir: &Path,
        skill_id: &str,
    ) -> Result<Vec<PathBuf>> {
        let mut created = Vec::new();

        for entry in fs::read_dir(source_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                let file_name = path.file_name().unwrap().to_string_lossy();
                let link_name = format!(
                    "{SYMLINK_PREFIX}{}--{file_name}",
                    skill_id.replace('/', "--")
                );
                let link_path = target_dir.join(&link_name);

                if link_path.exists() || link_path.is_symlink() {
                    fs::remove_file(&link_path)?;
                }

                unix_fs::symlink(&path, &link_path)?;
                created.push(link_path);
            }
        }

        Ok(created)
    }

    fn remove_matching_symlinks(
        &self,
        dir: &Path,
        prefix: &str,
    ) -> Result<Vec<PathBuf>> {
        let mut removed = Vec::new();

        if !dir.exists() {
            return Ok(removed);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(prefix) && entry.path().is_symlink() {
                fs::remove_file(entry.path())?;
                removed.push(entry.path());
            }
        }

        Ok(removed)
    }

    fn verify_dir(&self, dir: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        let mut healthy = Vec::new();
        let mut broken = Vec::new();

        if !dir.exists() {
            return Ok((healthy, broken));
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(SYMLINK_PREFIX) && entry.path().is_symlink() {
                let target = fs::read_link(entry.path())?;
                if target.exists() {
                    healthy.push(entry.path());
                } else {
                    broken.push(entry.path());
                }
            }
        }

        Ok((healthy, broken))
    }
}

/// Load installed skill IDs from state.json (backward-compatible wrapper)
pub fn load_installed_ids(state_path: &Path) -> Vec<String> {
    use crate::models::state::load_state;
    load_state(state_path).installed
}

/// Save installed skill IDs to state.json (preserves teams and other state)
pub fn save_installed_ids(state_path: &Path, ids: &[String]) -> Result<()> {
    use crate::models::state::{load_state, save_state};
    let mut state = load_state(state_path);
    state.installed = ids.to_vec();
    save_state(state_path, &state)
}
