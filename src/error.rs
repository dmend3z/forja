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

impl ForjaError {
    /// Return an actionable hint for the user.
    pub fn hint(&self) -> &str {
        match self {
            Self::Io(_) => "Check file permissions and disk space",
            Self::Json(_) => {
                "Verify the JSON file is valid (try: cat <file> | python3 -m json.tool)"
            }
            Self::NoHomeDir => "Set the $HOME environment variable",
            Self::NotInitialized => "Run: forja init",
            Self::SkillNotFound(_) => "Try: forja search <query> or forja list --available",
            Self::AlreadyInstalled(_) => "Use: forja uninstall <skill> first, then reinstall",
            Self::NotInstalled(_) => "Install it first: forja install <skill>",
            Self::Git(_) => "Check your network connection and git installation",
            Self::TeamNotFound(_) => "List available teams: forja team list",
            Self::TeamAlreadyExists(_) => {
                "Delete the existing team first: forja team delete <name>"
            }
            Self::InvalidSettings(_) => "Check ~/.claude/settings.json for syntax errors",
            Self::PromptCancelled => "Re-run the command to try again",
            Self::Dialoguer(_) => "Try running in a standard terminal emulator",
            Self::NoPlansFound => "Create a plan first: forja plan \"your task\"",
            Self::PlanNotFound(_) => "List plans with: forja execute (interactive picker)",
            Self::ClaudeCliNotFound => "Install Claude Code from https://claude.ai/download",
        }
    }

    /// Return a specific exit code for this error category.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::NotInitialized => 2,
            Self::SkillNotFound(_) | Self::TeamNotFound(_) | Self::PlanNotFound(_) => 3,
            Self::Io(_) | Self::Json(_) => 4,
            _ => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, ForjaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_has_hint() {
        let variants: Vec<ForjaError> = vec![
            ForjaError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test")),
            ForjaError::Json(serde_json::from_str::<String>("invalid").unwrap_err()),
            ForjaError::NoHomeDir,
            ForjaError::NotInitialized,
            ForjaError::SkillNotFound("test".into()),
            ForjaError::AlreadyInstalled("test".into()),
            ForjaError::NotInstalled("test".into()),
            ForjaError::Git("test".into()),
            ForjaError::TeamNotFound("test".into()),
            ForjaError::TeamAlreadyExists("test".into()),
            ForjaError::InvalidSettings("test".into()),
            ForjaError::PromptCancelled,
            ForjaError::Dialoguer("test".into()),
            ForjaError::NoPlansFound,
            ForjaError::PlanNotFound("test".into()),
            ForjaError::ClaudeCliNotFound,
        ];

        for variant in &variants {
            let hint = variant.hint();
            assert!(!hint.is_empty(), "Empty hint for: {variant}");
        }
    }

    #[test]
    fn hints_contain_relevant_commands() {
        assert!(
            ForjaError::SkillNotFound("x".into())
                .hint()
                .contains("forja search")
        );
        assert!(ForjaError::NotInitialized.hint().contains("forja init"));
        assert!(
            ForjaError::NotInstalled("x".into())
                .hint()
                .contains("forja install")
        );
        assert!(
            ForjaError::TeamNotFound("x".into())
                .hint()
                .contains("forja team list")
        );
        assert!(ForjaError::NoPlansFound.hint().contains("forja plan"));
    }

    #[test]
    fn display_still_works() {
        let err = ForjaError::SkillNotFound("test/skill".into());
        let display = format!("{err}");
        assert!(display.contains("test/skill"));
        assert!(display.contains("Skill not found"));
    }

    #[test]
    fn exit_codes_correct() {
        assert_eq!(ForjaError::NotInitialized.exit_code(), 2);
        assert_eq!(ForjaError::SkillNotFound("x".into()).exit_code(), 3);
        assert_eq!(ForjaError::TeamNotFound("x".into()).exit_code(), 3);
        assert_eq!(ForjaError::PlanNotFound("x".into()).exit_code(), 3);
        assert_eq!(
            ForjaError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "")).exit_code(),
            4
        );
        assert_eq!(
            ForjaError::Json(serde_json::from_str::<String>("bad").unwrap_err()).exit_code(),
            4
        );
        assert_eq!(ForjaError::NoHomeDir.exit_code(), 1);
        assert_eq!(ForjaError::PromptCancelled.exit_code(), 1);
    }
}
