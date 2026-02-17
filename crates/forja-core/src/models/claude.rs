use serde::Deserialize;

/// Claude Code team config (`~/.claude/teams/<name>/config.json`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ClaudeTeamConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub lead_agent_id: String,
    #[serde(default)]
    pub lead_session_id: String,
    #[serde(default)]
    pub members: Vec<ClaudeTeamMember>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ClaudeTeamMember {
    #[serde(default)]
    pub agent_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub agent_type: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub joined_at: u64,
    #[serde(default)]
    pub prompt: String,
}

/// Claude Code inbox message (`~/.claude/teams/<name>/inboxes/<member>.json`).
/// The file contains a JSON array of these.
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeInboxMessage {
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub read: bool,
}

/// Claude Code task (`~/.claude/tasks/<team-id>/<n>.json`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ClaudeTask {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub active_form: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub blocks: Vec<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
}
