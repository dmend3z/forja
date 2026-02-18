use crate::output;

/// Context used to determine which tips are relevant.
pub struct TipContext {
    pub installed_count: usize,
    pub total_skills: usize,
    pub has_teams: bool,
    pub is_initialized: bool,
}

/// A contextual tip with optional command suggestion.
pub struct Tip {
    pub message: String,
    pub command: Option<String>,
    pub priority: u8,
}

/// Return relevant tips based on current state, sorted by priority (highest first).
pub fn get_tips(ctx: &TipContext) -> Vec<Tip> {
    let mut tips = Vec::new();

    if !ctx.is_initialized {
        tips.push(Tip {
            message: "Get started by initializing forja".into(),
            command: Some("forja init".into()),
            priority: 10,
        });
        return tips;
    }

    if ctx.installed_count == 0 {
        tips.push(Tip {
            message: "No skills installed — install all available skills".into(),
            command: Some("forja install --all".into()),
            priority: 9,
        });
    }

    if !ctx.has_teams && ctx.installed_count > 5 {
        tips.push(Tip {
            message: "Create a team for complex tasks".into(),
            command: Some("forja team preset solo-sprint".into()),
            priority: 5,
        });
    }

    if ctx.installed_count > 0 {
        tips.push(Tip {
            message: "Let AI pick the right skill for your task".into(),
            command: Some("forja task \"your task\"".into()),
            priority: 4,
        });
    }

    if ctx.installed_count < ctx.total_skills && ctx.installed_count > 0 {
        tips.push(Tip {
            message: "You have more skills available to install".into(),
            command: Some("forja list --available".into()),
            priority: 3,
        });
    }

    tips.push(Tip {
        message: "Keep your skills catalog up to date".into(),
        command: Some("forja update".into()),
        priority: 1,
    });

    tips.sort_by(|a, b| b.priority.cmp(&a.priority));
    tips
}

/// Print a single random tip from the set of contextual tips.
pub fn print_random_tip(ctx: &TipContext) {
    let tips = get_tips(ctx);
    if let Some(tip) = tips.first() {
        let msg = match &tip.command {
            Some(cmd) => format!("{} Try: {}", tip.message, cmd),
            None => tip.message.clone(),
        };
        output::print_tip(&msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uninitialized_gets_init_tip() {
        let ctx = TipContext {
            installed_count: 0,
            total_skills: 25,
            has_teams: false,
            is_initialized: false,
        };
        let tips = get_tips(&ctx);
        assert_eq!(tips.len(), 1);
        assert!(tips[0].command.as_deref() == Some("forja init"));
    }

    #[test]
    fn zero_installed_gets_install_all_tip() {
        let ctx = TipContext {
            installed_count: 0,
            total_skills: 25,
            has_teams: false,
            is_initialized: true,
        };
        let tips = get_tips(&ctx);
        assert!(tips.iter().any(|t| {
            t.command
                .as_deref()
                .is_some_and(|c| c.contains("install --all"))
        }));
    }

    #[test]
    fn no_teams_with_many_skills_gets_team_tip() {
        let ctx = TipContext {
            installed_count: 10,
            total_skills: 25,
            has_teams: false,
            is_initialized: true,
        };
        let tips = get_tips(&ctx);
        assert!(tips.iter().any(|t| {
            t.command
                .as_deref()
                .is_some_and(|c| c.contains("team preset"))
        }));
    }

    #[test]
    fn has_teams_no_team_tip() {
        let ctx = TipContext {
            installed_count: 10,
            total_skills: 25,
            has_teams: true,
            is_initialized: true,
        };
        let tips = get_tips(&ctx);
        assert!(!tips.iter().any(|t| {
            t.command
                .as_deref()
                .is_some_and(|c| c.contains("team preset"))
        }));
    }

    #[test]
    fn tips_sorted_by_priority() {
        let ctx = TipContext {
            installed_count: 3,
            total_skills: 25,
            has_teams: false,
            is_initialized: true,
        };
        let tips = get_tips(&ctx);
        for window in tips.windows(2) {
            assert!(window[0].priority >= window[1].priority);
        }
    }

    #[test]
    fn fully_set_up_gets_low_priority_tips() {
        let ctx = TipContext {
            installed_count: 25,
            total_skills: 25,
            has_teams: true,
            is_initialized: true,
        };
        let tips = get_tips(&ctx);
        // Should still have at least the update tip and task tip
        assert!(tips.len() >= 2);
        // No high-priority tips (install all, init)
        assert!(tips.iter().all(|t| t.priority < 9));
    }
}
