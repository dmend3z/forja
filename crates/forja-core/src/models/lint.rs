use std::fmt;
use std::path::PathBuf;

/// Result of linting a single skill directory.
#[derive(Debug)]
pub struct LintResult {
    pub skill_path: PathBuf,
    pub skill_id: String,
    pub issues: Vec<LintIssue>,
}

impl LintResult {
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.level == LintLevel::Error)
    }

    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.level == LintLevel::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.level == LintLevel::Warning)
            .count()
    }
}

/// A single lint issue found during validation.
#[derive(Debug)]
pub struct LintIssue {
    pub level: LintLevel,
    pub rule: String,
    pub message: String,
}

/// Severity level for lint issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintLevel {
    Error,
    Warning,
}

impl fmt::Display for LintLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LintLevel::Error => write!(f, "error"),
            LintLevel::Warning => write!(f, "warning"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_result_counts() {
        let result = LintResult {
            skill_path: PathBuf::from("/test"),
            skill_id: "test/skill".to_string(),
            issues: vec![
                LintIssue {
                    level: LintLevel::Error,
                    rule: "manifest-missing".to_string(),
                    message: "No manifest found".to_string(),
                },
                LintIssue {
                    level: LintLevel::Warning,
                    rule: "readme-missing".to_string(),
                    message: "No README.md".to_string(),
                },
                LintIssue {
                    level: LintLevel::Error,
                    rule: "name-empty".to_string(),
                    message: "Name is empty".to_string(),
                },
            ],
        };

        assert!(result.has_errors());
        assert_eq!(result.error_count(), 2);
        assert_eq!(result.warning_count(), 1);
    }

    #[test]
    fn lint_result_no_errors() {
        let result = LintResult {
            skill_path: PathBuf::from("/test"),
            skill_id: "test/skill".to_string(),
            issues: vec![LintIssue {
                level: LintLevel::Warning,
                rule: "readme-missing".to_string(),
                message: "No README.md".to_string(),
            }],
        };

        assert!(!result.has_errors());
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 1);
    }
}
