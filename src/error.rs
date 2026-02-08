use thiserror::Error;

/// All error types returned by forja CLI operations.
#[derive(Debug, Error)]
pub enum ForjaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Could not determine home directory")]
    NoHomeDir,

    #[error("forja is not initialized. Run `forja init` first")]
    NotInitialized,

    #[error(
        "Skill not found: {0}. Run 'forja search <query>' or 'forja list --available' to browse"
    )]
    SkillNotFound(String),

    #[error("Skill already installed: {0}")]
    AlreadyInstalled(String),

    #[error("Skill not installed: {0}")]
    NotInstalled(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Team not found: {0}")]
    TeamNotFound(String),

    #[error("Team already exists: {0}")]
    TeamAlreadyExists(String),

    #[error("Invalid settings.json: {0}")]
    InvalidSettings(String),

    #[error("Prompt cancelled by user")]
    PromptCancelled,

    #[error("Prompt error: {0}")]
    Dialoguer(String),

    #[error("No pending plans found. Run /forja-plan in Claude Code first")]
    NoPlansFound,

    #[error("Plan not found: {0}")]
    PlanNotFound(String),

    #[error("Claude Code CLI not found. Install it from https://claude.ai/download")]
    ClaudeCliNotFound,
}

pub type Result<T> = std::result::Result<T, ForjaError>;
