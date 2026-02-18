use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The five workflow phases that organize the skill catalog, plus a Teams meta-phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Research,
    Code,
    Test,
    Review,
    Deploy,
    Teams,
}

impl Phase {
    pub fn all() -> &'static [Phase] {
        &[
            Phase::Research,
            Phase::Code,
            Phase::Test,
            Phase::Review,
            Phase::Deploy,
            Phase::Teams,
        ]
    }

    pub fn description(&self) -> &'static str {
        match self {
            Phase::Research => "Explore codebase, research docs, plan architecture",
            Phase::Code => "Write code with tech-specific best practices",
            Phase::Test => "TDD, E2E testing, coverage analysis",
            Phase::Review => "Code quality, security, performance, PR workflow",
            Phase::Deploy => "Git, CI/CD, deployment automation",
            Phase::Teams => "Multi-agent team configurations",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Phase::Research => "research",
            Phase::Code => "code",
            Phase::Test => "test",
            Phase::Review => "review",
            Phase::Deploy => "deploy",
            Phase::Teams => "teams",
        }
    }

    pub fn is_thinking_phase(&self) -> bool {
        matches!(self, Phase::Research | Phase::Review)
    }
}

impl FromStr for Phase {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "research" => Ok(Phase::Research),
            "code" => Ok(Phase::Code),
            "test" => Ok(Phase::Test),
            "review" => Ok(Phase::Review),
            "deploy" => Ok(Phase::Deploy),
            "teams" => Ok(Phase::Teams),
            _ => Err(format!("unknown phase: {s}")),
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
