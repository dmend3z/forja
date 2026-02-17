use crate::error::{ForjaError, Result};
use crate::models::agent_file::AgentFrontmatter;

/// Split content into YAML frontmatter and markdown body.
///
/// Returns `(yaml_str, body_str)` or an error if no frontmatter is found.
pub fn split_frontmatter(content: &str) -> Result<(&str, &str)> {
    let stripped = content
        .strip_prefix("---")
        .ok_or_else(|| ForjaError::InvalidSpec("missing opening --- delimiter".into()))?;

    let end = stripped
        .find("---")
        .ok_or_else(|| ForjaError::InvalidSpec("missing closing --- delimiter".into()))?;

    let yaml = stripped[..end].trim();
    let body = stripped[end + 3..].trim_start();

    Ok((yaml, body))
}

/// Parse an agent `.md` file: extract YAML frontmatter into `AgentFrontmatter` and return the body.
pub fn parse_agent_frontmatter(content: &str) -> Result<(AgentFrontmatter, String)> {
    let (yaml, body) = split_frontmatter(content)?;
    let frontmatter: AgentFrontmatter = serde_yaml::from_str(yaml)?;
    Ok((frontmatter, body.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_basic_frontmatter() {
        let content = "---\nname: test\n---\nBody here.";
        let (yaml, body) = split_frontmatter(content).unwrap();
        assert_eq!(yaml, "name: test");
        assert_eq!(body, "Body here.");
    }

    #[test]
    fn split_trims_whitespace() {
        let content = "---\n  key: value  \n---\n\n  Body with leading spaces.";
        let (yaml, body) = split_frontmatter(content).unwrap();
        assert_eq!(yaml, "key: value");
        assert_eq!(body, "Body with leading spaces.");
    }

    #[test]
    fn split_fails_without_opening() {
        let err = split_frontmatter("no frontmatter").unwrap_err();
        assert!(err.to_string().contains("missing opening ---"));
    }

    #[test]
    fn split_fails_without_closing() {
        let err = split_frontmatter("---\nno closing").unwrap_err();
        assert!(err.to_string().contains("missing closing ---"));
    }

    #[test]
    fn parse_agent_frontmatter_basic() {
        let content = "---\nname: my-agent\ndescription: Does stuff\ntools: Bash, Read\nmodel: sonnet\n---\n\n# Instructions\n\nDo the thing.";
        let (fm, body) = parse_agent_frontmatter(content).unwrap();
        assert_eq!(fm.name, "my-agent");
        assert_eq!(fm.description.as_deref(), Some("Does stuff"));
        assert_eq!(fm.tools.as_deref(), Some("Bash, Read"));
        assert_eq!(fm.model.as_deref(), Some("sonnet"));
        assert!(body.starts_with("# Instructions"));
    }

    #[test]
    fn parse_agent_frontmatter_minimal() {
        let content = "---\nname: minimal\n---\nBody.";
        let (fm, _body) = parse_agent_frontmatter(content).unwrap();
        assert_eq!(fm.name, "minimal");
        assert!(fm.description.is_none());
        assert!(fm.tools.is_none());
        assert!(fm.model.is_none());
    }
}
