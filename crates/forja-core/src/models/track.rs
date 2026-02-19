use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{ForjaError, Result};
use crate::frontmatter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TrackStatus {
    Draft,
    InProgress,
    Complete,
    Archived,
}

impl TrackStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::InProgress => "in-progress",
            Self::Complete => "complete",
            Self::Archived => "archived",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrackPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackFrontmatter {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TrackStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<TrackPriority>,
    pub created: String,
}

/// A single item row parsed from the track's markdown table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackItem {
    pub id: String,
    pub task: String,
    pub status: String,
    pub spec: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackFile {
    #[serde(flatten)]
    pub frontmatter: TrackFrontmatter,
    pub body: String,
    pub items: Vec<TrackItem>,
}

impl TrackFile {
    pub fn id(&self) -> &str {
        &self.frontmatter.id
    }

    pub fn title(&self) -> &str {
        &self.frontmatter.title
    }

    pub fn progress(&self) -> (usize, usize) {
        let done = self
            .items
            .iter()
            .filter(|i| i.status == "done")
            .count();
        (done, self.items.len())
    }
}

/// Parse track items from a markdown table in the body.
///
/// Expected format:
/// ```text
/// | ID  | Task                    | Status      | Spec      |
/// |-----|-------------------------|-------------|-----------|
/// | M1  | Auth & user management  | done        | auth      |
/// ```
fn parse_track_table(body: &str) -> Vec<TrackItem> {
    let mut items = Vec::new();
    let mut in_table = false;
    let mut past_separator = false;

    for line in body.lines() {
        let trimmed = line.trim();

        if !in_table {
            // Look for table header: must have pipe-separated columns with ID, Task, Status, Spec
            if trimmed.starts_with('|') && trimmed.contains("ID") && trimmed.contains("Status") {
                in_table = true;
                continue;
            }
            continue;
        }

        // Skip separator row (|---|---|...)
        if !past_separator {
            if trimmed.starts_with('|') && trimmed.contains("---") {
                past_separator = true;
            }
            continue;
        }

        // Parse data rows
        if !trimmed.starts_with('|') {
            break; // end of table
        }

        let cols: Vec<&str> = trimmed
            .split('|')
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .collect();

        if cols.len() >= 4 {
            items.push(TrackItem {
                id: cols[0].to_string(),
                task: cols[1].to_string(),
                status: cols[2].to_string(),
                spec: cols[3].to_string(),
            });
        }
    }

    items
}

pub fn parse_track(content: &str) -> Result<TrackFile> {
    let (yaml, body) = frontmatter::split_frontmatter(content)?;
    let fm: TrackFrontmatter = serde_yaml::from_str(yaml)?;
    let items = parse_track_table(body);

    Ok(TrackFile {
        frontmatter: fm,
        body: body.to_string(),
        items,
    })
}

pub fn load_track(path: &Path) -> Result<TrackFile> {
    let content = fs::read_to_string(path).map_err(|e| {
        ForjaError::Io(e)
    })?;
    parse_track(&content)
}

pub fn discover_tracks(dir: &Path) -> Result<Vec<TrackFile>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut tracks = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "md")
            && let Ok(track) = load_track(&path)
        {
            tracks.push(track);
        }
    }

    tracks.sort_by(|a, b| a.id().cmp(b.id()));
    Ok(tracks)
}

