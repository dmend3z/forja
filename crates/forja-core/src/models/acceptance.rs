use serde::{Deserialize, Serialize};

/// An acceptance criterion that can be either a simple string or a structured
/// object with an optional test command.
///
/// YAML examples:
/// ```yaml
/// acceptance_criteria:
///   - "Users can log in"                              # simple string
///   - description: "Protected routes reject unauth"   # structured
///     test: "cargo test auth_reject"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AcceptanceCriterion {
    Simple(String),
    Structured {
        description: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        test: Option<String>,
    },
}

impl AcceptanceCriterion {
    pub fn description(&self) -> &str {
        match self {
            Self::Simple(s) => s,
            Self::Structured { description, .. } => description,
        }
    }

    pub fn test_command(&self) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::Structured { test, .. } => test.as_deref(),
        }
    }
}

impl std::fmt::Display for AcceptanceCriterion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple(s) => write!(f, "{s}"),
            Self::Structured {
                description, test, ..
            } => {
                write!(f, "{description}")?;
                if let Some(cmd) = test {
                    write!(f, " (test: {cmd})")?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_simple_string() {
        let yaml = "\"Users can log in\"";
        let criterion: AcceptanceCriterion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(criterion.description(), "Users can log in");
        assert!(criterion.test_command().is_none());
    }

    #[test]
    fn deserialize_structured_with_test() {
        let yaml = r#"
description: "Protected routes reject unauthenticated"
test: "cargo test auth_reject"
"#;
        let criterion: AcceptanceCriterion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            criterion.description(),
            "Protected routes reject unauthenticated"
        );
        assert_eq!(criterion.test_command(), Some("cargo test auth_reject"));
    }

    #[test]
    fn deserialize_structured_without_test() {
        let yaml = r#"
description: "Dashboard loads in under 2 seconds"
"#;
        let criterion: AcceptanceCriterion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            criterion.description(),
            "Dashboard loads in under 2 seconds"
        );
        assert!(criterion.test_command().is_none());
    }

    #[test]
    fn deserialize_list_mixed() {
        let yaml = r#"
- "Simple criterion"
- description: "Structured criterion"
  test: "npm test"
- "Another simple one"
"#;
        let criteria: Vec<AcceptanceCriterion> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(criteria.len(), 3);
        assert!(matches!(criteria[0], AcceptanceCriterion::Simple(_)));
        assert!(matches!(criteria[1], AcceptanceCriterion::Structured { .. }));
        assert!(matches!(criteria[2], AcceptanceCriterion::Simple(_)));
    }

    #[test]
    fn display_simple() {
        let c = AcceptanceCriterion::Simple("Users can log in".to_string());
        assert_eq!(format!("{c}"), "Users can log in");
    }

    #[test]
    fn display_structured_with_test() {
        let c = AcceptanceCriterion::Structured {
            description: "Auth works".to_string(),
            test: Some("cargo test".to_string()),
        };
        assert_eq!(format!("{c}"), "Auth works (test: cargo test)");
    }

    #[test]
    fn display_structured_without_test() {
        let c = AcceptanceCriterion::Structured {
            description: "Auth works".to_string(),
            test: None,
        };
        assert_eq!(format!("{c}"), "Auth works");
    }

    #[test]
    fn roundtrip_simple() {
        let original = AcceptanceCriterion::Simple("Test this".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let parsed: AcceptanceCriterion = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.description(), "Test this");
    }

    #[test]
    fn roundtrip_structured() {
        let original = AcceptanceCriterion::Structured {
            description: "Test this".to_string(),
            test: Some("cargo test".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: AcceptanceCriterion = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.description(), "Test this");
        assert_eq!(parsed.test_command(), Some("cargo test"));
    }
}
