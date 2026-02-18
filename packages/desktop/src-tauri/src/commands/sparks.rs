use forja_spark::events::SparkStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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
/// Maps spark ID → child PID for killing running sparks.
pub type ChildPidStore = Arc<Mutex<HashMap<String, u32>>>;

fn sparks_file(project_id: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".forja")
        .join("desktop")
        .join(format!("sparks-{project_id}.json"))
}

fn load_sparks_from_disk(project_id: &str) -> Vec<SparkInfo> {
    let path = sparks_file(project_id);
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_sparks_to_disk(project_id: &str, sparks: &[SparkInfo]) {
    let path = sparks_file(project_id);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(sparks) {
        let _ = fs::write(&path, json);
    }
}

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
    pids: State<'_, ChildPidStore>,
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
    let pids_handle = Arc::clone(&pids);
    let pid_for_spawn = project_id.clone();

    let child = Command::new("claude")
        .args(&args)
        .current_dir(&project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn claude CLI: {e}"))?;

    // Store PID so stop_spark can kill the process
    if let Some(child_pid) = child.id() {
        if let Ok(mut pid_map) = pids_handle.lock() {
            pid_map.insert(id.clone(), child_pid);
        }
    }

    // Update to Running
    if let Ok(mut map) = store_handle.lock() {
        if let Some(s) = map.get_mut(&id) {
            s.status = SparkStatus::Running;
        }
    }

    tokio::spawn(async move {
        let result = child.wait_with_output().await;
        let finished_at = chrono::Utc::now().to_rfc3339();

        // Clean up PID entry
        if let Ok(mut pid_map) = pids_handle.lock() {
            pid_map.remove(&id);
        }

        if let Ok(mut map) = store_handle.lock() {
            if let Some(s) = map.get_mut(&id) {
                // Race guard: only update if still Running (stop_spark may have set Stopped)
                if s.status != SparkStatus::Running {
                    return;
                }

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

            // Persist terminal sparks to disk
            let terminal: Vec<SparkInfo> = map
                .values()
                .filter(|s| s.project_id == pid_for_spawn && is_terminal(s.status))
                .cloned()
                .collect();
            save_sparks_to_disk(&pid_for_spawn, &terminal);
        }
    });

    Ok(spark)
}

fn is_terminal(status: SparkStatus) -> bool {
    matches!(status, SparkStatus::Stopped | SparkStatus::Failed)
}

#[tauri::command]
pub fn list_sparks(
    project_id: String,
    store: State<'_, SparkStore>,
) -> Result<Vec<SparkInfo>, String> {
    let map = store.lock().map_err(|e| format!("Lock error: {e}"))?;

    // In-memory sparks (includes running + recently finished)
    let memory_ids: std::collections::HashSet<String> = map
        .values()
        .filter(|s| s.project_id == project_id)
        .map(|s| s.id.clone())
        .collect();

    let mut sparks: Vec<SparkInfo> = map
        .values()
        .filter(|s| s.project_id == project_id)
        .cloned()
        .collect();

    // Merge in disk-persisted sparks not already in memory
    for disk_spark in load_sparks_from_disk(&project_id) {
        if !memory_ids.contains(&disk_spark.id) {
            sparks.push(disk_spark);
        }
    }

    sparks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(sparks)
}

#[tauri::command]
pub fn stop_spark(
    id: String,
    store: State<'_, SparkStore>,
    pids: State<'_, ChildPidStore>,
) -> Result<(), String> {
    // Kill the child process via PID
    if let Ok(mut pid_map) = pids.lock() {
        if let Some(pid) = pid_map.remove(&id) {
            // Send SIGKILL on unix, TerminateProcess on windows
            let _ = kill_process(pid);
        }
    }

    // Update status to Stopped
    if let Ok(mut map) = store.lock() {
        if let Some(s) = map.get_mut(&id) {
            s.status = SparkStatus::Stopped;
            s.finished_at = Some(chrono::Utc::now().to_rfc3339());

            // Persist to disk
            let project_id = s.project_id.clone();
            let terminal: Vec<SparkInfo> = map
                .values()
                .filter(|s| s.project_id == project_id && is_terminal(s.status))
                .cloned()
                .collect();
            save_sparks_to_disk(&project_id, &terminal);
        }
    }

    Ok(())
}

fn kill_process(pid: u32) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()?;
    }
    #[cfg(not(unix))]
    {
        std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output()?;
    }
    Ok(())
}