pub fn find_track(dir: &Path, track_id: &str) -> Result<TrackFile> {
    let tracks = discover_tracks(dir)?;
    tracks
        .into_iter()
        .find(|t| t.id() == track_id)
        .ok_or_else(|| ForjaError::InvalidArgument(format!("track not found: {track_id}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_TRACK: &str = r#"---
id: mvp
title: "MVP Work Track"
description: "Core product delivery"
status: in-progress
owner: "Daniel"
priority: high
created: "2026-02-18"
---
# MVP Work Track

| ID  | Task                    | Status      | Spec      |
|-----|-------------------------|-------------|-----------|
| M1  | Auth & user management  | done        | auth      |
| M2  | API endpoints           | in-progress | api       |
| M3  | Dashboard UI            | todo        | dashboard |
"#;

    #[test]
    fn parse_full_track() {
        let track = parse_track(FULL_TRACK).unwrap();
        assert_eq!(track.id(), "mvp");
        assert_eq!(track.title(), "MVP Work Track");
        assert_eq!(track.frontmatter.status, TrackStatus::InProgress);
        assert_eq!(track.frontmatter.owner.as_deref(), Some("Daniel"));
        assert_eq!(track.frontmatter.priority, Some(TrackPriority::High));
        assert_eq!(track.items.len(), 3);

        assert_eq!(track.items[0].id, "M1");
        assert_eq!(track.items[0].task, "Auth & user management");
        assert_eq!(track.items[0].status, "done");
        assert_eq!(track.items[0].spec, "auth");

        assert_eq!(track.items[1].id, "M2");
        assert_eq!(track.items[1].status, "in-progress");
    }

    #[test]
    fn progress_counts_done() {
        let track = parse_track(FULL_TRACK).unwrap();
        let (done, total) = track.progress();
        assert_eq!(done, 1);
        assert_eq!(total, 3);
    }

    #[test]
    fn parse_minimal_track() {
        let content = r#"---
id: quick
title: Quick Track
description: A fast track
status: draft
created: "2026-02-18"
---
No table here.
"#;
        let track = parse_track(content).unwrap();
        assert_eq!(track.id(), "quick");
        assert!(track.items.is_empty());
    }

    #[test]
    fn parse_track_status_variants() {
        for (input, expected) in [
            ("draft", TrackStatus::Draft),
            ("in-progress", TrackStatus::InProgress),
            ("complete", TrackStatus::Complete),
            ("archived", TrackStatus::Archived),
        ] {
            let content = format!(
                "---\nid: t\ntitle: T\ndescription: D\nstatus: {input}\ncreated: \"2026-01-01\"\n---\n"
            );
            let track = parse_track(&content).unwrap();
            assert_eq!(track.frontmatter.status, expected);
        }
    }

    #[test]
    fn parse_fails_missing_required() {
        let content = "---\nid: t\n---\n";
        let err = parse_track(content).unwrap_err();
        assert!(matches!(err, ForjaError::Yaml(_)));
    }

    #[test]
    fn discover_tracks_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("track-mvp.md"), FULL_TRACK).unwrap();
        fs::write(
            dir.path().join("track-v2.md"),
            "---\nid: v2\ntitle: V2\ndescription: V2 track\nstatus: draft\ncreated: \"2026-01-01\"\n---\n",
        )
        .unwrap();
        fs::write(dir.path().join("not-a-track.txt"), "ignored").unwrap();

        let tracks = discover_tracks(dir.path()).unwrap();
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].id(), "mvp");
        assert_eq!(tracks[1].id(), "v2");
    }

    #[test]
    fn discover_tracks_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let tracks = discover_tracks(dir.path()).unwrap();
        assert!(tracks.is_empty());
    }

    #[test]
    fn discover_tracks_missing_dir() {
        let tracks = discover_tracks(Path::new("/nonexistent")).unwrap();
        assert!(tracks.is_empty());
    }

    #[test]
    fn find_track_by_id() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("track-mvp.md"), FULL_TRACK).unwrap();

        let track = find_track(dir.path(), "mvp").unwrap();
        assert_eq!(track.id(), "mvp");
    }

    #[test]
    fn find_track_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let err = find_track(dir.path(), "missing").unwrap_err();
        assert!(matches!(err, ForjaError::InvalidArgument(_)));
    }

    #[test]
    fn track_status_as_str() {
        assert_eq!(TrackStatus::Draft.as_str(), "draft");
        assert_eq!(TrackStatus::InProgress.as_str(), "in-progress");
        assert_eq!(TrackStatus::Complete.as_str(), "complete");
        assert_eq!(TrackStatus::Archived.as_str(), "archived");
    }
}
