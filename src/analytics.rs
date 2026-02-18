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
        assert_eq!(events[0].command, "task");
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
    fn track_caps_at_max() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("analytics.json");

        // Pre-fill with MAX_EVENTS
        let events: Vec<AnalyticsEvent> = (0..MAX_EVENTS)
            .map(|i| AnalyticsEvent {
                skill_id: format!("skill-{i}"),
                command: "task".to_string(),
                timestamp: "2026-01-01T00:00:00Z".to_string(),
            })
            .collect();
        let json = serde_json::to_string(&events).unwrap();
        fs::write(&path, json).unwrap();

        // Track one more
        track(&path, "new-skill", "task").unwrap();

        let loaded = load(&path);
        assert_eq!(loaded.len(), MAX_EVENTS);
        // Oldest event should have been trimmed; newest should be last
        assert_eq!(loaded.last().unwrap().skill_id, "new-skill");
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let events = load(Path::new("/nonexistent/analytics.json"));
        assert!(events.is_empty());
    }

    #[test]
    fn load_invalid_json_returns_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("analytics.json");
        fs::write(&path, "not json").unwrap();

        let events = load(&path);
        assert!(events.is_empty());
    }
}
