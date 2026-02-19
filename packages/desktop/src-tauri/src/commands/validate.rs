use forja_core::models::validate;
use std::path::PathBuf;

#[derive(serde::Serialize)]
pub struct ValidationResultDto {
    pub is_valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub errors: Vec<ValidationErrorDto>,
}

#[derive(serde::Serialize)]
pub struct ValidationErrorDto {
    pub file: String,
    pub message: String,
    pub severity: String,
}

#[tauri::command]
pub fn validate_project(project_path: String) -> Result<ValidationResultDto, String> {
    let forja_dir = PathBuf::from(&project_path).join(".forja");
    let result = validate::validate_project(&forja_dir).map_err(|e| e.to_string())?;

    Ok(ValidationResultDto {
        is_valid: result.is_valid(),
        error_count: result.error_count(),
        warning_count: result.warning_count(),
        errors: result
            .errors
            .iter()
            .map(|e| ValidationErrorDto {
                file: e.file.clone(),
                message: e.message.clone(),
                severity: match e.severity {
                    validate::Severity::Error => "error".to_string(),
                    validate::Severity::Warning => "warning".to_string(),
                },
            })
            .collect(),
    })
}
