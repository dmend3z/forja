use serde::{Deserialize, Serialize};

use super::skill::Skill;

/// Parsed YAML frontmatter from an agent `.md` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFrontmatter {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// A fully parsed agent file: frontmatter metadata + markdown body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFile {
    pub filename: String,
    pub frontmatter: AgentFrontmatter,
    pub body: String,
}

/// Full detail view of a skill including its agent files and directory listings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDetail {
    pub skill: Skill,
    pub agents: Vec<AgentFile>,
    pub skill_files: Vec<String>,
    pub command_files: Vec<String>,
}
