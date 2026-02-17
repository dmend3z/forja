use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Chat-level events parsed from `claude --output-format stream-json`.
///
/// These map to individual streaming chunks that the frontend renders
/// as they arrive — text deltas become typing animation, tool uses
/// become collapsible cards, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatEvent {
    MessageStart {
        id: String,
        role: String,
    },
    TextDelta {
        index: usize,
        text: String,
    },
    ToolUseStart {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        id: String,
        content: String,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageStop,
    Error {
        message: String,
    },
}

/// Spark-level events that wrap chat events with spark identity.
///
/// The Tauri layer emits these as `app.emit("spark:{id}", event)` so
/// the frontend can route events to the correct spark tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SparkEvent {
    StatusChanged {
        spark_id: String,
        status: SparkStatus,
    },
    Chat {
        spark_id: String,
        event: ChatEvent,
    },
}

/// Lifecycle status of a spark (claude CLI process).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SparkStatus {
    Starting,
    Running,
    Idle,
    Stopped,
    Failed,
}
