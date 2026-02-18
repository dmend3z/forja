use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::phase::Phase;

/// A skill entry indexed from the registry catalog.
/// Built by scanning skills/<phase>/<tech>/<skill-name>/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub phase: Phase,
    pub tech: String,
    pub path: PathBuf,
    pub installed: bool,
    pub content_types: Vec<ContentType>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Skill,
    Agent,
    Command,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ContentType::Skill => "skill",
            ContentType::Agent => "agent",
            ContentType::Command => "command",
        };
        write!(f, "{s}")
    }
}
