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
        assert_eq!(loaded.agents[0].skill_id, "code/typescript/feature");
        assert_eq!(loaded.stack.as_ref().unwrap().language, "TypeScript");
    }

    #[test]
    fn plan_status_serializes_lowercase() {
        let plan = sample_plan("test", PlanStatus::Pending);
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains(r#""status":"pending"#));

        let plan = sample_plan("test", PlanStatus::Executed);
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains(r#""status":"executed"#));
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
    fn find_latest_pending_skips_executed() {
        let dir = TempDir::new().unwrap();

        let executed = sample_plan("20260208-143022-done", PlanStatus::Executed);
        save_plan(&dir.path().join("20260208-143022-done.json"), &executed).unwrap();

        let pending = sample_plan("20260201-100000-waiting", PlanStatus::Pending);
        save_plan(&dir.path().join("20260201-100000-waiting.json"), &pending).unwrap();

        let found = find_latest_pending(dir.path()).unwrap();
        assert_eq!(found.id, "20260201-100000-waiting");
    }

    #[test]
    fn find_latest_pending_empty_dir_errors() {
        let dir = TempDir::new().unwrap();
        assert!(find_latest_pending(dir.path()).is_err());
    }

    #[test]
    fn find_latest_pending_nonexistent_dir_errors() {
        let path = std::path::PathBuf::from("/tmp/forja_nonexistent_plans_dir");
        assert!(find_latest_pending(&path).is_err());
    }

    #[test]
    fn plan_without_stack_serializes() {
        let mut plan = sample_plan("no-stack", PlanStatus::Pending);
        plan.stack = None;

        let json = serde_json::to_string(&plan).unwrap();
        assert!(!json.contains("\"stack\""));

        let loaded: PlanMetadata = serde_json::from_str(&json).unwrap();
        assert!(loaded.stack.is_none());
    }

    #[test]
    fn plan_with_phases_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("phases-plan.json");

        let mut plan = sample_plan("phases-test", PlanStatus::Pending);
        plan.phases = vec![
            PlanPhase {
                name: "Database schema".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec!["migrations/001_users.sql".to_string()],
                files_to_modify: vec![],
                instructions: "Create users table with JWT fields".to_string(),
                depends_on: vec![],
            },
            PlanPhase {
                name: "Auth middleware".to_string(),
                agent_role: "coder".to_string(),
                files_to_create: vec!["src/middleware/auth.ts".to_string()],
                files_to_modify: vec!["src/app.ts".to_string()],
                instructions: "Add JWT validation middleware".to_string(),
                depends_on: vec!["Database schema".to_string()],
            },
        ];

        save_plan(&path, &plan).unwrap();
        let loaded = load_plan(&path).unwrap();

        assert_eq!(loaded.phases.len(), 2);
        assert_eq!(loaded.phases[0].name, "Database schema");
        assert_eq!(
            loaded.phases[0].files_to_create,
            vec!["migrations/001_users.sql"]
        );
        assert!(loaded.phases[0].files_to_modify.is_empty());
        assert_eq!(loaded.phases[1].depends_on, vec!["Database schema"]);
        assert_eq!(loaded.phases[1].agent_role, "coder");
    }

    #[test]
    fn plan_with_quality_gates_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("gates-plan.json");

        let mut plan = sample_plan("gates-test", PlanStatus::Pending);
        plan.quality_gates = vec![
            "All tests must pass".to_string(),
            "No TypeScript errors".to_string(),
            "Security audit passes".to_string(),
        ];

        save_plan(&path, &plan).unwrap();
        let loaded = load_plan(&path).unwrap();

        assert_eq!(loaded.quality_gates.len(), 3);
        assert_eq!(loaded.quality_gates[0], "All tests must pass");
        assert_eq!(loaded.quality_gates[2], "Security audit passes");
    }

    #[test]
    fn old_plan_without_new_fields_deserializes() {
        let json = r#"{
            "id": "old-plan",
            "created": "2026-01-01T00:00:00Z",
            "status": "pending",
            "task": "Legacy task",
            "team_size": "quick-fix",
            "profile": "balanced",
            "agents": [
                { "skill_id": "code/general/feature", "role": "coder" }
            ],
            "stack": { "language": "JavaScript", "framework": null }
        }"#;

        let plan: PlanMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(plan.id, "old-plan");
        assert!(plan.quality_gates.is_empty());
        assert!(plan.phases.is_empty());
    }
}
