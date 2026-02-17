use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub last_opened: String,
    pub forja_initialized: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectList {
    pub projects: Vec<Project>,
    pub active_project_id: Option<String>,
}

fn projects_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".forja")
        .join("desktop")
        .join("projects.json")
}

fn load_projects() -> ProjectList {
    let path = projects_file();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or(ProjectList {
            projects: vec![],
            active_project_id: None,
        }),
        Err(_) => ProjectList {
            projects: vec![],
            active_project_id: None,
        },
    }
}

fn save_projects(list: &ProjectList) -> Result<(), String> {
    let path = projects_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let json =
        serde_json::to_string_pretty(list).map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn list_projects() -> Result<ProjectList, String> {
    Ok(load_projects())
}

#[tauri::command]
pub fn add_project(path: String) -> Result<Project, String> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let name = path_buf
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let forja_initialized = path_buf.join(".forja").exists();

    let project = Project {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        path: path_buf,
        last_opened: chrono::Utc::now().to_rfc3339(),
        forja_initialized,
    };

    let mut list = load_projects();
    list.projects.push(project.clone());
    save_projects(&list)?;

    Ok(project)
}
