use forja_core::models::plan::{self, PlanMetadata};
use std::path::PathBuf;

#[tauri::command]
pub fn list_plans(project_path: String) -> Result<Vec<PlanMetadata>, String> {
    let plans_dir = PathBuf::from(&project_path).join(".forja").join("plans");

    if !plans_dir.exists() {
        return Ok(Vec::new());
    }

    let mut json_files: Vec<_> = std::fs::read_dir(&plans_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|ext| ext == "json")
                && !p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .contains(".checkpoint.")
        })
        .collect();

    json_files.sort();

    let mut plans = Vec::new();
    for path in json_files.into_iter().rev() {
        if let Ok(p) = plan::load_plan(&path) {
            plans.push(p);
        }
    }

    Ok(plans)
}

#[tauri::command]
pub fn get_plan(project_path: String, plan_id: String) -> Result<PlanMetadata, String> {
    let plans_dir = PathBuf::from(&project_path).join(".forja").join("plans");

    if !plans_dir.exists() {
        return Err("plans directory not found".to_string());
    }

    let entries = std::fs::read_dir(&plans_dir).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json")
            && !path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .contains(".checkpoint.")
        {
            if let Ok(p) = plan::load_plan(&path) {
                if p.id == plan_id {
                    return Ok(p);
                }
            }
        }
    }

    Err(format!("plan not found: {plan_id}"))
}
