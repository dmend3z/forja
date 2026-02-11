use super::skill::Skill;

/// The full catalog index, built by scanning the skills/ directory.
/// Rebuilt on every invocation — fast enough for hundreds of skills (<50ms).
#[derive(Debug)]
pub struct Registry {
    pub skills: Vec<Skill>,
}

/// Result of resolving a skill identifier (ID or short name).
#[derive(Debug)]
pub enum ResolveResult<'a> {
    Found(&'a Skill),
    NotFound,
    Ambiguous(Vec<&'a Skill>),
}

impl Registry {
    pub fn new(skills: Vec<Skill>) -> Self {
        Self { skills }
    }

    pub fn find_by_id(&self, id: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.id == id)
    }

    pub fn find_by_name(&self, name: &str) -> Vec<&Skill> {
        let q = name.to_lowercase();
        self.skills
            .iter()
            .filter(|s| s.name.to_lowercase() == q)
            .collect()
    }

    pub fn resolve(&self, identifier: &str) -> ResolveResult<'_> {
        // Try exact ID first
        if let Some(skill) = self.find_by_id(identifier) {
            return ResolveResult::Found(skill);
        }

        // Try name match
        let matches = self.find_by_name(identifier);
        match matches.len() {
            0 => ResolveResult::NotFound,
            1 => ResolveResult::Found(matches[0]),
            _ => ResolveResult::Ambiguous(matches),
        }
    }

    pub fn search(&self, query: &str) -> Vec<&Skill> {
        let q = query.to_lowercase();
        self.skills
            .iter()
            .filter(|s| {
                s.id.to_lowercase().contains(&q)
                    || s.name.to_lowercase().contains(&q)
                    || s.description.to_lowercase().contains(&q)
                    || s.phase.as_str().contains(&q)
                    || s.tech.to_lowercase().contains(&q)
                    || s.keywords.iter().any(|k| k.to_lowercase().contains(&q))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::phase::Phase;
    use crate::models::skill::ContentType;
    use std::path::PathBuf;

    fn make_skill(id: &str, name: &str, keywords: Vec<&str>) -> Skill {
        Skill {
            id: id.to_string(),
            name: name.to_string(),
            description: "test description".to_string(),
            phase: Phase::Code,
            tech: "general".to_string(),
            path: PathBuf::from("/tmp/test"),
            installed: false,
            content_types: vec![ContentType::Agent],
            keywords: keywords.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn search_matches_keywords() {
        let registry = Registry::new(vec![
            make_skill(
                "code/rust/feature",
                "rust-coder",
                vec!["ownership", "borrow-checker"],
            ),
            make_skill("code/general/feature", "coder", vec!["patterns"]),
        ]);

        let results = registry.search("ownership");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "code/rust/feature");
    }

    #[test]
    fn search_keywords_case_insensitive() {
        let registry = Registry::new(vec![make_skill(
            "code/rust/feature",
            "rust-coder",
            vec!["Ownership"],
        )]);

        let results = registry.search("ownership");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn resolve_by_exact_id() {
        let registry = Registry::new(vec![make_skill("code/rust/feature", "rust-coder", vec![])]);

        assert!(matches!(
            registry.resolve("code/rust/feature"),
            ResolveResult::Found(s) if s.id == "code/rust/feature"
        ));
    }

    #[test]
    fn resolve_by_unique_name() {
        let registry = Registry::new(vec![
            make_skill("code/rust/feature", "rust-coder", vec![]),
            make_skill("code/general/feature", "coder", vec![]),
        ]);

        assert!(matches!(
            registry.resolve("rust-coder"),
            ResolveResult::Found(s) if s.id == "code/rust/feature"
        ));
    }

    #[test]
    fn resolve_ambiguous_name() {
        let registry = Registry::new(vec![
            make_skill("code/rust/feature", "feature", vec![]),
            make_skill("code/ts/feature", "feature", vec![]),
        ]);

        assert!(matches!(
            registry.resolve("feature"),
            ResolveResult::Ambiguous(v) if v.len() == 2
        ));
    }

    #[test]
    fn resolve_not_found() {
        let registry = Registry::new(vec![make_skill("code/rust/feature", "rust-coder", vec![])]);

        assert!(matches!(
            registry.resolve("nonexistent"),
            ResolveResult::NotFound
        ));
    }

    #[test]
    fn resolve_id_takes_precedence_over_name() {
        let registry = Registry::new(vec![
            make_skill("code/rust/feature", "rust-coder", vec![]),
            make_skill("rust-coder", "something-else", vec![]),
        ]);

        // "rust-coder" matches the ID of the second skill exactly
        assert!(matches!(
            registry.resolve("rust-coder"),
            ResolveResult::Found(s) if s.id == "rust-coder"
        ));
    }
}
