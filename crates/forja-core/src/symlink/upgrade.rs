use crate::error::Result;
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::registry::diff::{ChangeType, compute_diff, load_previous_head};
use crate::registry::git;
use crate::symlink::manager::{SymlinkManager, load_installed_ids};

/// Find installed skills that have been modified since the last update.
pub fn find_upgradable(paths: &ForjaPaths) -> Result<Vec<String>> {
    let old_head = load_previous_head(&paths.forja_root)?;
    let new_head = git::head_sha(&paths.registry)?;

    if old_head == new_head {
        return Ok(Vec::new());
    }

    let changes = compute_diff(&paths.registry, &old_head, &new_head)?;
    let installed_ids = load_installed_ids(&paths.state);

    let upgradable: Vec<String> = changes
        .into_iter()
        .filter(|c| c.change_type == ChangeType::Modified && installed_ids.contains(&c.skill_id))
        .map(|c| c.skill_id)
        .collect();

    Ok(upgradable)
}

/// Reinstall a skill by removing and re-creating its symlinks.
pub fn reinstall_skill(paths: &ForjaPaths, skill_id: &str) -> Result<()> {
    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&paths.registry, &installed_ids)?;

    let skill = registry
        .find_by_id(skill_id)
        .ok_or_else(|| crate::error::ForjaError::SkillNotFound(skill_id.to_string()))?;

    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    manager.uninstall(skill_id)?;
    manager.install(skill)?;

    Ok(())
}
