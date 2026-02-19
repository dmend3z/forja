use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{ForjaError, Result};
use crate::frontmatter;
use crate::models::acceptance::AcceptanceCriterion;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SpecStatus {
    #[default]
    Draft,
    Ready,
    InProgress,
    Review,
    Done,
    Blocked,
    // Legacy variants kept for backward compat with existing specs
    #[serde(alias = "planning")]
    Planning,
    #[serde(alias = "executing")]
    Executing,
    #[serde(alias = "complete")]
    Complete,
    #[serde(alias = "failed")]
    Failed,
}

impl SpecStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Ready => "ready",
            Self::InProgress => "in-progress",
            Self::Review => "review",
            Self::Done => "done",
            Self::Blocked => "blocked",
            Self::Planning => "planning",
            Self::Executing => "executing",
            Self::Complete => "complete",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFrontmatter {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SpecStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_by: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
    /// New structured acceptance criteria (supports simple strings and {description, test} objects).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// Legacy field kept for backward compatibility. Prefer `acceptance_criteria`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub success_criteria: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFile {
    #[serde(flatten)]
    pub frontmatter: SpecFrontmatter,
    pub body: String,
    /// Effective status: from frontmatter if present, otherwise Draft.
    #[serde(skip)]
    pub status: SpecStatus,
}

impl SpecFile {
    pub fn id(&self) -> &str {
        &self.frontmatter.id
    }

    pub fn title(&self) -> &str {
        &self.frontmatter.title
    }

    /// Return all acceptance criteria, merging both new `acceptance_criteria` and legacy `success_criteria`.
    pub fn all_acceptance_criteria(&self) -> Vec<&AcceptanceCriterion> {
        // If acceptance_criteria is populated, prefer it
        if !self.frontmatter.acceptance_criteria.is_empty() {
            return self.frontmatter.acceptance_criteria.iter().collect();
        }
        // Fallback: convert success_criteria to references via a collected vec
        // (can't return references to temporary AcceptanceCriterion)
        Vec::new()
    }

    /// Return legacy success_criteria strings (for backward compat display).
    pub fn success_criteria_strings(&self) -> Vec<&str> {
        if !self.frontmatter.acceptance_criteria.is_empty() {
            self.frontmatter
                .acceptance_criteria
                .iter()
                .map(|c| c.description())
                .collect()
        } else {
            self.frontmatter
                .success_criteria
                .iter()
                .map(|s| s.as_str())
                .collect()
        }
    }
}

/// Build a structured task description from a spec for use as the `$ARGUMENTS`
/// placeholder in the forja-plan template.
pub fn build_task_description(spec: &SpecFile) -> String {
    let mut desc = String::new();

    desc.push_str(&format!("{}\n\n", spec.frontmatter.title));
    desc.push_str(&format!("{}\n", spec.frontmatter.description));

    if let Some(ref priority) = spec.frontmatter.priority {
        desc.push_str(&format!("\nPriority: {priority}\n"));
    }

    if !spec.frontmatter.requirements.is_empty() {
        desc.push_str("\nRequirements:\n");
        for req in &spec.frontmatter.requirements {
            desc.push_str(&format!("- {req}\n"));
        }
    }

    if !spec.frontmatter.constraints.is_empty() {
        desc.push_str("\nConstraints:\n");
        for c in &spec.frontmatter.constraints {
            desc.push_str(&format!("- {c}\n"));
        }
    }

    // Prefer acceptance_criteria, fallback to success_criteria
    let criteria = spec.success_criteria_strings();
    if !criteria.is_empty() {
        desc.push_str("\nAcceptance Criteria:\n");
        for sc in &criteria {
            desc.push_str(&format!("- {sc}\n"));
        }
    }

    if !spec.frontmatter.depends_on.is_empty() {
        desc.push_str("\nDepends On:\n");
        for dep in &spec.frontmatter.depends_on {
            desc.push_str(&format!("- {dep}\n"));
        }
    }

    if !spec.body.is_empty() {
        desc.push_str(&format!("\nContext:\n{}\n", spec.body));
    }

    desc
}

