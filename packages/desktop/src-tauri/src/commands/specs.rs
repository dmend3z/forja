use forja_core::models::spec::{self, SpecFile};
use std::path::PathBuf;

#[tauri::command]
pub fn list_specs(project_path: String) -> Result<Vec<SpecFile>, String> {
    let specs_dir = PathBuf::from(&project_path).join("docs").join("specs");

    match spec::discover_specs(&specs_dir) {
        Ok(specs) => Ok(specs),
        Err(e) => {
            // Missing docs/specs/ is not an error — just no specs yet
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
    let specs_dir = PathBuf::from(&project_path).join("docs").join("specs");
    spec::find_spec(&specs_dir, &spec_id).map_err(|e| e.to_string())
}
