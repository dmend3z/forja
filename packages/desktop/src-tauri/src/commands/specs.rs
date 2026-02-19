use forja_core::models::spec::{self, SpecFile};
use std::path::PathBuf;

fn resolve_specs_dir(project_path: &str) -> PathBuf {
    let forja_specs_dir = PathBuf::from(project_path).join(".forja").join("specs");
    if forja_specs_dir.exists() {
        forja_specs_dir
    } else {
        PathBuf::from(project_path).join("docs").join("specs")
    }
}

#[tauri::command]
pub fn list_specs(project_path: String) -> Result<Vec<SpecFile>, String> {
    let specs_dir = resolve_specs_dir(&project_path);

    match spec::discover_specs(&specs_dir) {
        Ok(specs) => Ok(specs),
        Err(e) => {
            // Missing specs dir is not an error — just no specs yet
            if e.to_string().contains("not found") {
                Ok(Vec::new())
            } else {
                Err(e.to_string())
            }
        }
    }
}

#[tauri::command]
pub fn get_spec(project_path: String, spec_id: String) -> Result<SpecFile, String> {
    let specs_dir = resolve_specs_dir(&project_path);
    spec::find_spec(&specs_dir, &spec_id).map_err(|e| e.to_string())
}
