use forja_core::models::run_log::{self, RunLog};
use std::path::PathBuf;

#[tauri::command]
pub fn list_runs(project_path: String) -> Result<Vec<RunLog>, String> {
    let runs_dir = PathBuf::from(&project_path).join(".forja").join("runs");
    run_log::discover_runs(&runs_dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_run(project_path: String, run_id: String) -> Result<RunLog, String> {
    let runs_dir = PathBuf::from(&project_path).join(".forja").join("runs");
    let path = runs_dir.join(format!("{run_id}.md"));
    run_log::load_run_log(&path).map_err(|e| e.to_string())
}
