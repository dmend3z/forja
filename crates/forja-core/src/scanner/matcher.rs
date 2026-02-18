use crate::models::registry::Registry;

use super::models::{Confidence, DetectedTech, SkillRecommendation};

/// Static mapping from detected technology name to forja skill IDs.
const TECH_TO_SKILLS: &[(&str, &[&str])] = &[
    ("rust", &["code/rust/feature"]),
    ("typescript", &["code/typescript/feature"]),
    ("nextjs", &["code/nextjs/feature"]),
    ("nestjs", &["code/nestjs/feature"]),
    ("python", &["code/python/feature"]),
    ("golang", &["code/golang/feature"]),
    ("playwright", &["test/e2e/playwright"]),
    ("sql", &["code/database/feature"]),
    ("prisma", &["code/database/feature"]),
    ("drizzle", &["code/database/feature"]),
    ("sqlx", &["code/database/feature"]),
];

/// Skills recommended for every project regardless of tech stack.
const ALWAYS_RECOMMEND: &[&str] = &[
    "research/codebase/explorer",
    "review/code-quality/reviewer",
    "review/security/auditor",
    "deploy/git/commit",
    "deploy/git/pr",
    "test/tdd/workflow",
    "test/generate/suite",
    "test/coverage/analyzer",
];

/// Map detected technologies to skill recommendations using the static mapping table.
pub fn match_skills(
    techs: &[DetectedTech],
    registry: &Registry,
    installed: &[String],
) -> Vec<SkillRecommendation> {
    let mut recs = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // Match tech-specific skills
    for tech in techs {
        if let Some(&(_, skill_ids)) = TECH_TO_SKILLS.iter().find(|(name, _)| *name == tech.name) {
            for &skill_id in skill_ids {
                if seen_ids.insert(skill_id.to_string())
                    && let Some(rec) = build_recommendation(
                        skill_id,
                        registry,
                        installed,
                        tech.confidence,
                        &tech.evidence.join("; "),
                        vec![tech.name.clone()],
                    )
                {
                    recs.push(rec);
                }
            }
        }
    }

    // Always-recommend skills
    for &skill_id in ALWAYS_RECOMMEND {
        if seen_ids.insert(skill_id.to_string())
            && let Some(rec) = build_recommendation(
                skill_id,
                registry,
                installed,
                Confidence::High,
                "Always recommended",
                vec![],
            )
        {
            recs.push(rec);
        }
    }

    recs
}

/// Merge deterministic and AI recommendations, deduplicating by skill_id.
/// Higher confidence wins. If same confidence, AI wins (richer reason).
pub fn merge_recommendations(
    det: Vec<SkillRecommendation>,
    ai: Vec<SkillRecommendation>,
) -> Vec<SkillRecommendation> {
    let mut map: std::collections::HashMap<String, SkillRecommendation> = std::collections::HashMap::new();

    // Insert deterministic first
    for rec in det {
        map.insert(rec.skill_id.clone(), rec);
    }

    // Merge AI: replace if higher confidence, or same confidence (AI has richer reason)
    for rec in ai {
        let replace = match map.get(&rec.skill_id) {
            None => true,
            Some(existing) => rec.confidence >= existing.confidence,
        };
        if replace {
            map.insert(rec.skill_id.clone(), rec);
        }
    }

    let mut merged: Vec<_> = map.into_values().collect();
    sort_recommendations(&mut merged);
    merged
}

/// Sort recommendations by phase order then confidence descending.
pub fn sort_recommendations(recs: &mut [SkillRecommendation]) {
    let phase_order = |p: &str| -> u8 {
        match p {
            "research" => 0,
            "code" => 1,
            "test" => 2,
            "review" => 3,
            "deploy" => 4,
            "teams" => 5,
            _ => 6,
        }
    };
    recs.sort_by(|a, b| {
        phase_order(&a.phase)
            .cmp(&phase_order(&b.phase))
            .then_with(|| b.confidence.cmp(&a.confidence))
    });
}

