use forja_core::scanner::models::SkillRecommendation;

/// A single item in the scan results list.
pub struct ScanItem {
    pub rec: SkillRecommendation,
    pub checked: bool,
    pub locked: bool, // already installed, cannot uncheck
}

/// TUI state for the scan checkbox selection.
pub struct ScanApp {
    pub groups: Vec<(String, Vec<ScanItem>)>, // (phase_name, items) grouped by phase
    pub cursor: (usize, usize),               // (group_idx, item_idx)
    pub should_install: bool,
    pub should_quit: bool,
    pub scroll_offset: usize,
    pub tech_count: usize,
}

impl ScanApp {
    /// Create a new ScanApp from recommendations.
    /// Groups by phase, pre-checks non-installed items, locks installed ones.
    pub fn new(recommendations: Vec<SkillRecommendation>, show_installed: bool) -> Self {
        let tech_count = 0; // set by caller
        let mut groups: Vec<(String, Vec<ScanItem>)> = Vec::new();

        for rec in recommendations {
            if !show_installed && rec.installed {
                continue;
            }

            let item = ScanItem {
                locked: rec.installed,
                checked: !rec.installed, // pre-check non-installed
                rec,
            };

            if let Some(group) = groups.iter_mut().find(|(phase, _)| *phase == item.rec.phase) {
                group.1.push(item);
            } else {
                let phase = item.rec.phase.clone();
                groups.push((phase, vec![item]));
            }
        }

        // Remove empty groups
        groups.retain(|(_, items)| !items.is_empty());

        let cursor = (0, 0);

        ScanApp {
            groups,
            cursor,
            should_install: false,
            should_quit: false,
            scroll_offset: 0,
            tech_count,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    pub fn total_items(&self) -> usize {
        self.groups.iter().map(|(_, items)| items.len()).sum()
    }

    pub fn checked_count(&self) -> usize {
        self.groups
            .iter()
            .flat_map(|(_, items)| items.iter())
            .filter(|item| item.checked && !item.locked)
            .count()
    }

    /// Move cursor down, wrapping across groups.
    pub fn move_down(&mut self) {
        if self.groups.is_empty() {
            return;
        }
        let (gi, ii) = self.cursor;
        let group_len = self.groups[gi].1.len();
        if ii + 1 < group_len {
            self.cursor = (gi, ii + 1);
        } else if gi + 1 < self.groups.len() {
            self.cursor = (gi + 1, 0);
        }
        // At the very end: don't wrap
    }

    /// Move cursor up, wrapping across groups.
    pub fn move_up(&mut self) {
        if self.groups.is_empty() {
            return;
        }
        let (gi, ii) = self.cursor;
        if ii > 0 {
            self.cursor = (gi, ii - 1);
        } else if gi > 0 {
            let prev_len = self.groups[gi - 1].1.len();
            self.cursor = (gi - 1, prev_len - 1);
        }
        // At the very start: don't wrap
    }

    /// Jump to next phase group.
    pub fn jump_next_group(&mut self) {
        if self.groups.is_empty() {
            return;
        }
        let (gi, _) = self.cursor;
        if gi + 1 < self.groups.len() {
            self.cursor = (gi + 1, 0);
        } else {
            self.cursor = (0, 0); // wrap to first
        }
    }

    /// Toggle the current item's checkbox.
    pub fn toggle(&mut self) {
        if self.groups.is_empty() {
            return;
        }
        let (gi, ii) = self.cursor;
        let item = &mut self.groups[gi].1[ii];
        if !item.locked {
            item.checked = !item.checked;
        }
    }

    /// Select all non-locked items.
    pub fn select_all(&mut self) {
        for (_, items) in &mut self.groups {
            for item in items {
                if !item.locked {
                    item.checked = true;
                }
            }
        }
    }

    /// Deselect all non-locked items.
    pub fn deselect_all(&mut self) {
        for (_, items) in &mut self.groups {
            for item in items {
                if !item.locked {
                    item.checked = false;
                }
            }
        }
    }

    /// Confirm selection and mark for installation.
    pub fn confirm(&mut self) {
        self.should_install = true;
    }

    /// Get the skill IDs of all checked (non-locked) items.
    pub fn selected_skill_ids(&self) -> Vec<String> {
        self.groups
            .iter()
            .flat_map(|(_, items)| items.iter())
            .filter(|item| item.checked && !item.locked)
            .map(|item| item.rec.skill_id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forja_core::scanner::models::Confidence;

    fn make_rec(id: &str, phase: &str, installed: bool) -> SkillRecommendation {
        SkillRecommendation {
            skill_id: id.to_string(),
            name: id.split('/').last().unwrap().to_string(),
            description: format!("{id} description"),
            phase: phase.to_string(),
            confidence: Confidence::High,
            reason: "test".to_string(),
            installed,
            matched_techs: vec![],
        }
    }

    #[test]
    fn groups_by_phase() {
        let recs = vec![
            make_rec("code/rust/feature", "code", false),
            make_rec("code/ts/feature", "code", false),
            make_rec("test/tdd/workflow", "test", false),
        ];
        let app = ScanApp::new(recs, false);
        assert_eq!(app.groups.len(), 2);
        assert_eq!(app.groups[0].0, "code");
        assert_eq!(app.groups[0].1.len(), 2);
        assert_eq!(app.groups[1].0, "test");
    }

    #[test]
    fn pre_checks_non_installed() {
        let recs = vec![
            make_rec("code/rust/feature", "code", false),
            make_rec("code/ts/feature", "code", true),
        ];
        let app = ScanApp::new(recs, true);
        assert!(app.groups[0].1[0].checked); // non-installed: checked
        assert!(!app.groups[0].1[1].checked); // installed: not checked (locked)
    }

    #[test]
    fn installed_items_locked() {
        let recs = vec![make_rec("code/rust/feature", "code", true)];
        let app = ScanApp::new(recs, true);
        assert!(app.groups[0].1[0].locked);
    }

    #[test]
    fn hide_installed_when_not_show() {
        let recs = vec![
            make_rec("code/rust/feature", "code", false),
            make_rec("code/ts/feature", "code", true),
        ];
        let app = ScanApp::new(recs, false);
        assert_eq!(app.total_items(), 1); // only non-installed
    }

    #[test]
    fn move_down_within_group() {
        let recs = vec![
            make_rec("code/rust/feature", "code", false),
            make_rec("code/ts/feature", "code", false),
        ];
        let mut app = ScanApp::new(recs, false);
        assert_eq!(app.cursor, (0, 0));
        app.move_down();
        assert_eq!(app.cursor, (0, 1));
    }

    #[test]
    fn move_down_across_groups() {
        let recs = vec![
            make_rec("code/rust/feature", "code", false),
            make_rec("test/tdd/workflow", "test", false),
        ];
        let mut app = ScanApp::new(recs, false);
        app.move_down(); // to (1, 0) — next group
        assert_eq!(app.cursor, (1, 0));
    }

    #[test]
    fn move_up_across_groups() {
        let recs = vec![
            make_rec("code/rust/feature", "code", false),
            make_rec("test/tdd/workflow", "test", false),
        ];
        let mut app = ScanApp::new(recs, false);
        app.cursor = (1, 0);
        app.move_up();
        assert_eq!(app.cursor, (0, 0)); // back to last item of previous group
    }

    #[test]
    fn toggle_checks_and_unchecks() {
        let recs = vec![make_rec("code/rust/feature", "code", false)];
        let mut app = ScanApp::new(recs, false);
        assert!(app.groups[0].1[0].checked); // pre-checked
        app.toggle();
        assert!(!app.groups[0].1[0].checked);
        app.toggle();
        assert!(app.groups[0].1[0].checked);
    }

    #[test]
    fn toggle_ignores_locked() {
        let recs = vec![make_rec("code/rust/feature", "code", true)];
        let mut app = ScanApp::new(recs, true);
        let was_checked = app.groups[0].1[0].checked;
        app.toggle();
        assert_eq!(app.groups[0].1[0].checked, was_checked); // unchanged
    }

    #[test]
    fn select_all_deselect_all() {
        let recs = vec![
            make_rec("code/rust/feature", "code", false),
            make_rec("code/ts/feature", "code", false),
        ];
        let mut app = ScanApp::new(recs, false);
        app.deselect_all();
        assert_eq!(app.checked_count(), 0);
        app.select_all();
        assert_eq!(app.checked_count(), 2);
    }

    #[test]
    fn selected_skill_ids() {
        let recs = vec![
            make_rec("code/rust/feature", "code", false),
            make_rec("code/ts/feature", "code", false),
        ];
        let mut app = ScanApp::new(recs, false);
        // Both pre-checked, deselect one
        app.toggle(); // deselect first
        let ids = app.selected_skill_ids();
        assert_eq!(ids, vec!["code/ts/feature"]);
    }

    #[test]
    fn jump_next_group() {
        let recs = vec![
            make_rec("code/rust/feature", "code", false),
            make_rec("test/tdd/workflow", "test", false),
            make_rec("deploy/git/commit", "deploy", false),
        ];
        let mut app = ScanApp::new(recs, false);
        assert_eq!(app.cursor.0, 0); // code
        app.jump_next_group();
        assert_eq!(app.cursor.0, 1); // test
        app.jump_next_group();
        assert_eq!(app.cursor.0, 2); // deploy
        app.jump_next_group();
        assert_eq!(app.cursor.0, 0); // wrap to code
    }

    #[test]
    fn empty_recommendations() {
        let app = ScanApp::new(vec![], false);
        assert!(app.is_empty());
        assert_eq!(app.total_items(), 0);
    }

    #[test]
    fn confirm_sets_should_install() {
        let recs = vec![make_rec("code/rust/feature", "code", false)];
        let mut app = ScanApp::new(recs, false);
        app.confirm();
        assert!(app.should_install);
    }
}
