use crate::models::phase::Phase;

/// Generate the content for a skill.json manifest template.
pub fn skill_json(name: &str, description: &str) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "name": name,
        "description": description,
        "version": "0.1.0",
        "keywords": []
    }))
    .unwrap_or_default()
}

/// Generate the content for an agent .md file with YAML frontmatter.
pub fn agent_md(name: &str, phase: Phase) -> String {
    let model = if phase.is_thinking_phase() {
        "opus"
    } else {
        "sonnet"
    };

    format!(
        r#"---
name: {name}
description: TODO — describe what this agent does
model: {model}
---

# {name}

You are a specialized agent for TODO.

## Capabilities

- TODO: list key capabilities

## Rules

- Read CLAUDE.md before starting
- Follow existing project patterns
- Ask when requirements are ambiguous
"#
    )
}

/// Generate a basic README.md for a new skill.
pub fn readme_md(name: &str, phase: Phase, tech: &str) -> String {
    format!(
        r#"# {name}

> Phase: {phase} | Tech: {tech}

TODO: describe what this skill does and when to use it.

## Installation

```bash
forja install {phase}/{tech}/{name}
```

## Usage

TODO: add usage examples.
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_json_is_valid() {
        let json = skill_json("my-skill", "A test skill");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "my-skill");
        assert_eq!(parsed["description"], "A test skill");
        assert_eq!(parsed["version"], "0.1.0");
    }

    #[test]
    fn agent_md_thinking_phase_uses_opus() {
        let content = agent_md("researcher", Phase::Research);
        assert!(content.contains("model: opus"));
    }

    #[test]
    fn agent_md_coding_phase_uses_sonnet() {
        let content = agent_md("coder", Phase::Code);
        assert!(content.contains("model: sonnet"));
    }

    #[test]
    fn readme_contains_install_command() {
        let content = readme_md("my-skill", Phase::Code, "rust");
        assert!(content.contains("forja install code/rust/my-skill"));
    }
}