/// Parse a spec file from its raw markdown content.
///
/// Expects YAML frontmatter between `---` delimiters, followed by a markdown body.
pub fn parse_spec(content: &str) -> Result<SpecFile> {
    let (yaml, body) = frontmatter::split_frontmatter(content)?;
    let fm: SpecFrontmatter = serde_yaml::from_str(yaml)?;

    let status = fm.status.clone().unwrap_or(SpecStatus::Draft);

    Ok(SpecFile {
        frontmatter: fm,
        body: body.to_string(),
        status,
    })
}

/// Load and parse a single spec file from disk.
pub fn load_spec(path: &Path) -> Result<SpecFile> {
    let content = fs::read_to_string(path).map_err(|e| {
        ForjaError::SpecNotFound(format!("{}: {e}", path.display()))
    })?;
    parse_spec(&content)
}

/// Discover all spec files (`.md`) in a directory.
///
/// Returns specs sorted by id. Non-`.md` files and files that fail to parse are skipped.
pub fn discover_specs(dir: &Path) -> Result<Vec<SpecFile>> {
    if !dir.exists() {
        return Err(ForjaError::SpecNotFound(format!(
            "specs directory not found: {}",
            dir.display()
        )));
    }

    let mut specs = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "md")
            && let Ok(spec) = load_spec(&path)
        {
            specs.push(spec);
        }
    }

    specs.sort_by(|a, b| a.id().cmp(b.id()));
    Ok(specs)
}

/// Discover specs from .forja/specs/ first, falling back to docs/specs/.
pub fn discover_specs_with_fallback(project_root: &Path) -> Result<Vec<SpecFile>> {
    let forja_specs = project_root.join(".forja").join("specs");
    if forja_specs.exists() {
        return discover_specs(&forja_specs);
    }

    let docs_specs = project_root.join("docs").join("specs");
    if docs_specs.exists() {
        return discover_specs(&docs_specs);
    }

    Err(ForjaError::SpecNotFound(
        "no specs directory found (.forja/specs/ or docs/specs/)".to_string(),
    ))
}

/// Find a specific spec by ID, checking .forja/specs/ first then docs/specs/.
pub fn find_spec_with_fallback(project_root: &Path, spec_id: &str) -> Result<SpecFile> {
    let specs = discover_specs_with_fallback(project_root)?;
    specs
        .into_iter()
        .find(|s| s.id() == spec_id)
        .ok_or_else(|| ForjaError::SpecNotFound(spec_id.to_string()))
}

/// Find a specific spec by ID from a directory.
pub fn find_spec(dir: &Path, spec_id: &str) -> Result<SpecFile> {
    let specs = discover_specs(dir)?;
    specs
        .into_iter()
        .find(|s| s.id() == spec_id)
        .ok_or_else(|| ForjaError::SpecNotFound(spec_id.to_string()))
}

