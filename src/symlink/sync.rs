use crate::error::Result;
use crate::models::active_project::{
    clear_active_project, load_active_project, save_active_project, ActiveProject,
};
use crate::paths::{ForjaMode, ForjaPaths};
use crate::registry::catalog;
use crate::symlink::manager::{load_installed_ids, SymlinkManager};
use colored::Colorize;

/// Rebuild all `forja--` symlinks in `~/.claude/` from the current context's state.
///
/// Returns `true` if the active project switched (i.e. a different project was synced before).
pub fn sync_symlinks(paths: &ForjaPaths) -> Result<bool> {
    let global_forja = ForjaPaths::global_forja_root()?;
    let active_path = global_forja.join("active_project.json");
    let previous = load_active_project(&active_path);

    let mut switched = false;

    // Detect project switch
    if let Some(ref prev) = previous {
        let current_root = paths.project_root.as_deref();
        let prev_root = Some(prev.project_root.as_path());
        if current_root != prev_root {
            println!(
                "  {} Switching from {} to {}",
                "WARNING:".yellow().bold(),
                prev.project_name.dimmed(),
                paths.display_name().bold()
            );
            switched = true;
        }
    }

    // Remove all existing forja symlinks
    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    manager.remove_all_forja_symlinks()?;

    // Recreate symlinks from current state
    let installed_ids = load_installed_ids(&paths.state);
    if !installed_ids.is_empty() {
        if let Ok(registry) = catalog::scan(&paths.registry, &installed_ids) {
            for skill in &registry.skills {
                if installed_ids.contains(&skill.id) {
                    let _ = manager.install(skill);
                }
            }
        }
    }

    // Update active project tracker
    match paths.mode {
        ForjaMode::Project => {
            if let Some(ref root) = paths.project_root {
                let active = ActiveProject::new(paths.display_name(), root.clone());
                save_active_project(&active_path, &active)?;
            }
        }
        ForjaMode::Global => {
            clear_active_project(&active_path)?;
        }
    }

    Ok(switched)
}
