use crate::error::Result;
use crate::models::active_project::{clear_active_project, load_active_project};
use crate::paths::{ForjaMode, ForjaPaths};
use crate::registry::catalog;
use crate::symlink::manager::{SymlinkManager, load_installed_ids};

/// Result of a sync operation, containing info about what happened.
pub struct SyncResult {
    /// Whether the active project switched (global mode only).
    pub switched: bool,
    /// If switched, the name of the previous project.
    pub previous_project: Option<String>,
    /// The current project display name.
    pub current_project: String,
}

/// Rebuild all `forja--` symlinks from the current context's state.
///
/// In **project** mode, symlinks go to `<project>/.claude/` — no global tracking needed.
/// In **global** mode, symlinks go to `~/.claude/` with active-project tracking.
///
/// Returns a `SyncResult` with switch information. The caller handles output.
pub fn sync_symlinks(paths: &ForjaPaths) -> Result<SyncResult> {
    let current_project = paths.display_name();

    // Project mode: symlinks are isolated per-project, no global tracking needed
    if paths.mode == ForjaMode::Project {
        let manager =
            SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
        manager.remove_project_symlinks(&paths.registry)?;

        let installed_ids = load_installed_ids(&paths.state);
        if !installed_ids.is_empty()
            && let Ok(registry) = catalog::scan(&paths.registry, &installed_ids)
        {
            for skill in &registry.skills {
                if installed_ids.contains(&skill.id) {
                    let _ = manager.install(skill);
                }
            }
        }
        return Ok(SyncResult {
            switched: false,
            previous_project: None,
            current_project,
        });
    }

    // Global mode: existing logic with active-project tracking
    let global_forja = ForjaPaths::global_forja_root()?;
    let active_path = global_forja.join("active_project.json");
    let previous = load_active_project(&active_path);

    let mut switched = false;
    let mut previous_project = None;

    // Detect project switch
    if let Some(ref prev) = previous {
        let current_root = paths.project_root.as_deref();
        let prev_root = Some(prev.project_root.as_path());
        if current_root != prev_root {
            switched = true;
            previous_project = Some(prev.project_name.clone());
        }
    }

    // Remove only this project's symlinks (preserves other projects' symlinks)
    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    manager.remove_project_symlinks(&paths.registry)?;

    // Recreate symlinks from current state
    let installed_ids = load_installed_ids(&paths.state);
    if !installed_ids.is_empty()
        && let Ok(registry) = catalog::scan(&paths.registry, &installed_ids)
    {
        for skill in &registry.skills {
            if installed_ids.contains(&skill.id) {
                let _ = manager.install(skill);
            }
        }
    }

    // Update active project tracker
    clear_active_project(&active_path)?;

    Ok(SyncResult {
        switched,
        previous_project,
        current_project,
    })
}
