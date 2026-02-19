use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{ForjaError, Result};
use crate::frontmatter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded,
}

impl DecisionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Deprecated => "deprecated",
            Self::Superseded => "superseded",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionFrontmatter {
    pub id: String,
    pub title: String,
    pub status: DecisionStatus,
    pub date: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_specs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionFile {
    #[serde(flatten)]
    pub frontmatter: DecisionFrontmatter,
    pub body: String,
}

impl DecisionFile {
    pub fn id(&self) -> &str {
        &self.frontmatter.id
    }

    pub fn title(&self) -> &str {
        &self.frontmatter.title
    }
}

pub fn parse_decision(content: &str) -> Result<DecisionFile> {
    let (yaml, body) = frontmatter::split_frontmatter(content)?;
    let fm: DecisionFrontmatter = serde_yaml::from_str(yaml)?;

    Ok(DecisionFile {
        frontmatter: fm,
        body: body.to_string(),
    })
}

pub fn load_decision(path: &Path) -> Result<DecisionFile> {
    let content = fs::read_to_string(path).map_err(|e| {
        ForjaError::Io(e)
    })?;
    parse_decision(&content)
}

pub fn discover_decisions(dir: &Path) -> Result<Vec<DecisionFile>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut decisions = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "md")
            && let Ok(decision) = load_decision(&path)
        {
            decisions.push(decision);
        }
    }

    decisions.sort_by(|a, b| a.id().cmp(b.id()));
    Ok(decisions)
}

pub fn find_decision(dir: &Path, decision_id: &str) -> Result<DecisionFile> {
    let decisions = discover_decisions(dir)?;
    decisions
        .into_iter()
        .find(|d| d.id() == decision_id)
        .ok_or_else(|| ForjaError::InvalidArgument(format!("decision not found: {decision_id}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_DECISION: &str = r#"---
id: "001"
title: "Authentication Strategy"
status: accepted
date: "2026-02-18"
related_specs:
  - auth
superseded_by: null
---
## Context
We need a secure authentication strategy.

## Decision
Use JWT tokens in httpOnly cookies.

## Consequences
- Secure against XSS token theft
- Requires CSRF protection
"#;

    #[test]
    fn parse_full_decision() {
        let decision = parse_decision(FULL_DECISION).unwrap();
        assert_eq!(decision.id(), "001");
        assert_eq!(decision.title(), "Authentication Strategy");
        assert_eq!(decision.frontmatter.status, DecisionStatus::Accepted);
        assert_eq!(decision.frontmatter.date, "2026-02-18");
        assert_eq!(decision.frontmatter.related_specs, vec!["auth"]);
        assert!(decision.frontmatter.superseded_by.is_none());
        assert!(decision.body.contains("Context"));
        assert!(decision.body.contains("JWT tokens"));
    }

    #[test]
    fn parse_minimal_decision() {
        let content = r#"---
id: "002"
title: "Database Choice"
status: proposed
date: "2026-02-18"
---
Use PostgreSQL.
"#;
        let decision = parse_decision(content).unwrap();
        assert_eq!(decision.id(), "002");
        assert_eq!(decision.frontmatter.status, DecisionStatus::Proposed);
        assert!(decision.frontmatter.related_specs.is_empty());
    }

    #[test]
    fn parse_superseded_decision() {
        let content = r#"---
id: "001"
title: "Old Strategy"
status: superseded
date: "2026-01-01"
superseded_by: "003"
---
Old approach.
"#;
        let decision = parse_decision(content).unwrap();
        assert_eq!(decision.frontmatter.status, DecisionStatus::Superseded);
        assert_eq!(decision.frontmatter.superseded_by.as_deref(), Some("003"));
    }

    #[test]
    fn decision_status_variants() {
        for (input, expected) in [
            ("proposed", DecisionStatus::Proposed),
            ("accepted", DecisionStatus::Accepted),
            ("deprecated", DecisionStatus::Deprecated),
            ("superseded", DecisionStatus::Superseded),
        ] {
            let content = format!(
                "---\nid: t\ntitle: T\nstatus: {input}\ndate: \"2026-01-01\"\n---\n"
            );
            let decision = parse_decision(&content).unwrap();
            assert_eq!(decision.frontmatter.status, expected);
        }
    }

    #[test]
    fn decision_status_as_str() {
        assert_eq!(DecisionStatus::Proposed.as_str(), "proposed");
        assert_eq!(DecisionStatus::Accepted.as_str(), "accepted");
        assert_eq!(DecisionStatus::Deprecated.as_str(), "deprecated");
        assert_eq!(DecisionStatus::Superseded.as_str(), "superseded");
    }

    #[test]
    fn parse_fails_missing_required() {
        let content = "---\nid: t\n---\n";
        let err = parse_decision(content).unwrap_err();
        assert!(matches!(err, ForjaError::Yaml(_)));
    }

    #[test]
    fn discover_decisions_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("001-auth.md"), FULL_DECISION).unwrap();
        fs::write(
            dir.path().join("002-db.md"),
            "---\nid: \"002\"\ntitle: DB\nstatus: proposed\ndate: \"2026-01-01\"\n---\nBody.",
        )
        .unwrap();

        let decisions = discover_decisions(dir.path()).unwrap();
        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0].id(), "001");
        assert_eq!(decisions[1].id(), "002");
    }

    #[test]
    fn discover_decisions_missing_dir() {
        let decisions = discover_decisions(Path::new("/nonexistent")).unwrap();
        assert!(decisions.is_empty());
    }

    #[test]
    fn find_decision_by_id() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("001-auth.md"), FULL_DECISION).unwrap();

        let decision = find_decision(dir.path(), "001").unwrap();
        assert_eq!(decision.id(), "001");
    }

    #[test]
    fn find_decision_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let err = find_decision(dir.path(), "999").unwrap_err();
        assert!(matches!(err, ForjaError::InvalidArgument(_)));
    }
}