fn build_recommendation(
    skill_id: &str,
    registry: &Registry,
    installed: &[String],
    confidence: Confidence,
    reason: &str,
    matched_techs: Vec<String>,
) -> Option<SkillRecommendation> {
    let skill = registry.find_by_id(skill_id)?;
    Some(SkillRecommendation {
        skill_id: skill_id.to_string(),
        name: skill.name.clone(),
        description: skill.description.clone(),
        phase: skill.phase.as_str().to_string(),
        confidence,
        reason: reason.to_string(),
        installed: installed.contains(&skill_id.to_string()),
        matched_techs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::phase::Phase;
    use crate::models::registry::Registry;
    use crate::models::skill::{ContentType, Skill};
    use crate::scanner::models::{DetectedTech, DetectionSource};
    use std::path::PathBuf;

    fn make_skill(id: &str, name: &str, phase: Phase) -> Skill {
        Skill {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("{name} skill"),
            phase,
            tech: "general".to_string(),
            path: PathBuf::from("/tmp/test"),
            installed: false,
            content_types: vec![ContentType::Agent],
            keywords: vec![],
        }
    }

    fn make_registry() -> Registry {
        Registry::new(vec![
            make_skill("code/rust/feature", "rust-coder", Phase::Code),
            make_skill("code/typescript/feature", "ts-coder", Phase::Code),
            make_skill("code/nextjs/feature", "nextjs-coder", Phase::Code),
            make_skill("code/database/feature", "db-coder", Phase::Code),
            make_skill("research/codebase/explorer", "explorer", Phase::Research),
            make_skill("review/code-quality/reviewer", "reviewer", Phase::Review),
            make_skill("review/security/auditor", "auditor", Phase::Review),
            make_skill("deploy/git/commit", "committer", Phase::Deploy),
            make_skill("deploy/git/pr", "pr-creator", Phase::Deploy),
            make_skill("test/tdd/workflow", "tdd-guide", Phase::Test),
            make_skill("test/generate/suite", "test-gen", Phase::Test),
            make_skill("test/coverage/analyzer", "coverage", Phase::Test),
            make_skill("test/e2e/playwright", "e2e-tester", Phase::Test),
        ])
    }

    fn make_tech(name: &str) -> DetectedTech {
        DetectedTech {
            name: name.to_string(),
            category: "language".to_string(),
            evidence: vec![format!("Found {name}")],
            source: DetectionSource::Deterministic,
            confidence: Confidence::High,
            version: None,
        }
    }

    #[test]
    fn match_rust_project() {
        let registry = make_registry();
        let techs = vec![make_tech("rust")];
        let recs = match_skills(&techs, &registry, &[]);

        assert!(recs.iter().any(|r| r.skill_id == "code/rust/feature"));
        // Always-recommend skills should also be present
        assert!(recs.iter().any(|r| r.skill_id == "research/codebase/explorer"));
        assert!(recs.iter().any(|r| r.skill_id == "deploy/git/commit"));
    }

    #[test]
    fn match_marks_installed() {
        let registry = make_registry();
        let techs = vec![make_tech("rust")];
        let installed = vec!["code/rust/feature".to_string()];
        let recs = match_skills(&techs, &registry, &installed);

        let rust_rec = recs.iter().find(|r| r.skill_id == "code/rust/feature").unwrap();
        assert!(rust_rec.installed);
    }

    #[test]
    fn match_no_duplicate_skill_ids() {
        let registry = make_registry();
        let techs = vec![make_tech("sql"), make_tech("prisma")];
        let recs = match_skills(&techs, &registry, &[]);

        let db_count = recs.iter().filter(|r| r.skill_id == "code/database/feature").count();
        assert_eq!(db_count, 1, "database skill should appear once");
    }

    #[test]
    fn match_empty_techs_still_recommends_always() {
        let registry = make_registry();
        let recs = match_skills(&[], &registry, &[]);

        assert!(!recs.is_empty(), "should still have always-recommend skills");
        assert!(recs.iter().any(|r| r.skill_id == "research/codebase/explorer"));
    }

    #[test]
    fn merge_ai_wins_on_same_confidence() {
        let det = vec![SkillRecommendation {
            skill_id: "code/rust/feature".to_string(),
            name: "rust-coder".to_string(),
            description: "Writes Rust".to_string(),
            phase: "code".to_string(),
            confidence: Confidence::High,
            reason: "Found Cargo.toml".to_string(),
            installed: false,
            matched_techs: vec!["rust".to_string()],
        }];
        let ai = vec![SkillRecommendation {
            skill_id: "code/rust/feature".to_string(),
            name: "rust-coder".to_string(),
            description: "Writes Rust".to_string(),
            phase: "code".to_string(),
            confidence: Confidence::High,
            reason: "Rust project using tokio async with axum web framework".to_string(),
            installed: false,
            matched_techs: vec!["rust".to_string()],
        }];

        let merged = merge_recommendations(det, ai);
        let rust = merged.iter().find(|r| r.skill_id == "code/rust/feature").unwrap();
        assert!(rust.reason.contains("tokio"), "AI reason should win on same confidence");
    }

    #[test]
    fn merge_higher_confidence_wins() {
        let det = vec![SkillRecommendation {
            skill_id: "test/e2e/playwright".to_string(),
            name: "e2e-tester".to_string(),
            description: "E2E testing".to_string(),
            phase: "test".to_string(),
            confidence: Confidence::Low,
            reason: "Maybe".to_string(),
            installed: false,
            matched_techs: vec![],
        }];
        let ai = vec![SkillRecommendation {
            skill_id: "test/e2e/playwright".to_string(),
            name: "e2e-tester".to_string(),
            description: "E2E testing".to_string(),
            phase: "test".to_string(),
            confidence: Confidence::High,
            reason: "Playwright config found with comprehensive test suite".to_string(),
            installed: false,
            matched_techs: vec!["playwright".to_string()],
        }];

        let merged = merge_recommendations(det, ai);
        let pw = merged.iter().find(|r| r.skill_id == "test/e2e/playwright").unwrap();
        assert_eq!(pw.confidence, Confidence::High);
    }

    #[test]
    fn sort_by_phase_then_confidence() {
        let mut recs = vec![
            SkillRecommendation {
                skill_id: "deploy/git/commit".into(),
                name: "committer".into(),
                description: "".into(),
                phase: "deploy".into(),
                confidence: Confidence::High,
                reason: "".into(),
                installed: false,
                matched_techs: vec![],
            },
            SkillRecommendation {
                skill_id: "research/codebase/explorer".into(),
                name: "explorer".into(),
                description: "".into(),
                phase: "research".into(),
                confidence: Confidence::Medium,
                reason: "".into(),
                installed: false,
                matched_techs: vec![],
            },
            SkillRecommendation {
                skill_id: "code/rust/feature".into(),
                name: "rust-coder".into(),
                description: "".into(),
                phase: "code".into(),
                confidence: Confidence::High,
                reason: "".into(),
                installed: false,
                matched_techs: vec![],
            },
        ];

        sort_recommendations(&mut recs);
        assert_eq!(recs[0].phase, "research");
        assert_eq!(recs[1].phase, "code");
        assert_eq!(recs[2].phase, "deploy");
    }
}
