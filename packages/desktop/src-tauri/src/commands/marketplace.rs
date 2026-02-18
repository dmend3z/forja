use forja_core::frontmatter;
use forja_core::models::agent_file::{AgentFile, SkillDetail};
use forja_core::models::skill::Skill;
use forja_core::models::state::{load_state, save_state, InstallMeta};
use forja_core::paths::ForjaPaths;
use forja_core::registry::catalog;
use forja_core::symlink::manager::{load_installed_ids, save_installed_ids, SymlinkManager};
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Emitter;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForjaPathsDto {
    pub registry: String,
    pub state: String,
}

#[tauri::command]
pub fn get_forja_paths() -> Result<ForjaPathsDto, String> {
    let paths = ForjaPaths::global().map_err(|e| e.to_string())?;
    Ok(ForjaPathsDto {
        registry: paths.registry.to_string_lossy().to_string(),
        state: paths.state.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn list_skills(registry_path: String) -> Result<Vec<Skill>, String> {
    let path = std::path::PathBuf::from(&registry_path);
    let paths = ForjaPaths::global().map_err(|e| e.to_string())?;
    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&path, &installed_ids).map_err(|e| e.to_string())?;
    Ok(registry.skills)
}

#[tauri::command]
pub fn search_skills(registry_path: String, query: String) -> Result<Vec<Skill>, String> {
    let path = std::path::PathBuf::from(&registry_path);
    let paths = ForjaPaths::global().map_err(|e| e.to_string())?;
    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&path, &installed_ids).map_err(|e| e.to_string())?;
    let results: Vec<Skill> = registry.search(&query).into_iter().cloned().collect();
    Ok(results)
}

#[tauri::command]
pub fn get_skill_detail(registry_path: String, skill_id: String) -> Result<SkillDetail, String> {
    let path = std::path::PathBuf::from(&registry_path);
    let paths = ForjaPaths::global().map_err(|e| e.to_string())?;
    let installed_ids = load_installed_ids(&paths.state);
    let registry = catalog::scan(&path, &installed_ids).map_err(|e| e.to_string())?;

    let skill = registry
        .find_by_id(&skill_id)
        .ok_or_else(|| format!("Skill not found: {skill_id}"))?
        .clone();

    let agents = read_agent_files(&skill.path);
    let skill_files = list_md_files(&skill.path.join("skills"));
    let command_files = list_md_files(&skill.path.join("commands"));

    Ok(SkillDetail {
        skill,
        agents,
        skill_files,
        command_files,
    })
}

#[tauri::command]
pub fn install_skill(registry_path: String, skill_id: String) -> Result<(), String> {
    let path = std::path::PathBuf::from(&registry_path);
    let paths = ForjaPaths::global().map_err(|e| e.to_string())?;
    let mut installed_ids = load_installed_ids(&paths.state);

    if installed_ids.contains(&skill_id) {
        return Err(format!("Skill already installed: {skill_id}"));
    }

    let registry = catalog::scan(&path, &installed_ids).map_err(|e| e.to_string())?;
    let skill = registry
        .find_by_id(&skill_id)
        .ok_or_else(|| format!("Skill not found: {skill_id}"))?;

    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    manager.install(skill).map_err(|e| e.to_string())?;

    installed_ids.push(skill_id.clone());
    save_installed_ids(&paths.state, &installed_ids).map_err(|e| e.to_string())?;

    // Record install metadata
    let mut state = load_state(&paths.state);
    state.install_metadata.insert(
        skill_id,
        InstallMeta {
            install_date: chrono::Utc::now().to_rfc3339(),
            last_used: None,
        },
    );
    save_state(&paths.state, &state).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn uninstall_skill(skill_id: String) -> Result<(), String> {
    let paths = ForjaPaths::global().map_err(|e| e.to_string())?;
    let mut installed_ids = load_installed_ids(&paths.state);

    if !installed_ids.contains(&skill_id) {
        return Err(format!("Skill not installed: {skill_id}"));
    }

    let manager = SymlinkManager::new(paths.claude_agents.clone(), paths.claude_commands.clone());
    manager.uninstall(&skill_id).map_err(|e| e.to_string())?;

    installed_ids.retain(|id| id != &skill_id);
    save_installed_ids(&paths.state, &installed_ids).map_err(|e| e.to_string())?;

    // Remove install metadata
    let mut state = load_state(&paths.state);
    state.install_metadata.remove(&skill_id);
    save_state(&paths.state, &state).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn create_skill(
    app: tauri::AppHandle,
    name: String,
    phase: String,
    tech: String,
    description: String,
) -> Result<String, String> {
    let paths = ForjaPaths::global().map_err(|e| e.to_string())?;
    let skill_dir = paths
        .registry
        .join("skills")
        .join(&phase)
        .join(&tech)
        .join(&name);

    // Create directory structure
    fs::create_dir_all(skill_dir.join("agents")).map_err(|e| e.to_string())?;

    // Create skill.json manifest
    let manifest = serde_json::json!({
        "name": name,
        "description": description,
    });
    fs::write(
        skill_dir.join("skill.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .map_err(|e| e.to_string())?;

    let prompt = format!(
        "Generate an agent markdown file for a Claude Code agent called '{name}'.\n\
         Description: {description}\n\
         Phase: {phase}, Tech: {tech}\n\n\
         Output ONLY the complete markdown content (with YAML frontmatter between --- delimiters).\n\
         The frontmatter should have: name, description, tools (comma-separated list of tools the agent needs).\n\
         The body should contain the agent's system prompt instructions."
    );

    let agent_path = skill_dir.join("agents").join(format!("{name}.md"));
    let agent_path_str = agent_path.to_string_lossy().to_string();

    tokio::spawn(async move {
        let _ = app.emit("create-skill-output", "Generating agent with Claude...\n");

        let result = Command::new("claude")
            .args(["--print", "--", &prompt])
            .output()
            .await;

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if output.status.success() {
                    let _ = fs::write(&agent_path_str, &stdout);
                    let _ = app.emit("create-skill-output", format!("Done! Agent saved to {agent_path_str}\n"));
                    let _ = app.emit("create-skill-done", stdout);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let _ = app.emit("create-skill-error", stderr);
                }
            }
            Err(e) => {
                let _ = app.emit("create-skill-error", format!("Failed to run claude CLI: {e}"));
            }
        }
    });

    Ok(format!("{phase}/{tech}/{name}"))
}

/// Read and parse all agent `.md` files from a skill's agents/ directory.
fn read_agent_files(skill_path: &std::path::Path) -> Vec<AgentFile> {
    let agents_dir = skill_path.join("agents");
    if !agents_dir.exists() {
        return Vec::new();
    }

    let mut agents = Vec::new();
    let Ok(entries) = fs::read_dir(&agents_dir) else {
        return agents;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let filename = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                match frontmatter::parse_agent_frontmatter(&content) {
                    Ok((fm, body)) => {
                        agents.push(AgentFile {
                            filename,
                            frontmatter: fm,
                            body,
                        });
                    }
                    Err(_) => {
                        // Agent file without valid frontmatter — still show it
                        agents.push(AgentFile {
                            filename,
                            frontmatter: forja_core::models::agent_file::AgentFrontmatter {
                                name: path
                                    .file_stem()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                                description: None,
                                tools: None,
                                model: None,
                            },
                            body: content,
                        });
                    }
                }
            }
        }
    }

    agents.sort_by(|a, b| a.filename.cmp(&b.filename));
    agents
}

/// List `.md` filenames in a directory.
fn list_md_files(dir: &std::path::Path) -> Vec<String> {
    if !dir.exists() {
        return Vec::new();
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut files: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                Some(path.file_name()?.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();

    files.sort();
    files
}
