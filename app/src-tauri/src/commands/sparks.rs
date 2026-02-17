use forja_spark::events::SparkStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;
use tokio::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SparkType {
    Task,
    QuickFix,
    Plan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkInfo {
    pub id: String,
    pub project_id: String,
    pub spark_type: SparkType,
    pub description: String,
    pub status: SparkStatus,
    pub created_at: String,
    pub finished_at: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
}

pub type SparkStore = Arc<Mutex<HashMap<String, SparkInfo>>>;

fn build_quickfix_prompt(description: &str) -> String {
    format!(
        "Execute this as a quick fix.\n\
         ## Task\n\
         {description}\n\
         ## Rules\n\
         - Read CLAUDE.md before starting\n\
         - Make minimal, focused changes\n\
         - Run tests if available\n\
         - Commit with a descriptive message"
    )
}

fn build_cli_args(spark_type: SparkType, description: &str) -> Vec<String> {
    let prompt = match spark_type {
        SparkType::Task => description.to_string(),
        SparkType::Plan => {
            format!("Create a detailed implementation plan for: {description}")
        }
        SparkType::QuickFix => build_quickfix_prompt(description),
    };

    vec![
        "--dangerously-skip-permissions".into(),
        "--print".into(),
        "--".into(),
        prompt,
    ]
}

#[tauri::command]
pub async fn start_spark(
    project_id: String,
    spark_type: SparkType,
    description: String,
    project_path: String,
    store: State<'_, SparkStore>,
) -> Result<SparkInfo, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let spark = SparkInfo {
        id: id.clone(),
        project_id: project_id.clone(),
        spark_type,
        description: description.clone(),
        status: SparkStatus::Starting,
        created_at: now,
        finished_at: None,
        output: None,
        error: None,
    };

    store
        .lock()
        .map_err(|e| format!("Lock error: {e}"))?
        .insert(id.clone(), spark.clone());

    let args = build_cli_args(spark_type, &description);
    let store_handle = Arc::clone(&store);

    tokio::spawn(async move {
        // Update to Running
        if let Ok(mut map) = store_handle.lock() {
            if let Some(s) = map.get_mut(&id) {
                s.status = SparkStatus::Running;
            }
        }

        let result = Command::new("claude")
            .args(&args)
            .current_dir(&project_path)
            .output()
            .await;

        let finished_at = chrono::Utc::now().to_rfc3339();

        if let Ok(mut map) = store_handle.lock() {
            if let Some(s) = map.get_mut(&id) {
                s.finished_at = Some(finished_at);
                match result {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                        if output.status.success() {
                            s.status = SparkStatus::Stopped;
                            s.output = if stdout.is_empty() {
                                None
                            } else {
                                Some(stdout)
                            };
                        } else {
                            s.status = SparkStatus::Failed;
                            s.error = Some(if stderr.is_empty() { stdout } else { stderr });
                        }
                    }
                    Err(e) => {
                        s.status = SparkStatus::Failed;
                        s.error = Some(format!("Failed to run claude CLI: {e}"));
                    }
                }
            }
        }
    });

    Ok(spark)
}

#[tauri::command]
pub fn list_sparks(
    project_id: String,
    store: State<'_, SparkStore>,
) -> Result<Vec<SparkInfo>, String> {
    let map = store.lock().map_err(|e| format!("Lock error: {e}"))?;

    let mut sparks: Vec<SparkInfo> = map
        .values()
        .filter(|s| s.project_id == project_id)
        .cloned()
        .collect();

    sparks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(sparks)
}
