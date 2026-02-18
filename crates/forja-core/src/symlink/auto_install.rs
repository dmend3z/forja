use crate::error::Result;
use crate::paths::ForjaPaths;
use crate::registry::catalog;
use crate::symlink::manager::{SymlinkManager, load_installed_ids, save_installed_ids};

/// Result of an auto-install operation.
pub struct AutoInstallResult {
    pub installed: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub not_found: Vec<String>,
}

/// Auto-install missing agent symlinks for the given skill IDs.
///
/// Returns a structured result — the caller is responsible for printing output.
pub fn auto_install_missing(paths: &ForjaPaths, skill_ids: &[&str]) -> Result<AutoInstallResult> {
    let installed_ids = load_installed_ids(&paths.state);
    let missing: Vec<&str> = skill_ids
        .iter()
        .copied()
        .filter(|id| !installed_ids.contains(&id.to_string()))
        .collect();

    if missing.is_empty() {
        return Ok(AutoInstallResult {
            installed: vec![],
            failed: vec![],
            not_found: vec![],
        });
    }

    let mut current_ids = installed_ids;
    let registry = catalog::scan(&paths.registry, &current_ids)?;
    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());

    let mut installed = Vec::new();
    let mut failed = Vec::new();
    let mut not_found = Vec::new();

    for skill_id in &missing {
        match registry.find_by_id(skill_id) {
            Some(skill) => match manager.install(skill) {
                Ok(_) => {
                    current_ids.push(skill_id.to_string());
                    installed.push(skill_id.to_string());
                }
                Err(e) => {
                    failed.push((skill_id.to_string(), e.to_string()));
                }
            },
            None => {
                not_found.push(skill_id.to_string());
            }
        }
    }

    save_installed_ids(&paths.state, &current_ids)?;

    Ok(AutoInstallResult {
        installed,
        failed,
        not_found,
    })
}
