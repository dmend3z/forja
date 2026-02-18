use std::path::Path;
use std::time::Duration;

use tokio::process::Command;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::error::{ForjaError, Result};
use crate::models::skill::Skill;

use super::models::{AiAnalysisOutput, AiSkillRecommendation, AiTechDeepDive, DetectedTech, SkillRecommendation};

const INITIAL_TIMEOUT: Duration = Duration::from_secs(60);
const DEEP_DIVE_TIMEOUT: Duration = Duration::from_secs(45);
const MAX_CONCURRENT: usize = 5;

/// Check if the `claude` CLI is available on the system.
pub fn claude_cli_available() -> bool {
    std::process::Command::new("claude")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Run initial AI analysis of the codebase.
/// Sends project info + deterministic results to Claude for deeper technology detection.
pub async fn analyze_codebase(
    root: &Path,
    deterministic_techs: &[DetectedTech],
) -> Result<AiAnalysisOutput> {
    let tech_summary: Vec<String> = deterministic_techs
        .iter()
        .map(|t| format!("- {} ({}): {}", t.name, t.category, t.evidence.join(", ")))
        .collect();

    let prompt = format!(
        r#"Analyze the project at {root} for technologies used. I've already detected these:

{techs}

Look deeper for additional technologies, frameworks, patterns, or tools that filesystem scanning might miss. Check configuration files, import patterns, and project structure.

Output ONLY a JSON block with this structure:
```json
{{
  "technologies": [
    {{
      "name": "technology-name",
      "category": "language|framework|database|testing|styling|devops|async|web",
      "confidence": "high|medium|low",
      "evidence": ["reason1", "reason2"]
    }}
  ]
}}
```

Include both what I already found (confirmed) and any new discoveries. Use lowercase names."#,
        root = root.display(),
        techs = tech_summary.join("\n"),
    );

    let output = run_claude(&prompt, INITIAL_TIMEOUT).await?;
    let json_str = extract_json_block(&output)
        .ok_or_else(|| ForjaError::ScanError("No JSON block found in Claude output".into()))?;

    serde_json::from_str::<AiAnalysisOutput>(json_str)
        .map_err(|e| ForjaError::ScanError(format!("Failed to parse AI analysis: {e}")))
}

/// Fan-out deep-dive analysis: one Claude call per technology to match skills.
/// Returns results for each technology (individual failures are non-fatal).
pub async fn deep_dive_technologies(
    root: &Path,
    techs: &[DetectedTech],
    skills: &[Skill],
) -> Vec<Result<Vec<SkillRecommendation>>> {
    let semaphore = std::sync::Arc::new(Semaphore::new(MAX_CONCURRENT));
    let mut join_set = JoinSet::new();

    let skill_catalog = build_skill_catalog(skills);
    let root = root.to_path_buf();

    for tech in techs {
        let sem = semaphore.clone();
        let tech = tech.clone();
        let catalog = skill_catalog.clone();
        let root = root.clone();

        join_set.spawn(async move {
            let _permit = sem.acquire().await.map_err(|e| {
                ForjaError::ScanError(format!("Semaphore error: {e}"))
            })?;

            deep_dive_single(&root, &tech, &catalog).await
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(inner) => results.push(inner),
            Err(e) => results.push(Err(ForjaError::ScanError(format!("Task join error: {e}")))),
        }
    }
    results
}

async fn deep_dive_single(
    root: &Path,
    tech: &DetectedTech,
    skill_catalog: &str,
) -> Result<Vec<SkillRecommendation>> {
    let prompt = format!(
        r#"I'm analyzing a project at {root} that uses {tech_name} ({category}).
Evidence: {evidence}

Here are the available forja skills:
{catalog}

Which of these skills would be most useful for this project's {tech_name} usage?
Consider the project's specific patterns and needs.

Output ONLY a JSON block:
```json
{{
  "technology": "{tech_name}",
  "recommended_skills": [
    {{
      "skill_id": "phase/tech/name",
      "confidence": "high|medium|low",
      "reason": "Brief explanation of why this skill is relevant"
    }}
  ]
}}
```

Only recommend skills that genuinely match. Prefer fewer, high-confidence recommendations."#,
        root = root.display(),
        tech_name = tech.name,
        category = tech.category,
        evidence = tech.evidence.join(", "),
        catalog = skill_catalog,
    );

    let output = run_claude(&prompt, DEEP_DIVE_TIMEOUT).await?;
    let json_str = extract_json_block(&output)
        .ok_or_else(|| ForjaError::ScanError(format!("No JSON in deep-dive for {}", tech.name)))?;

    let dive: AiTechDeepDive = serde_json::from_str(json_str)
        .map_err(|e| ForjaError::ScanError(format!("Parse error for {}: {e}", tech.name)))?;

    Ok(dive
        .recommended_skills
        .into_iter()
        .map(|ai_rec| ai_rec_to_skill_rec(ai_rec, &tech.name))
        .collect())
}

fn ai_rec_to_skill_rec(ai: AiSkillRecommendation, tech_name: &str) -> SkillRecommendation {
    SkillRecommendation {
        skill_id: ai.skill_id.clone(),
        name: ai.skill_id.split('/').next_back().unwrap_or(&ai.skill_id).to_string(),
        description: String::new(), // filled in by merge step
        phase: ai.skill_id.split('/').next().unwrap_or("code").to_string(),
        confidence: ai.confidence,
        reason: ai.reason,
        installed: false,
        matched_techs: vec![tech_name.to_string()],
    }
}

fn build_skill_catalog(skills: &[Skill]) -> String {
    skills
        .iter()
        .map(|s| format!("- {} ({}): {}", s.id, s.phase, s.description))
        .collect::<Vec<_>>()
        .join("\n")
}

async fn run_claude(prompt: &str, timeout: Duration) -> Result<String> {
    let result = tokio::time::timeout(
        timeout,
        Command::new("claude")
            .args(["--print", "--output-format", "text", "--max-turns", "1", "--", prompt])
            .output(),
    )
    .await;

    match result {
        Ok(Ok(output)) => {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .map_err(|e| ForjaError::ScanError(format!("Invalid UTF-8 from Claude: {e}")))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(ForjaError::ScanError(format!("Claude CLI failed: {stderr}")))
            }
        }
        Ok(Err(e)) => Err(ForjaError::ScanError(format!("Failed to run Claude CLI: {e}"))),
        Err(_) => Err(ForjaError::ClaudeCliTimeout(timeout.as_secs())),
    }
}

/// Extract JSON content from between ```json and ``` delimiters.
fn extract_json_block(text: &str) -> Option<&str> {
    let start = text.find("```json")?;
    let json_start = start + "```json".len();
    let rest = &text[json_start..];
    let end = rest.find("```")?;
    let json = rest[..end].trim();
    Some(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::models::Confidence;

    #[test]
    fn extract_json_from_markdown() {
        let text = r#"Here is the analysis:
```json
{"technologies": [{"name": "rust"}]}
```
That's all."#;

        let json = extract_json_block(text).unwrap();
        assert!(json.contains("\"rust\""));
        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(parsed["technologies"].is_array());
    }

    #[test]
    fn extract_json_no_block() {
        let text = "No JSON here, just text.";
        assert!(extract_json_block(text).is_none());
    }

    #[test]
    fn extract_json_multiple_blocks_takes_first() {
        let text = r#"
```json
{"first": true}
```
Some text
```json
{"second": true}
```"#;

        let json = extract_json_block(text).unwrap();
        assert!(json.contains("\"first\""));
    }

    #[test]
    fn build_skill_catalog_format() {
        let skills = vec![Skill {
            id: "code/rust/feature".into(),
            name: "rust-coder".into(),
            description: "Writes Rust code".into(),
            phase: crate::models::phase::Phase::Code,
            tech: "rust".into(),
            path: std::path::PathBuf::from("/tmp"),
            installed: false,
            content_types: vec![],
            keywords: vec![],
        }];

        let catalog = build_skill_catalog(&skills);
        assert!(catalog.contains("code/rust/feature"));
        assert!(catalog.contains("Writes Rust code"));
    }

    #[test]
    fn ai_rec_to_skill_rec_conversion() {
        let ai = AiSkillRecommendation {
            skill_id: "code/rust/feature".into(),
            confidence: Confidence::High,
            reason: "Strong Rust project".into(),
        };

        let rec = ai_rec_to_skill_rec(ai, "rust");
        assert_eq!(rec.skill_id, "code/rust/feature");
        assert_eq!(rec.phase, "code");
        assert_eq!(rec.name, "feature");
        assert_eq!(rec.matched_techs, vec!["rust"]);
    }

    #[test]
    fn claude_cli_available_check() {
        // This test just verifies the function doesn't panic
        let _ = claude_cli_available();
    }
}
