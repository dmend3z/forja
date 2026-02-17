use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{ForjaError, Result};
use crate::frontmatter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SpecStatus {
    Draft,
    Planning,
    Ready,
    Executing,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFrontmatter {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub success_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFile {
    #[serde(flatten)]
    pub frontmatter: SpecFrontmatter,
    pub body: String,
    #[serde(default = "default_status")]
    pub status: SpecStatus,
}

fn default_status() -> SpecStatus {
    SpecStatus::Draft
}

impl SpecFile {
    pub fn id(&self) -> &str {
        &self.frontmatter.id
    }

    pub fn title(&self) -> &str {
        &self.frontmatter.title
    }
}

/// Parse a spec file from its raw markdown content.
///
/// Expects YAML frontmatter between `---` delimiters, followed by a markdown body.
pub fn parse_spec(content: &str) -> Result<SpecFile> {
    let (yaml, body) = frontmatter::split_frontmatter(content)?;
    let fm: SpecFrontmatter = serde_yaml::from_str(yaml)?;

    Ok(SpecFile {
        frontmatter: fm,
        body: body.to_string(),
        status: SpecStatus::Draft,
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

/// Find a specific spec by ID from a directory.
pub fn find_spec(dir: &Path, spec_id: &str) -> Result<SpecFile> {
    let specs = discover_specs(dir)?;
    specs
        .into_iter()
        .find(|s| s.id() == spec_id)
        .ok_or_else(|| ForjaError::SpecNotFound(spec_id.to_string()))
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
    fn parse_minimal_spec() {
        let content = "---\nid: minimal\ntitle: Minimal\ndescription: A minimal spec\n---\n\nBody here.";
        let spec = parse_spec(content).unwrap();
        assert_eq!(spec.id(), "minimal");
        assert!(spec.frontmatter.priority.is_none());
        assert!(spec.frontmatter.tags.is_empty());
        assert!(spec.frontmatter.requirements.is_empty());
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
    fn spec_status_serializes_lowercase() {
        let json = serde_json::to_string(&SpecStatus::Planning).unwrap();
        assert_eq!(json, "\"planning\"");
    }

    #[test]
    fn spec_status_roundtrip() {
        for status in [
            SpecStatus::Draft,
            SpecStatus::Planning,
            SpecStatus::Ready,
            SpecStatus::Executing,
            SpecStatus::Complete,
            SpecStatus::Failed,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let parsed: SpecStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, status);
        }
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
}
