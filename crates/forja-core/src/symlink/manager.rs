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

        // Symlink agents/*.md -> ~/.claude/agents/
        let agents_dir = skill.path.join("agents");
        if agents_dir.exists() {
            fs::create_dir_all(&self.claude_agents_dir)?;
            created.extend(self.symlink_dir(&agents_dir, &self.claude_agents_dir, &skill.id)?);
        }

        // Symlink commands/*.md -> ~/.claude/commands/
        // (skip Teams-phase skills -- their commands are managed by `forja team`)
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

    /// Remove all `forja--` prefixed symlinks from both agents/ and commands/.
    #[cfg(test)]
    pub fn remove_all_forja_symlinks(&self) -> Result<Vec<PathBuf>> {
        let mut removed = Vec::new();
        removed.extend(self.remove_matching_symlinks(&self.claude_agents_dir, SYMLINK_PREFIX)?);
        removed.extend(self.remove_matching_symlinks(&self.claude_commands_dir, SYMLINK_PREFIX)?);
        Ok(removed)
    }

    /// Remove `forja--` symlinks whose target is under `registry_path`.
    /// Symlinks belonging to other projects are preserved.
    pub fn remove_project_symlinks(&self, registry_path: &Path) -> Result<Vec<PathBuf>> {
        let mut removed = Vec::new();
        removed.extend(self.remove_symlinks_under(&self.claude_agents_dir, registry_path)?);
        removed.extend(self.remove_symlinks_under(&self.claude_commands_dir, registry_path)?);
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

    fn remove_symlinks_under(&self, dir: &Path, registry_path: &Path) -> Result<Vec<PathBuf>> {
        let mut removed = Vec::new();
        if !dir.exists() {
            return Ok(removed);
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(SYMLINK_PREFIX)
                && entry.path().is_symlink()
                && let Ok(target) = fs::read_link(entry.path())
                && target.starts_with(registry_path)
            {
                fs::remove_file(entry.path())?;
                removed.push(entry.path());
            }
        }
        Ok(removed)
    }

    fn remove_matching_symlinks(&self, dir: &Path, prefix: &str) -> Result<Vec<PathBuf>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::phase::Phase;
    use crate::models::skill::{ContentType, Skill};
    use std::os::unix::fs as unix_fs;
    use tempfile::TempDir;

    fn make_skill(dir: &TempDir, id: &str, phase: Phase) -> Skill {
        let skill_path = dir.path().join(id);
        let agents_dir = skill_path.join("agents");
        fs::create_dir_all(&agents_dir).unwrap();
        fs::write(agents_dir.join("coder.md"), "# Agent").unwrap();

        Skill {
            id: id.to_string(),
            name: id.split('/').last().unwrap().to_string(),
            description: "test skill".to_string(),
            phase,
            tech: "general".to_string(),
            path: skill_path,
            installed: false,
            content_types: vec![ContentType::Agent],
            keywords: vec![],
        }
    }

    #[test]
    fn install_creates_agent_symlinks() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let agents_dir = target.path().join("agents");
        let commands_dir = target.path().join("commands");

        let skill = make_skill(&source, "code/general/feature", Phase::Code);
        let manager = SymlinkManager::new(agents_dir.clone(), commands_dir);

        let created = manager.install(&skill).unwrap();

        assert_eq!(created.len(), 1);
        assert!(created[0].is_symlink());
        let name = created[0].file_name().unwrap().to_string_lossy().to_string();
        assert!(name.starts_with("forja--"));
        assert!(name.contains("code--general--feature"));
    }

    #[test]
    fn uninstall_removes_matching_symlinks() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let agents_dir = target.path().join("agents");
        let commands_dir = target.path().join("commands");

        let skill = make_skill(&source, "code/general/feature", Phase::Code);
        let manager = SymlinkManager::new(agents_dir.clone(), commands_dir.clone());

        manager.install(&skill).unwrap();
        assert_eq!(fs::read_dir(&agents_dir).unwrap().count(), 1);

        let removed = manager.uninstall("code/general/feature").unwrap();
        assert_eq!(removed.len(), 1);
        assert_eq!(fs::read_dir(&agents_dir).unwrap().count(), 0);
    }

    #[test]
    fn verify_detects_healthy_symlinks() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let agents_dir = target.path().join("agents");
        let commands_dir = target.path().join("commands");

        let skill = make_skill(&source, "code/general/feature", Phase::Code);
        let manager = SymlinkManager::new(agents_dir, commands_dir);
        manager.install(&skill).unwrap();

        let (healthy, broken) = manager.verify().unwrap();
        assert_eq!(healthy.len(), 1);
        assert!(broken.is_empty());
    }

    #[test]
    fn verify_detects_broken_symlinks() {
        let target = TempDir::new().unwrap();
        let agents_dir = target.path().join("agents");
        let commands_dir = target.path().join("commands");
        fs::create_dir_all(&agents_dir).unwrap();

        let broken_link = agents_dir.join("forja--code--general--gone--coder.md");
        unix_fs::symlink("/tmp/nonexistent_forja_test_target", &broken_link).unwrap();

        let manager = SymlinkManager::new(agents_dir, commands_dir);
        let (healthy, broken) = manager.verify().unwrap();
        assert!(healthy.is_empty());
        assert_eq!(broken.len(), 1);
    }
}
