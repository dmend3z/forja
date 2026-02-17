use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::{RwLock, broadcast};

use forja_core::models::claude::{ClaudeInboxMessage, ClaudeTask, ClaudeTeamConfig};

use super::events::{
    DashboardEvent, MemberSnapshot, MessageGroupSnapshot, MessageSnapshot, TaskGroupSnapshot,
    TaskSnapshot, TeamSnapshot,
};

/// Shared dashboard state, safe for concurrent access.
#[derive(Clone)]
pub struct DashboardState {
    pub teams: Arc<RwLock<HashMap<String, TeamSnapshot>>>,
    pub tasks: Arc<RwLock<HashMap<String, Vec<TaskSnapshot>>>>,
    pub messages: Arc<RwLock<HashMap<String, Vec<MessageGroupSnapshot>>>>,
    /// Maps team name → task directory name (UUID or human-readable).
    pub team_task_mapping: Arc<RwLock<HashMap<String, String>>>,
    pub tx: broadcast::Sender<DashboardEvent>,
}

impl DashboardState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            teams: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            team_task_mapping: Arc::new(RwLock::new(HashMap::new())),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DashboardEvent> {
        self.tx.subscribe()
    }

    /// Build a full snapshot of current state.
    pub async fn snapshot(&self) -> DashboardEvent {
        let teams = self.teams.read().await;
        let tasks = self.tasks.read().await;
        let messages = self.messages.read().await;

        DashboardEvent::Snapshot {
            teams: teams.values().cloned().collect(),
            tasks: tasks
                .iter()
                .map(|(team_name, task_list)| TaskGroupSnapshot {
                    team_name: team_name.clone(),
                    tasks: task_list.clone(),
                })
                .collect(),
            messages: messages.values().flatten().cloned().collect(),
        }
    }

    /// Update (or insert) a team from a parsed config file.
    pub async fn update_team(&self, config: &ClaudeTeamConfig) {
        let snapshot = TeamSnapshot {
            name: config.name.clone(),
            description: config.description.clone(),
            created_at: config.created_at,
            members: config
                .members
                .iter()
                .map(|m| MemberSnapshot {
                    name: m.name.clone(),
                    agent_type: m.agent_type.clone(),
                    model: m.model.clone(),
                    color: m.color.clone(),
                })
                .collect(),
        };

        self.teams
            .write()
            .await
            .insert(config.name.clone(), snapshot.clone());

        // Update team→task mapping if we have a leadSessionId
        if !config.lead_session_id.is_empty() {
            self.team_task_mapping
                .write()
                .await
                .insert(config.name.clone(), config.lead_session_id.clone());
        }

        let _ = self.tx.send(DashboardEvent::TeamUpdated { team: snapshot });
    }

    /// Remove a team by name.
    pub async fn remove_team(&self, team_name: &str) {
        self.teams.write().await.remove(team_name);
        self.team_task_mapping.write().await.remove(team_name);
        self.tasks.write().await.remove(team_name);
        self.messages.write().await.remove(team_name);

        let _ = self.tx.send(DashboardEvent::TeamDeleted {
            team_name: team_name.to_string(),
        });
    }

    /// Update a single task for a team.
    pub async fn update_task(&self, team_name: &str, task: &ClaudeTask) {
        let snapshot = TaskSnapshot {
            id: task.id.clone(),
            subject: task.subject.clone(),
            status: task.status.clone(),
            owner: task.owner.clone(),
            active_form: task.active_form.clone(),
            blocks: task.blocks.clone(),
            blocked_by: task.blocked_by.clone(),
        };

        let mut tasks = self.tasks.write().await;
        let task_list = tasks.entry(team_name.to_string()).or_default();

        // Replace existing task with same ID or append
        if let Some(pos) = task_list.iter().position(|t| t.id == snapshot.id) {
            task_list[pos] = snapshot.clone();
        } else {
            task_list.push(snapshot.clone());
        }

        let _ = self.tx.send(DashboardEvent::TaskUpdated {
            team_name: team_name.to_string(),
            task: snapshot,
        });
    }

    /// Update inbox messages for a team member.
    pub async fn update_inbox(
        &self,
        team_name: &str,
        recipient: &str,
        raw_messages: &[ClaudeInboxMessage],
    ) {
        let messages: Vec<MessageSnapshot> = raw_messages
            .iter()
            .map(|m| MessageSnapshot {
                from: m.from.clone(),
                text: m.text.clone(),
                timestamp: m.timestamp.clone(),
                color: m.color.clone(),
                read: m.read,
            })
            .collect();

        let group = MessageGroupSnapshot {
            team_name: team_name.to_string(),
            recipient: recipient.to_string(),
            messages: messages.clone(),
        };

        let mut all_messages = self.messages.write().await;
        let groups = all_messages.entry(team_name.to_string()).or_default();

        if let Some(pos) = groups.iter().position(|g| g.recipient == recipient) {
            // Send events only for truly new messages
            let old_count = groups[pos].messages.len();
            let new_count = messages.len();
            if new_count > old_count {
                for msg in &messages[old_count..] {
                    let _ = self.tx.send(DashboardEvent::MessageReceived {
                        team_name: team_name.to_string(),
                        recipient: recipient.to_string(),
                        message: msg.clone(),
                    });
                }
            }
            groups[pos] = group;
        } else {
            for msg in &messages {
                let _ = self.tx.send(DashboardEvent::MessageReceived {
                    team_name: team_name.to_string(),
                    recipient: recipient.to_string(),
                    message: msg.clone(),
                });
            }
            groups.push(group);
        }
    }

    /// Perform initial scan of teams and tasks directories.
    pub async fn initial_scan(&self, teams_dir: &Path, tasks_dir: &Path) {
        self.scan_teams(teams_dir).await;
        self.scan_tasks(tasks_dir).await;
    }

    async fn scan_teams(&self, teams_dir: &Path) {
        let entries = match fs::read_dir(teams_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let team_name = entry.file_name().to_string_lossy().to_string();

            // Parse config.json
            let config_path = entry.path().join("config.json");
            if let Some(config) = read_json::<ClaudeTeamConfig>(&config_path) {
                self.update_team(&config).await;
            }

            // Parse inboxes
            let inboxes_dir = entry.path().join("inboxes");
            if inboxes_dir.is_dir()
                && let Ok(inbox_entries) = fs::read_dir(&inboxes_dir)
            {
                for inbox_entry in inbox_entries.flatten() {
                    let path = inbox_entry.path();
                    if path.extension().is_some_and(|e| e == "json") {
                        let recipient = path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        if let Some(messages) = read_json::<Vec<ClaudeInboxMessage>>(&path) {
                            self.update_inbox(&team_name, &recipient, &messages).await;
                        }
                    }
                }
            }
        }
    }

    async fn scan_tasks(&self, tasks_dir: &Path) {
        let entries = match fs::read_dir(tasks_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        // Build reverse mapping: task_dir_name → team_name
        let mapping = self.team_task_mapping.read().await;
        let reverse: HashMap<&str, &str> = mapping
            .iter()
            .map(|(team, task_dir)| (task_dir.as_str(), team.as_str()))
            .collect();

        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().to_string();

            // Try reverse mapping first, fall back to dir name as team name
            let team_name = reverse
                .get(dir_name.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| dir_name.clone());

            if let Ok(task_files) = fs::read_dir(entry.path()) {
                for task_file in task_files.flatten() {
                    let path = task_file.path();
                    if path.extension().is_some_and(|e| e == "json")
                        && let Some(task) = read_json::<ClaudeTask>(&path)
                    {
                        self.update_task(&team_name, &task).await;
                    }
                }
            }
        }
    }

    /// Resolve a task directory UUID to a team name using the mapping.
    pub async fn resolve_team_name(&self, task_dir_name: &str) -> Option<String> {
        let mapping = self.team_task_mapping.read().await;
        mapping
            .iter()
            .find(|(_, v)| v.as_str() == task_dir_name)
            .map(|(k, _)| k.clone())
    }
}

/// Read and parse a JSON file, returning None on any error.
fn read_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Option<T> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}
