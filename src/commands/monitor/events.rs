use serde::Serialize;

/// Events streamed to the dashboard via SSE.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum DashboardEvent {
    /// Full state snapshot, sent on initial SSE connection.
    Snapshot {
        teams: Vec<TeamSnapshot>,
        tasks: Vec<TaskGroupSnapshot>,
        messages: Vec<MessageGroupSnapshot>,
    },

    TeamUpdated {
        team: TeamSnapshot,
    },
    TeamDeleted {
        team_name: String,
    },

    TaskUpdated {
        team_name: String,
        task: TaskSnapshot,
    },
    TaskDeleted {
        team_name: String,
        task_id: String,
    },

    MessageReceived {
        team_name: String,
        recipient: String,
        message: MessageSnapshot,
    },

    Heartbeat,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamSnapshot {
    pub name: String,
    pub description: String,
    pub created_at: u64,
    pub members: Vec<MemberSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberSnapshot {
    pub name: String,
    pub agent_type: String,
    pub model: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskGroupSnapshot {
    pub team_name: String,
    pub tasks: Vec<TaskSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskSnapshot {
    pub id: String,
    pub subject: String,
    pub status: String,
    pub owner: String,
    pub active_form: String,
    pub blocks: Vec<String>,
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageGroupSnapshot {
    pub team_name: String,
    pub recipient: String,
    pub messages: Vec<MessageSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageSnapshot {
    pub from: String,
    pub text: String,
    pub timestamp: String,
    pub color: String,
    pub read: bool,
}
