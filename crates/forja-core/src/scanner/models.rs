use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// How a technology was detected during scanning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionSource {
    Deterministic,
    Ai,
    Both,
}

/// Confidence level for a detection or recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    pub fn as_indicator(&self) -> &'static str {
        match self {
            Confidence::Low => "+",
            Confidence::Medium => "++",
            Confidence::High => "+++",
        }
    }
}

impl PartialOrd for Confidence {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Confidence {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn rank(c: &Confidence) -> u8 {
            match c {
                Confidence::Low => 0,
                Confidence::Medium => 1,
                Confidence::High => 2,
            }
        }
        rank(self).cmp(&rank(other))
    }
}

/// A technology detected in the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTech {
    pub name: String,
    pub category: String,
    pub evidence: Vec<String>,
    pub source: DetectionSource,
    pub confidence: Confidence,
    pub version: Option<String>,
}

/// A skill recommendation produced by scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecommendation {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub phase: String,
    pub confidence: Confidence,
    pub reason: String,
    pub installed: bool,
    pub matched_techs: Vec<String>,
}

/// Complete scan result for JSON output and downstream consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub project_root: PathBuf,
    pub detected_techs: Vec<DetectedTech>,
    pub recommendations: Vec<SkillRecommendation>,
    pub ai_analysis_used: bool,
    pub scanned_at: DateTime<Utc>,
}

// --- AI Claude CLI output schemas (Deserialize only) ---

/// Top-level AI analysis output from initial codebase scan.
#[derive(Debug, Clone, Deserialize)]
pub struct AiAnalysisOutput {
    pub technologies: Vec<AiDetectedTech>,
}

/// A technology detected by AI analysis.
#[derive(Debug, Clone, Deserialize)]
pub struct AiDetectedTech {
    pub name: String,
    pub category: String,
    pub confidence: Confidence,
    pub evidence: Vec<String>,
}

/// Deep-dive analysis result for a single technology.
#[derive(Debug, Clone, Deserialize)]
pub struct AiTechDeepDive {
    pub technology: String,
    pub recommended_skills: Vec<AiSkillRecommendation>,
}

/// A skill recommendation from AI deep-dive analysis.
#[derive(Debug, Clone, Deserialize)]
pub struct AiSkillRecommendation {
    pub skill_id: String,
    pub confidence: Confidence,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_ordering() {
        assert!(Confidence::Low < Confidence::Medium);
        assert!(Confidence::Medium < Confidence::High);
        assert!(Confidence::Low < Confidence::High);
    }

    #[test]
    fn confidence_indicators() {
        assert_eq!(Confidence::Low.as_indicator(), "+");
        assert_eq!(Confidence::Medium.as_indicator(), "++");
        assert_eq!(Confidence::High.as_indicator(), "+++");
    }

    #[test]
    fn confidence_sorting() {
        let mut levels = vec![Confidence::High, Confidence::Low, Confidence::Medium];
        levels.sort();
        assert_eq!(levels, vec![Confidence::Low, Confidence::Medium, Confidence::High]);
    }

    #[test]
    fn detection_source_serde_roundtrip() {
        let source = DetectionSource::Both;
        let json = serde_json::to_string(&source).unwrap();
        let parsed: DetectionSource = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, DetectionSource::Both);
    }

    #[test]
    fn ai_analysis_output_deserialize() {
        let json = r#"{
            "technologies": [
                {
                    "name": "rust",
                    "category": "language",
                    "confidence": "high",
                    "evidence": ["Cargo.toml found"]
                }
            ]
        }"#;
        let output: AiAnalysisOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.technologies.len(), 1);
        assert_eq!(output.technologies[0].name, "rust");
        assert_eq!(output.technologies[0].confidence, Confidence::High);
    }

    #[test]
    fn ai_tech_deep_dive_deserialize() {
        let json = r#"{
            "technology": "rust",
            "recommended_skills": [
                {
                    "skill_id": "code/rust/feature",
                    "confidence": "high",
                    "reason": "Rust project with tokio async runtime"
                }
            ]
        }"#;
        let dive: AiTechDeepDive = serde_json::from_str(json).unwrap();
        assert_eq!(dive.technology, "rust");
        assert_eq!(dive.recommended_skills.len(), 1);
        assert_eq!(dive.recommended_skills[0].skill_id, "code/rust/feature");
    }
}
