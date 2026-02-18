use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::{ForjaError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PlanStatus {
    Pending,
    Executed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAgent {
    pub skill_id: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStack {
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanPhase {
    pub name: String,
    pub agent_role: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_to_create: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_to_modify: Vec<String>,
    pub instructions: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMetadata {
    pub id: String,
    pub created: String,
    pub status: PlanStatus,
    pub task: String,
    pub team_size: String,
    pub profile: String,
    pub agents: Vec<PlanAgent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<PlanStack>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quality_gates: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phases: Vec<PlanPhase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_spec: Option<String>,
}

pub fn load_plan(path: &Path) -> Result<PlanMetadata> {
    let content = fs::read_to_string(path)?;
    let plan: PlanMetadata = serde_json::from_str(&content)?;
    Ok(plan)
}

pub fn save_plan(path: &Path, plan: &PlanMetadata) -> Result<()> {
    let json = serde_json::to_string_pretty(plan)?;
    fs::write(path, json)?;
    Ok(())
}

/// Find the latest pending plan in the plans directory.
/// Plans are sorted by filename (which starts with YYYYMMDD-HHMMSS),
/// so the last one alphabetically is the most recent.
pub fn find_latest_pending(plans_dir: &Path) -> Result<PlanMetadata> {
    if !plans_dir.exists() {
        return Err(ForjaError::NoPlansFound);
    }

    let mut json_files: Vec<_> = fs::read_dir(plans_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();

    json_files.sort();

    // Iterate from newest to oldest, find first pending
    for path in json_files.into_iter().rev() {
        if let Ok(plan) = load_plan(&path)
            && plan.status == PlanStatus::Pending
        {
            return Ok(plan);
        }
    }

    Err(ForjaError::NoPlansFound)
}

/// Find the most recent plan linked to a spec ID via `source_spec`.
pub fn find_plan_for_spec(plans_dir: &Path, spec_id: &str) -> Result<PlanMetadata> {
    if !plans_dir.exists() {
        return Err(ForjaError::NoPlansFound);
    }

    let mut json_files: Vec<_> = fs::read_dir(plans_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();

    json_files.sort();

    for path in json_files.into_iter().rev() {
        if let Ok(plan) = load_plan(&path)
            && plan.source_spec.as_deref() == Some(spec_id)
        {
            return Ok(plan);
        }
    }

    Err(ForjaError::PlanNotFound(format!(
        "no plan found for spec '{spec_id}'"
    )))
}

// --- Execution Checkpoints ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PhaseStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseCheckpoint {
    pub phase_index: usize,
    pub phase_name: String,
    pub status: PhaseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCheckpoint {
    pub plan_id: String,
    pub started_at: String,
    pub last_updated: String,
    pub current_phase: Option<usize>,
    pub phases: Vec<PhaseCheckpoint>,
}

pub fn checkpoint_path(plans_dir: &Path, plan_id: &str) -> std::path::PathBuf {
    plans_dir.join(format!("{plan_id}.checkpoint.json"))
}

pub fn workspace_dir(plans_dir: &Path, plan_id: &str) -> std::path::PathBuf {
    plans_dir.join(format!("{plan_id}-workspace"))
}

pub fn initialize_checkpoint(plan: &PlanMetadata) -> ExecutionCheckpoint {
    let now = chrono::Utc::now().to_rfc3339();
    let phases = plan
        .phases
        .iter()
        .enumerate()
        .map(|(i, p)| PhaseCheckpoint {
            phase_index: i,
            phase_name: p.name.clone(),
            status: PhaseStatus::Pending,
            started_at: None,
            completed_at: None,
            exit_code: None,
            error_message: None,
        })
        .collect();

    ExecutionCheckpoint {
        plan_id: plan.id.clone(),
        started_at: now.clone(),
        last_updated: now,
        current_phase: None,
        phases,
    }
}

pub fn load_checkpoint(path: &Path) -> Result<ExecutionCheckpoint> {
    let content = fs::read_to_string(path)?;
    let checkpoint: ExecutionCheckpoint = serde_json::from_str(&content)?;
    Ok(checkpoint)
}

pub fn save_checkpoint(path: &Path, checkpoint: &ExecutionCheckpoint) -> Result<()> {
    let json = serde_json::to_string_pretty(checkpoint)?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_plan(id: &str, status: PlanStatus) -> PlanMetadata {
        PlanMetadata {
            id: id.to_string(),
            created: "2026-02-08T14:30:22Z".to_string(),
            status,
            task: "Add user auth".to_string(),
            team_size: "full-product".to_string(),
            profile: "balanced".to_string(),
            agents: vec![
                PlanAgent {
                    skill_id: "code/typescript/feature".to_string(),
                    role: "coder".to_string(),
                },
                PlanAgent {
                    skill_id: "test/tdd/workflow".to_string(),
                    role: "tester".to_string(),
                },
            ],
            stack: Some(PlanStack {
                language: "TypeScript".to_string(),
                framework: Some("Next.js".to_string()),
            }),
            quality_gates: vec![],
            phases: vec![],
            source_spec: None,
        }
    }

    #[test]
    fn plan_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test-plan.json");

        let plan = sample_plan("20260208-143022-user-auth", PlanStatus::Pending);
        save_plan(&path, &plan).unwrap();

        let loaded = load_plan(&path).unwrap();
        assert_eq!(loaded.id, "20260208-143022-user-auth");
        assert_eq!(loaded.status, PlanStatus::Pending);
        assert_eq!(loaded.agents.len(), 2);
    }

    #[test]
    fn find_latest_pending_returns_newest() {
        let dir = TempDir::new().unwrap();

        let old = sample_plan("20260201-100000-old-task", PlanStatus::Pending);
        save_plan(&dir.path().join("20260201-100000-old-task.json"), &old).unwrap();

        let new = sample_plan("20260208-143022-new-task", PlanStatus::Pending);
        save_plan(&dir.path().join("20260208-143022-new-task.json"), &new).unwrap();

        let found = find_latest_pending(dir.path()).unwrap();
        assert_eq!(found.id, "20260208-143022-new-task");
    }

    #[test]
    fn initialize_checkpoint_creates_all_phases_pending() {
        let mut plan = sample_plan("test-phases", PlanStatus::Pending);
        plan.phases = vec![
            PlanPhase {
                name: "Database schema".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec!["migrations/001.sql".to_string()],
                files_to_modify: vec![],
                instructions: "Create tables".to_string(),
                depends_on: vec![],
            },
            PlanPhase {
                name: "Auth middleware".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec![],
                files_to_modify: vec!["src/app.ts".to_string()],
                instructions: "Add JWT".to_string(),
                depends_on: vec!["Database schema".to_string()],
            },
        ];
        let checkpoint = initialize_checkpoint(&plan);

        assert_eq!(checkpoint.plan_id, "test-phases");
        assert_eq!(checkpoint.phases.len(), 2);
        assert!(
            checkpoint
                .phases
                .iter()
                .all(|p| p.status == PhaseStatus::Pending)
        );
    }

    #[test]
    fn find_plan_for_spec_finds_linked_plan() {
        let dir = TempDir::new().unwrap();

        let mut plan = sample_plan("20260210-120000-from-spec", PlanStatus::Pending);
        plan.source_spec = Some("user-auth".to_string());
        save_plan(
            &dir.path().join("20260210-120000-from-spec.json"),
            &plan,
        )
        .unwrap();

        let unlinked = sample_plan("20260209-100000-unlinked", PlanStatus::Pending);
        save_plan(
            &dir.path().join("20260209-100000-unlinked.json"),
            &unlinked,
        )
        .unwrap();

        let found = find_plan_for_spec(dir.path(), "user-auth").unwrap();
        assert_eq!(found.id, "20260210-120000-from-spec");
        assert_eq!(found.source_spec.as_deref(), Some("user-auth"));
    }

    #[test]
    fn find_plan_for_spec_returns_most_recent() {
        let dir = TempDir::new().unwrap();

        let mut old = sample_plan("20260201-100000-old", PlanStatus::Pending);
        old.source_spec = Some("my-spec".to_string());
        save_plan(&dir.path().join("20260201-100000-old.json"), &old).unwrap();

        let mut new = sample_plan("20260210-120000-new", PlanStatus::Pending);
        new.source_spec = Some("my-spec".to_string());
        save_plan(&dir.path().join("20260210-120000-new.json"), &new).unwrap();

        let found = find_plan_for_spec(dir.path(), "my-spec").unwrap();
        assert_eq!(found.id, "20260210-120000-new");
    }

    #[test]
    fn find_plan_for_spec_not_found() {
        let dir = TempDir::new().unwrap();

        let plan = sample_plan("20260210-120000-other", PlanStatus::Pending);
        save_plan(&dir.path().join("20260210-120000-other.json"), &plan).unwrap();

        let err = find_plan_for_spec(dir.path(), "nonexistent").unwrap_err();
        assert!(matches!(err, ForjaError::PlanNotFound(_)));
    }
}