/// Update the status field in a spec's frontmatter on disk.
pub fn update_spec_status(path: &Path, new_status: SpecStatus) -> Result<()> {
    let content = fs::read_to_string(path).map_err(|e| {
        ForjaError::SpecNotFound(format!("{}: {e}", path.display()))
    })?;

    let (yaml, body) = frontmatter::split_frontmatter(&content)?;
    let mut fm: SpecFrontmatter = serde_yaml::from_str(yaml)?;
    fm.status = Some(new_status);
    fm.updated = Some(chrono::Utc::now().format("%Y-%m-%d").to_string());

    let new_yaml = serde_yaml::to_string(&fm)?;
    let new_content = format!("---\n{new_yaml}---\n{body}");
    fs::write(path, new_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ForjaError;

    const VALID_SPEC: &str = r#"---
id: user-auth
title: Add User Authentication
description: Implement JWT-based authentication
priority: high
tags:
  - auth
  - security
requirements:
  - JWT token generation
  - Login endpoint
constraints:
  - Must use existing user table
success_criteria:
  - Users can log in and receive a token
  - Protected routes reject unauthenticated requests
---

# User Authentication

This spec describes the authentication system.

## Details

Use bcrypt for password hashing.
"#;

    const SPEC_WITH_NEW_FIELDS: &str = r#"---
id: auth
title: "User Authentication"
description: "JWT-based authentication"
status: in-progress
priority: high
track: mvp
assignee: "Daniel"
tags:
  - auth
  - security
blocked_by: []
depends_on:
  - database-setup
requirements:
  - "JWT token generation"
  - "Login endpoint"
constraints:
  - "Must use existing user table"
acceptance_criteria:
  - "Users can log in and receive a token"
  - description: "Protected routes reject unauthenticated"
    test: "cargo test auth_reject"
created: "2026-02-18"
updated: "2026-02-18"
---
# User Authentication
Context here.
"#;

    #[test]
    fn parse_valid_spec() {
        let spec = parse_spec(VALID_SPEC).unwrap();
        assert_eq!(spec.id(), "user-auth");
        assert_eq!(spec.title(), "Add User Authentication");
        assert_eq!(spec.frontmatter.description, "Implement JWT-based authentication");
        assert_eq!(spec.frontmatter.priority.as_deref(), Some("high"));
        assert_eq!(spec.frontmatter.tags, vec!["auth", "security"]);
        assert_eq!(spec.frontmatter.requirements.len(), 2);
        assert_eq!(spec.frontmatter.constraints.len(), 1);
        assert_eq!(spec.frontmatter.success_criteria.len(), 2);
        assert!(spec.body.starts_with("# User Authentication"));
        assert_eq!(spec.status, SpecStatus::Draft);
    }

    #[test]
    fn parse_spec_with_new_fields() {
        let spec = parse_spec(SPEC_WITH_NEW_FIELDS).unwrap();
        assert_eq!(spec.id(), "auth");
        assert_eq!(spec.status, SpecStatus::InProgress);
        assert_eq!(spec.frontmatter.track.as_deref(), Some("mvp"));
        assert_eq!(spec.frontmatter.assignee.as_deref(), Some("Daniel"));
        assert_eq!(spec.frontmatter.depends_on, vec!["database-setup"]);
        assert!(spec.frontmatter.blocked_by.is_empty());
        assert_eq!(spec.frontmatter.acceptance_criteria.len(), 2);
        assert_eq!(spec.frontmatter.created.as_deref(), Some("2026-02-18"));
        assert_eq!(spec.frontmatter.updated.as_deref(), Some("2026-02-18"));
    }

    #[test]
    fn acceptance_criteria_mixed_types() {
        let spec = parse_spec(SPEC_WITH_NEW_FIELDS).unwrap();
        let criteria = &spec.frontmatter.acceptance_criteria;

        assert_eq!(criteria[0].description(), "Users can log in and receive a token");
        assert!(criteria[0].test_command().is_none());

        assert_eq!(criteria[1].description(), "Protected routes reject unauthenticated");
        assert_eq!(criteria[1].test_command(), Some("cargo test auth_reject"));
    }

    #[test]
    fn success_criteria_strings_prefers_acceptance() {
        let spec = parse_spec(SPEC_WITH_NEW_FIELDS).unwrap();
        let strings = spec.success_criteria_strings();
        assert_eq!(strings.len(), 2);
        assert_eq!(strings[0], "Users can log in and receive a token");
    }

    #[test]
    fn success_criteria_strings_falls_back_to_legacy() {
        let spec = parse_spec(VALID_SPEC).unwrap();
        let strings = spec.success_criteria_strings();
        assert_eq!(strings.len(), 2);
        assert_eq!(strings[0], "Users can log in and receive a token");
    }

    #[test]
    fn status_from_frontmatter() {
        let content = "---\nid: s\ntitle: S\ndescription: D\nstatus: ready\n---\n";
        let spec = parse_spec(content).unwrap();
        assert_eq!(spec.status, SpecStatus::Ready);
    }

    #[test]
    fn status_defaults_to_draft() {
        let content = "---\nid: s\ntitle: S\ndescription: D\n---\n";
        let spec = parse_spec(content).unwrap();
        assert_eq!(spec.status, SpecStatus::Draft);
    }

    #[test]
    fn parse_minimal_spec() {
        let content = "---\nid: minimal\ntitle: Minimal\ndescription: A minimal spec\n---\n\nBody here.";
        let spec = parse_spec(content).unwrap();
        assert_eq!(spec.id(), "minimal");
        assert!(spec.frontmatter.priority.is_none());
        assert!(spec.frontmatter.tags.is_empty());
        assert!(spec.frontmatter.requirements.is_empty());
        assert!(spec.frontmatter.track.is_none());
        assert!(spec.frontmatter.assignee.is_none());
        assert!(spec.frontmatter.depends_on.is_empty());
        assert!(spec.frontmatter.acceptance_criteria.is_empty());
    }

    #[test]
    fn parse_fails_without_frontmatter() {
        let content = "# Just Markdown\n\nNo frontmatter here.";
        let err = parse_spec(content).unwrap_err();
        assert!(err.to_string().contains("missing opening ---"));
    }

    #[test]
    fn parse_fails_with_unclosed_frontmatter() {
        let content = "---\nid: broken\ntitle: Broken\n\nBody without closing delimiter.";
        let err = parse_spec(content).unwrap_err();
        assert!(err.to_string().contains("missing closing ---"));
    }

    #[test]
    fn parse_fails_with_missing_required_fields() {
        let content = "---\nid: incomplete\n---\n\nBody.";
        let err = parse_spec(content).unwrap_err();
        assert!(matches!(err, ForjaError::Yaml(_)));
    }

    #[test]
    fn parse_empty_body() {
        let content = "---\nid: empty-body\ntitle: Empty Body\ndescription: Has no body\n---\n";
        let spec = parse_spec(content).unwrap();
        assert_eq!(spec.id(), "empty-body");
        assert!(spec.body.is_empty());
    }

    #[test]
    fn spec_status_serializes_kebab() {
        let json = serde_json::to_string(&SpecStatus::InProgress).unwrap();
        assert_eq!(json, "\"in-progress\"");
    }

    #[test]
    fn spec_status_roundtrip() {
        for status in [
            SpecStatus::Draft,
            SpecStatus::Ready,
            SpecStatus::InProgress,
            SpecStatus::Review,
            SpecStatus::Done,
            SpecStatus::Blocked,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let parsed: SpecStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn legacy_status_compat() {
        // "complete" should still parse (aliased)
        let content = "---\nid: s\ntitle: S\ndescription: D\nstatus: complete\n---\n";
        let spec = parse_spec(content).unwrap();
        assert_eq!(spec.status, SpecStatus::Complete);
    }

    #[test]
    fn split_frontmatter_trims_correctly() {
        let content = "---\n  key: value  \n---\n\n  Body with leading spaces.";
        let (yaml, body) = frontmatter::split_frontmatter(content).unwrap();
        assert_eq!(yaml, "key: value");
        assert_eq!(body, "Body with leading spaces.");
    }

    fn write_spec(dir: &Path, filename: &str, id: &str, title: &str) {
        let content = format!(
            "---\nid: {id}\ntitle: {title}\ndescription: A test spec\n---\n\nBody for {id}."
        );
        fs::write(dir.join(filename), content).unwrap();
    }

    #[test]
    fn discover_specs_finds_md_files() {
        let dir = tempfile::tempdir().unwrap();
        write_spec(dir.path(), "alpha.md", "alpha", "Alpha Spec");
        write_spec(dir.path(), "beta.md", "beta", "Beta Spec");
        fs::write(dir.path().join("not-a-spec.txt"), "ignored").unwrap();

        let specs = discover_specs(dir.path()).unwrap();
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].id(), "alpha");
        assert_eq!(specs[1].id(), "beta");
    }

    #[test]
    fn discover_specs_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let specs = discover_specs(dir.path()).unwrap();
        assert!(specs.is_empty());
    }

    #[test]
    fn discover_specs_missing_dir() {
        let err = discover_specs(Path::new("/nonexistent/path")).unwrap_err();
        assert!(matches!(err, ForjaError::SpecNotFound(_)));
    }

    #[test]
    fn discover_specs_skips_malformed() {
        let dir = tempfile::tempdir().unwrap();
        write_spec(dir.path(), "good.md", "good", "Good Spec");
        fs::write(dir.path().join("bad.md"), "no frontmatter here").unwrap();

        let specs = discover_specs(dir.path()).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id(), "good");
    }

    #[test]
    fn find_spec_by_id() {
        let dir = tempfile::tempdir().unwrap();
        write_spec(dir.path(), "one.md", "one", "One");
        write_spec(dir.path(), "two.md", "two", "Two");

        let spec = find_spec(dir.path(), "two").unwrap();
        assert_eq!(spec.id(), "two");
    }

    #[test]
    fn find_spec_not_found() {
        let dir = tempfile::tempdir().unwrap();
        write_spec(dir.path(), "one.md", "one", "One");

        let err = find_spec(dir.path(), "missing").unwrap_err();
        assert!(matches!(err, ForjaError::SpecNotFound(_)));
    }

    #[test]
    fn load_spec_from_file() {
        let dir = tempfile::tempdir().unwrap();
        write_spec(dir.path(), "test.md", "test-id", "Test Title");

        let spec = load_spec(&dir.path().join("test.md")).unwrap();
        assert_eq!(spec.id(), "test-id");
        assert_eq!(spec.title(), "Test Title");
    }

    #[test]
    fn load_spec_missing_file() {
        let err = load_spec(Path::new("/nonexistent/file.md")).unwrap_err();
        assert!(matches!(err, ForjaError::SpecNotFound(_)));
    }

    #[test]
    fn build_task_description_full_spec() {
        let spec = parse_spec(VALID_SPEC).unwrap();
        let desc = build_task_description(&spec);

        assert!(desc.contains("Add User Authentication"));
        assert!(desc.contains("Implement JWT-based authentication"));
        assert!(desc.contains("Priority: high"));
        assert!(desc.contains("Requirements:"));
        assert!(desc.contains("- JWT token generation"));
        assert!(desc.contains("- Login endpoint"));
        assert!(desc.contains("Constraints:"));
        assert!(desc.contains("- Must use existing user table"));
        assert!(desc.contains("Acceptance Criteria:"));
        assert!(desc.contains("- Users can log in and receive a token"));
        assert!(desc.contains("Context:"));
        assert!(desc.contains("# User Authentication"));
    }

    #[test]
    fn build_task_description_minimal_spec() {
        let content = "---\nid: min\ntitle: Minimal\ndescription: A minimal spec\n---\n";
        let spec = parse_spec(content).unwrap();
        let desc = build_task_description(&spec);

        assert!(desc.contains("Minimal"));
        assert!(desc.contains("A minimal spec"));
        assert!(!desc.contains("Priority:"));
        assert!(!desc.contains("Requirements:"));
        assert!(!desc.contains("Context:"));
    }

    #[test]
    fn build_task_description_with_depends_on() {
        let content = "---\nid: d\ntitle: D\ndescription: D\ndepends_on:\n  - auth\n  - db\n---\n";
        let spec = parse_spec(content).unwrap();
        let desc = build_task_description(&spec);
        assert!(desc.contains("Depends On:"));
        assert!(desc.contains("- auth"));
        assert!(desc.contains("- db"));
    }

    #[test]
    fn parse_unicode_in_fields() {
        let content = "---\nid: café-api\ntitle: API café\ndescription: Gère les commandes\n---\n\nCorps en français.";
        let spec = parse_spec(content).unwrap();
        assert_eq!(spec.id(), "café-api");
        assert_eq!(spec.title(), "API café");
        assert!(spec.body.contains("français"));
    }

    #[test]
    fn parse_trailing_whitespace_in_values() {
        let content = "---\nid: spaced   \ntitle: Spaced Title   \ndescription: A desc   \n---\n\nBody.";
        let spec = parse_spec(content).unwrap();
        assert_eq!(spec.id(), "spaced");
        assert_eq!(spec.title(), "Spaced Title");
    }

    #[test]
    fn parse_multiline_body() {
        let content = "---\nid: multi\ntitle: Multi\ndescription: Multi body\n---\n\nLine 1\nLine 2\nLine 3";
        let spec = parse_spec(content).unwrap();
        assert_eq!(spec.body.lines().count(), 3);
    }

    #[test]
    fn discover_specs_ignores_nested_dirs() {
        let dir = tempfile::tempdir().unwrap();
        write_spec(dir.path(), "top.md", "top", "Top Spec");

        let nested = dir.path().join("nested");
        fs::create_dir_all(&nested).unwrap();
        write_spec(&nested, "deep.md", "deep", "Deep Spec");

        let specs = discover_specs(dir.path()).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id(), "top");
    }

    #[test]
    fn discover_specs_sorted_by_id() {
        let dir = tempfile::tempdir().unwrap();
        write_spec(dir.path(), "z.md", "zebra", "Zebra");
        write_spec(dir.path(), "a.md", "alpha", "Alpha");
        write_spec(dir.path(), "m.md", "middle", "Middle");

        let specs = discover_specs(dir.path()).unwrap();
        let ids: Vec<&str> = specs.iter().map(|s| s.id()).collect();
        assert_eq!(ids, vec!["alpha", "middle", "zebra"]);
    }

    #[test]
    fn find_spec_case_sensitive() {
        let dir = tempfile::tempdir().unwrap();
        write_spec(dir.path(), "test.md", "my-spec", "My Spec");

        let err = find_spec(dir.path(), "My-Spec").unwrap_err();
        assert!(matches!(err, ForjaError::SpecNotFound(_)));
    }

    #[test]
    fn spec_status_default_is_draft() {
        let content = "---\nid: s\ntitle: S\ndescription: D\n---\n";
        let spec = parse_spec(content).unwrap();
        assert_eq!(spec.status, SpecStatus::Draft);
    }

    #[test]
    fn build_task_description_no_priority() {
        let content = "---\nid: np\ntitle: No Priority\ndescription: A spec\nrequirements:\n  - one\n---\n";
        let spec = parse_spec(content).unwrap();
        let desc = build_task_description(&spec);
        assert!(!desc.contains("Priority:"));
        assert!(desc.contains("Requirements:"));
        assert!(desc.contains("- one"));
    }

    #[test]
    fn discover_specs_with_fallback_prefers_forja() {
        let dir = tempfile::tempdir().unwrap();

        // Create both directories
        let forja_specs = dir.path().join(".forja").join("specs");
        let docs_specs = dir.path().join("docs").join("specs");
        fs::create_dir_all(&forja_specs).unwrap();
        fs::create_dir_all(&docs_specs).unwrap();

        write_spec(&forja_specs, "from-forja.md", "from-forja", "From Forja");
        write_spec(&docs_specs, "from-docs.md", "from-docs", "From Docs");

        let specs = discover_specs_with_fallback(dir.path()).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id(), "from-forja");
    }

    #[test]
    fn discover_specs_with_fallback_uses_docs() {
        let dir = tempfile::tempdir().unwrap();

        let docs_specs = dir.path().join("docs").join("specs");
        fs::create_dir_all(&docs_specs).unwrap();
        write_spec(&docs_specs, "from-docs.md", "from-docs", "From Docs");

        let specs = discover_specs_with_fallback(dir.path()).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id(), "from-docs");
    }

    #[test]
    fn update_spec_status_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        fs::write(&path, "---\nid: t\ntitle: T\ndescription: D\nstatus: draft\n---\nBody.").unwrap();

        update_spec_status(&path, SpecStatus::Done).unwrap();

        let spec = load_spec(&path).unwrap();
        assert_eq!(spec.status, SpecStatus::Done);
        assert!(spec.frontmatter.updated.is_some());
    }
}
