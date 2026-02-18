use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use tokio::sync::mpsc;

use forja_core::models::claude::{ClaudeInboxMessage, ClaudeTask, ClaudeTeamConfig};

use super::state::DashboardState;

/// What kind of change was detected.
enum FileChange {
    /// Team config.json was created or modified.
    TeamConfig { path: PathBuf },
    /// An inbox file was created or modified.
    InboxUpdate {
        team_name: String,
        recipient: String,
        path: PathBuf,
    },
    /// A task file was created or modified.
    TaskUpdate { task_dir: String, path: PathBuf },
    /// A team directory was deleted.
    TeamDeleted { team_name: String },
}

/// Classify a filesystem event path into a meaningful `FileChange`.
///
/// Path patterns we care about:
///   teams/<name>/config.json          → TeamConfig
///   teams/<name>/inboxes/<member>.json → InboxUpdate
///   tasks/<dir>/<n>.json              → TaskUpdate
///   teams/<name>/ (removed)           → TeamDeleted
///
/// TODO(daniel): This is the classify_event function from the plan.
/// The design choice here is: we match on path components relative to
/// the watched root (~/.claude/). Corrupted files are handled downstream
/// by read_json returning None. We only classify .json files.
fn classify_event(path: &Path, teams_dir: &Path, tasks_dir: &Path) -> Option<FileChange> {
    // Check if this path is under teams/
    if let Ok(rel) = path.strip_prefix(teams_dir) {
        let components: Vec<&str> = rel
            .components()
            .map(|c| c.as_os_str().to_str().unwrap_or(""))
            .collect();

        match components.as_slice() {
            // teams/<name>/config.json
            [_team_name, "config.json"] => Some(FileChange::TeamConfig {
                path: path.to_path_buf(),
            }),
            // teams/<name>/inboxes/<member>.json
            [team_name, "inboxes", filename] if filename.ends_with(".json") => {
                let recipient = filename.strip_suffix(".json").unwrap_or(filename);
                Some(FileChange::InboxUpdate {
                    team_name: team_name.to_string(),
                    recipient: recipient.to_string(),
                    path: path.to_path_buf(),
                })
            }
            // teams/<name> (directory itself — check if deleted)
            [team_name] if !path.exists() => Some(FileChange::TeamDeleted {
                team_name: team_name.to_string(),
            }),
            _ => None,
        }
    }
    // Check if this path is under tasks/
    else if let Ok(rel) = path.strip_prefix(tasks_dir) {
        let components: Vec<&str> = rel
            .components()
            .map(|c| c.as_os_str().to_str().unwrap_or(""))
            .collect();

        match components.as_slice() {
            // tasks/<dir>/<n>.json
            [task_dir, filename] if filename.ends_with(".json") && !filename.starts_with('.') => {
                Some(FileChange::TaskUpdate {
                    task_dir: task_dir.to_string(),
                    path: path.to_path_buf(),
                })
            }
            _ => None,
        }
    } else {
        None
    }
}

/// Start watching for filesystem changes and feed them into the dashboard state.
pub async fn watch(
    state: Arc<DashboardState>,
    teams_dir: PathBuf,
    tasks_dir: PathBuf,
) -> notify::Result<()> {
    let (tx, mut rx) = mpsc::channel::<Vec<PathBuf>>(100);

    let teams_dir_clone = teams_dir.clone();
    let tasks_dir_clone = tasks_dir.clone();

    // Spawn the debounced file watcher on a blocking thread
    // (notify uses its own threads, but the callback bridges to our async channel)
    tokio::task::spawn_blocking(move || {
        let rt = tx;
        let mut debouncer = new_debouncer(
            Duration::from_millis(200),
            move |events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
                if let Ok(events) = events {
                    let paths: Vec<PathBuf> = events
                        .into_iter()
                        .filter(|e| e.kind == DebouncedEventKind::Any)
                        .map(|e| e.path)
                        .collect();
                    if !paths.is_empty() {
                        let _ = rt.blocking_send(paths);
                    }
                }
            },
        )
        .expect("Failed to create file watcher");

        // Watch both directories recursively
        if teams_dir_clone.exists() {
            debouncer
                .watcher()
                .watch(&teams_dir_clone, RecursiveMode::Recursive)
                .ok();
        }
        if tasks_dir_clone.exists() {
            debouncer
                .watcher()
                .watch(&tasks_dir_clone, RecursiveMode::Recursive)
                .ok();
        }

        // Keep the debouncer alive until the channel is dropped
        loop {
            std::thread::sleep(Duration::from_secs(60));
        }
    });

    // Process file change events
    while let Some(paths) = rx.recv().await {
        for path in paths {
            let change = classify_event(&path, &teams_dir, &tasks_dir);
            match change {
                Some(FileChange::TeamConfig { path }) => {
                    if let Some(config) = read_json::<ClaudeTeamConfig>(&path) {
                        state.update_team(&config).await;
                    }
                }
                Some(FileChange::InboxUpdate {
                    team_name,
                    recipient,
                    path,
                }) => {
                    if let Some(messages) = read_json::<Vec<ClaudeInboxMessage>>(&path) {
                        state.update_inbox(&team_name, &recipient, &messages).await;
                    }
                }
                Some(FileChange::TaskUpdate { task_dir, path }) => {
                    // Resolve task dir to team name
                    let team_name = state.resolve_team_name(&task_dir).await.unwrap_or(task_dir);
                    if let Some(task) = read_json::<ClaudeTask>(&path) {
                        state.update_task(&team_name, &task).await;
                    }
                }
                Some(FileChange::TeamDeleted { team_name }) => {
                    state.remove_team(&team_name).await;
                }
                None => {}
            }
        }
    }

    Ok(())
}

/// Read and parse a JSON file, returning None on any error (torn reads, corruption).
fn read_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Option<T> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}
