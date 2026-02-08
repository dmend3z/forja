use super::skill::Skill;

/// The full catalog index, built by scanning the skills/ directory.
/// Rebuilt on every invocation — fast enough for hundreds of skills (<50ms).
#[derive(Debug)]
pub struct Registry {
    pub skills: Vec<Skill>,
}

impl Registry {
    pub fn new(skills: Vec<Skill>) -> Self {
        Self { skills }
    }

    pub fn find_by_id(&self, id: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.id == id)
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
            })
            .collect()
    }
}
