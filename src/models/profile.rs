use std::fmt;
use std::str::FromStr;

use super::phase::Phase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Fast,
    Balanced,
    Max,
}

impl Profile {
    pub fn all() -> &'static [Profile] {
        &[Profile::Fast, Profile::Balanced, Profile::Max]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Profile::Fast => "fast",
            Profile::Balanced => "balanced",
            Profile::Max => "max",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Profile::Fast => "All sonnet — fastest, lowest cost",
            Profile::Balanced => "Opus for thinking phases, sonnet for execution",
            Profile::Max => "All opus — highest quality",
        }
    }

    pub fn resolve_model(&self, phase: Phase) -> &'static str {
        match self {
            Profile::Fast => "sonnet",
            Profile::Balanced => {
                if phase.is_thinking_phase() {
                    "opus"
                } else {
                    "sonnet"
                }
            }
            Profile::Max => "opus",
        }
    }
}

impl FromStr for Profile {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fast" => Ok(Profile::Fast),
            "balanced" => Ok(Profile::Balanced),
            "max" => Ok(Profile::Max),
            _ => Err(format!("unknown profile: {s}")),
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::phase::Phase;

    #[test]
    fn fast_all_sonnet() {
        let profile = Profile::Fast;
        for &phase in Phase::all() {
            assert_eq!(profile.resolve_model(phase), "sonnet");
        }
    }

    #[test]
    fn balanced_thinking_opus_others_sonnet() {
        let profile = Profile::Balanced;
        assert_eq!(profile.resolve_model(Phase::Research), "opus");
        assert_eq!(profile.resolve_model(Phase::Review), "opus");
        assert_eq!(profile.resolve_model(Phase::Code), "sonnet");
        assert_eq!(profile.resolve_model(Phase::Test), "sonnet");
        assert_eq!(profile.resolve_model(Phase::Deploy), "sonnet");
    }

    #[test]
    fn max_all_opus() {
        let profile = Profile::Max;
        for &phase in Phase::all() {
            assert_eq!(profile.resolve_model(phase), "opus");
        }
    }

    #[test]
    fn is_thinking_phase() {
        assert!(Phase::Research.is_thinking_phase());
        assert!(Phase::Review.is_thinking_phase());
        assert!(!Phase::Code.is_thinking_phase());
        assert!(!Phase::Test.is_thinking_phase());
        assert!(!Phase::Deploy.is_thinking_phase());
        assert!(!Phase::Teams.is_thinking_phase());
    }

    #[test]
    fn from_str_roundtrip() {
        for profile in Profile::all() {
            let parsed: Profile = profile.as_str().parse().unwrap();
            assert_eq!(parsed, *profile);
        }
        assert!("unknown".parse::<Profile>().is_err());
    }
}
