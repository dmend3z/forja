use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{ForjaError, Result};
use crate::frontmatter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Running,
    Complete,
    Failed,
}

impl RunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunLogFrontmatter {
    pub spec_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<String>,
    pub agent: String,
    pub status: RunStatus,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunLog {
    #[serde(flatten)]
    pub frontmatter: RunLogFrontmatter,
    pub body: String,
    /// Derived from filename, not from frontmatter.
    #[serde(skip)]
    pub filename: String,
}

impl RunLog {
    pub fn spec_id(&self) -> &str {
        &self.frontmatter.spec_id
    }
}

pub fn parse_run_log(content: &str) -> Result<RunLog> {
    let (yaml, body) = frontmatter::split_frontmatter(content)?;
    let fm: RunLogFrontmatter = serde_yaml::from_str(yaml)?;

    Ok(RunLog {
        frontmatter: fm,
        body: body.to_string(),
        filename: String::new(),
    })
}

pub fn load_run_log(path: &Path) -> Result<RunLog> {
    let content = fs::read_to_string(path).map_err(ForjaError::Io)?;
    let mut run = parse_run_log(&content)?;
    run.filename = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(run)
}

pub fn discover_runs(dir: &Path) -> Result<Vec<RunLog>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut runs = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "md")
            && let Ok(run) = load_run_log(&path)
        {
            runs.push(run);
        }
    }

    // Sort by filename descending (newest first, since filenames are timestamped)
    runs.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(runs)
}

/// Create a new run log file and return the path.
pub fn create_run_log(
    runs_dir: &Path,
    spec_id: &str,
    plan_id: Option<&str>,
    agent: &str,
) -> Result<std::path::PathBuf> {
    fs::create_dir_all(runs_dir)?;

    let now = chrono::Utc::now();
    let timestamp = now.format("%Y%m%d-%H%M%S");
    let filename = format!("run-{timestamp}-{spec_id}.md");
    let path = runs_dir.join(&filename);

    let plan_line = match plan_id {
        Some(id) => format!("plan_id: {id}\n"),
        None => String::new(),
    };

    let content = format!(
        "---\nspec_id: {spec_id}\n{plan_line}agent: {agent}\nstatus: running\nstarted_at: \"{}\"\n---\n",
        now.to_rfc3339()
    );

    fs::write(&path, content)?;
    Ok(path)
}

/// Update a run log with completion status and output.
pub fn complete_run_log(
    path: &Path,
    status: RunStatus,
    exit_code: Option<i32>,
    output: &str,
) -> Result<()> {
    let mut run = load_run_log(path)?;
    let now = chrono::Utc::now();

    run.frontmatter.status = status;
    run.frontmatter.completed_at = Some(now.to_rfc3339());
    run.frontmatter.exit_code = exit_code;

    // Calculate duration
    if let Ok(started) = chrono::DateTime::parse_from_rfc3339(&run.frontmatter.started_at) {
        let duration = now.signed_duration_since(started);
        run.frontmatter.duration_seconds = Some(duration.num_seconds().unsigned_abs());
    }

    // Serialize frontmatter back to YAML
    let yaml = serde_yaml::to_string(&run.frontmatter)?;
    let content = format!("---\n{yaml}---\n{output}");
    fs::write(path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_RUN: &str = r#"---
spec_id: auth
plan_id: plan-auth
agent: claude-code
status: complete
started_at: "2026-02-18T14:30:22Z"
completed_at: "2026-02-18T14:45:10Z"
duration_seconds: 888
exit_code: 0
---
Agent output goes here.
Successfully implemented authentication.
"#;

    #[test]
    fn parse_full_run_log() {
        let run = parse_run_log(FULL_RUN).unwrap();
        assert_eq!(run.spec_id(), "auth");
        assert_eq!(run.frontmatter.plan_id.as_deref(), Some("plan-auth"));
        assert_eq!(run.frontmatter.agent, "claude-code");
        assert_eq!(run.frontmatter.status, RunStatus::Complete);
        assert_eq!(run.frontmatter.duration_seconds, Some(888));
        assert_eq!(run.frontmatter.exit_code, Some(0));
        assert!(run.body.contains("Successfully implemented"));
    }

    #[test]
    fn parse_minimal_run_log() {
        let content = r#"---
spec_id: api
agent: claude-code
status: running
started_at: "2026-02-18T14:30:00Z"
---
"#;
        let run = parse_run_log(content).unwrap();
        assert_eq!(run.spec_id(), "api");
        assert!(run.frontmatter.plan_id.is_none());
        assert_eq!(run.frontmatter.status, RunStatus::Running);
        assert!(run.frontmatter.completed_at.is_none());
    }

    #[test]
    fn run_status_variants() {
        for (input, expected) in [
            ("running", RunStatus::Running),
            ("complete", RunStatus::Complete),
            ("failed", RunStatus::Failed),
        ] {
            let content = format!(
                "---\nspec_id: t\nagent: a\nstatus: {input}\nstarted_at: \"2026-01-01T00:00:00Z\"\n---\n"
            );
            let run = parse_run_log(&content).unwrap();
            assert_eq!(run.frontmatter.status, expected);
        }
    }

    #[test]
    fn run_status_as_str() {
        assert_eq!(RunStatus::Running.as_str(), "running");
        assert_eq!(RunStatus::Complete.as_str(), "complete");
        assert_eq!(RunStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn create_and_load_run_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_run_log(dir.path(), "auth", Some("plan-auth"), "claude-code").unwrap();

        assert!(path.exists());
        let run = load_run_log(&path).unwrap();
        assert_eq!(run.spec_id(), "auth");
        assert_eq!(run.frontmatter.agent, "claude-code");
        assert_eq!(run.frontmatter.status, RunStatus::Running);
    }

    #[test]
    fn create_run_log_without_plan() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_run_log(dir.path(), "api", None, "claude-code").unwrap();
        let run = load_run_log(&path).unwrap();
        assert!(run.frontmatter.plan_id.is_none());
    }

    #[test]
    fn complete_run_log_updates_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_run_log(dir.path(), "auth", None, "claude-code").unwrap();

        complete_run_log(&path, RunStatus::Complete, Some(0), "All done.").unwrap();

        let run = load_run_log(&path).unwrap();
        assert_eq!(run.frontmatter.status, RunStatus::Complete);
        assert!(run.frontmatter.completed_at.is_some());
        assert_eq!(run.frontmatter.exit_code, Some(0));
        assert!(run.body.contains("All done."));
    }

    #[test]
    fn discover_runs_from_dir() {
        let dir = tempfile::tempdir().unwrap();

        // Create two run logs
        let content1 = "---\nspec_id: auth\nagent: a\nstatus: complete\nstarted_at: \"2026-02-18T14:30:00Z\"\n---\nOutput 1";
        let content2 = "---\nspec_id: api\nagent: a\nstatus: running\nstarted_at: \"2026-02-18T15:00:00Z\"\n---\nOutput 2";
        fs::write(dir.path().join("run-20260218-143000-auth.md"), content1).unwrap();
        fs::write(dir.path().join("run-20260218-150000-api.md"), content2).unwrap();

        let runs = discover_runs(dir.path()).unwrap();
        assert_eq!(runs.len(), 2);
        // Sorted newest first
        assert_eq!(runs[0].spec_id(), "api");
        assert_eq!(runs[1].spec_id(), "auth");
    }

    #[test]
    fn discover_runs_missing_dir() {
        let runs = discover_runs(Path::new("/nonexistent")).unwrap();
        assert!(runs.is_empty());
    }
}
