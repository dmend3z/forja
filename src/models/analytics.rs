use serde::{Deserialize, Serialize};

/// A single skill usage event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub skill_id: String,
    pub command: String,
    pub timestamp: String,
}
