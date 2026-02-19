use forja_core::models::decision::{self, DecisionFile};
use std::path::PathBuf;

#[tauri::command]
pub fn list_decisions(project_path: String) -> Result<Vec<DecisionFile>, String> {
    let decisions_dir = PathBuf::from(&project_path).join(".forja").join("decisions");
    decision::discover_decisions(&decisions_dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_decision(project_path: String, decision_id: String) -> Result<DecisionFile, String> {
    let decisions_dir = PathBuf::from(&project_path).join(".forja").join("decisions");
    decision::find_decision(&decisions_dir, &decision_id).map_err(|e| e.to_string())
}
