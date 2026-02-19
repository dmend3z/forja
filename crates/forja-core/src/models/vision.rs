use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{ForjaError, Result};
use crate::frontmatter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStack {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosting: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionFrontmatter {
    pub project: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tech_stack: Option<TechStack>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub principles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_users: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub success_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionFile {
    #[serde(flatten)]
    pub frontmatter: VisionFrontmatter,
    pub body: String,
}

impl VisionFile {
    pub fn project(&self) -> &str {
        &self.frontmatter.project
    }

    pub fn description(&self) -> &str {
        &self.frontmatter.description
    }
}

pub fn parse_vision(content: &str) -> Result<VisionFile> {
    let (yaml, body) = frontmatter::split_frontmatter(content)?;
    let fm: VisionFrontmatter = serde_yaml::from_str(yaml)?;

    Ok(VisionFile {
        frontmatter: fm,
        body: body.to_string(),
    })
}

pub fn load_vision(path: &Path) -> Result<VisionFile> {
    let content = fs::read_to_string(path).map_err(|e| {
        ForjaError::Io(e)
    })?;
    parse_vision(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_VISION: &str = r#"---
project: "My SaaS App"
description: "One-line pitch"
version: "0.1.0"
tech_stack:
  language: "TypeScript"
  framework: "Next.js"
  database: "PostgreSQL"
  hosting: "Vercel"
principles:
  - "Simplicity over features"
  - "Mobile-first experience"
target_users:
  - "Small business owners"
success_metrics:
  - "100 active users in 3 months"
---
# Product Vision

Build the best SaaS app.
"#;

    #[test]
    fn parse_full_vision() {
        let vision = parse_vision(FULL_VISION).unwrap();
        assert_eq!(vision.project(), "My SaaS App");
        assert_eq!(vision.description(), "One-line pitch");
        assert_eq!(vision.frontmatter.version.as_deref(), Some("0.1.0"));
        let stack = vision.frontmatter.tech_stack.as_ref().unwrap();
        assert_eq!(stack.language.as_deref(), Some("TypeScript"));
        assert_eq!(stack.framework.as_deref(), Some("Next.js"));
        assert_eq!(stack.database.as_deref(), Some("PostgreSQL"));
        assert_eq!(stack.hosting.as_deref(), Some("Vercel"));
        assert_eq!(vision.frontmatter.principles.len(), 2);
        assert_eq!(vision.frontmatter.target_users.len(), 1);
        assert_eq!(vision.frontmatter.success_metrics.len(), 1);
        assert!(vision.body.contains("Product Vision"));
    }

    #[test]
    fn parse_minimal_vision() {
        let content = "---\nproject: Minimal\ndescription: A test\n---\n\nBody.";
        let vision = parse_vision(content).unwrap();
        assert_eq!(vision.project(), "Minimal");
        assert_eq!(vision.description(), "A test");
        assert!(vision.frontmatter.version.is_none());
        assert!(vision.frontmatter.tech_stack.is_none());
        assert!(vision.frontmatter.principles.is_empty());
    }

    #[test]
    fn parse_fails_missing_project() {
        let content = "---\ndescription: No project\n---\n";
        let err = parse_vision(content).unwrap_err();
        assert!(matches!(err, ForjaError::Yaml(_)));
    }

    #[test]
    fn parse_fails_missing_description() {
        let content = "---\nproject: Test\n---\n";
        let err = parse_vision(content).unwrap_err();
        assert!(matches!(err, ForjaError::Yaml(_)));
    }

    #[test]
    fn load_vision_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vision.md");
        fs::write(&path, FULL_VISION).unwrap();
        let vision = load_vision(&path).unwrap();
        assert_eq!(vision.project(), "My SaaS App");
    }

    #[test]
    fn load_vision_missing_file() {
        let err = load_vision(Path::new("/nonexistent/vision.md")).unwrap_err();
        assert!(matches!(err, ForjaError::Io(_)));
    }
}
