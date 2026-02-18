use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::models::analytics::AnalyticsEvent;

const MAX_EVENTS: usize = 10_000;

/// Track a skill usage event. Appends to the analytics file, capped at MAX_EVENTS.
pub fn track(analytics_path: &Path, skill_id: &str, command: &str) -> Result<()> {
    let mut events = load(analytics_path);

    events.push(AnalyticsEvent {
        skill_id: skill_id.to_string(),
        command: command.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    });

    // Cap at MAX_EVENTS by trimming oldest
    if events.len() > MAX_EVENTS {
        let excess = events.len() - MAX_EVENTS;
        events.drain(..excess);
    }

    let json = serde_json::to_string_pretty(&events)?;
    if let Some(parent) = analytics_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(analytics_path, json)?;

    Ok(())
}

/// Load analytics events from file. Returns empty vec if file doesn't exist or is invalid.
pub fn load(analytics_path: &Path) -> Vec<AnalyticsEvent> {
    fs::read_to_string(analytics_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

/// Analytics path relative to forja root.
pub fn analytics_path(forja_root: &Path) -> std::path::PathBuf {
    forja_root.join("analytics.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn track_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("analytics.json");

        track(&path, "code/rust/coder", "task").unwrap();

        let events = load(&path);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].skill_id, "code/rust/coder");
    }

    #[test]
    fn track_appends() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("analytics.json");

        track(&path, "code/rust/coder", "task").unwrap();
        track(&path, "test/tdd/workflow", "task").unwrap();

        let events = load(&path);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let events = load(Path::new("/nonexistent/analytics.json"));
        assert!(events.is_empty());
    }
}
