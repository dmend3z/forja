use tui_textarea::TextArea;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Textarea,
    Team,
    Profile,
}

pub struct App<'a> {
    pub focus: Focus,
    pub team_labels: Vec<String>,
    pub team_names: Vec<String>,
    pub team_index: usize,
    pub profile_options: Vec<String>,
    pub profile_index: usize,
    pub textarea: TextArea<'a>,
    pub should_quit: bool,
    pub should_launch: bool,
    pub error_message: Option<String>,
}

impl<'a> App<'a> {
    pub fn new(team_labels: Vec<String>, team_names: Vec<String>, profile_options: Vec<String>) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_placeholder_text("Describe your task here...");
        Self {
            focus: Focus::Textarea,
            team_labels,
            team_names,
            team_index: 0,
            profile_options,
            profile_index: 0,
            textarea,
            should_quit: false,
            should_launch: false,
            error_message: None,
        }
    }

    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Textarea => Focus::Team,
            Focus::Team => Focus::Profile,
            Focus::Profile => Focus::Textarea,
        };
    }

    pub fn prev_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Textarea => Focus::Profile,
            Focus::Team => Focus::Textarea,
            Focus::Profile => Focus::Team,
        };
    }

    pub fn select_next(&mut self) {
        match self.focus {
            Focus::Team => {
                self.team_index = (self.team_index + 1) % self.team_labels.len();
            }
            Focus::Profile => {
                self.profile_index = (self.profile_index + 1) % self.profile_options.len();
            }
            Focus::Textarea => {}
        }
    }

    pub fn select_prev(&mut self) {
        match self.focus {
            Focus::Team => {
                self.team_index = if self.team_index == 0 {
                    self.team_labels.len() - 1
                } else {
                    self.team_index - 1
                };
            }
            Focus::Profile => {
                self.profile_index = if self.profile_index == 0 {
                    self.profile_options.len() - 1
                } else {
                    self.profile_index - 1
                };
            }
            Focus::Textarea => {}
        }
    }

    pub fn selected_team(&self) -> Option<&str> {
        if self.team_index == 0 {
            None // "Solo" is always index 0
        } else {
            Some(&self.team_names[self.team_index])
        }
    }

    pub fn selected_profile(&self) -> &str {
        &self.profile_options[self.profile_index]
    }

    pub fn description(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn try_launch(&mut self) {
        let desc = self.description();
        if desc.trim().is_empty() {
            self.error_message = Some("Task description required".to_string());
        } else {
            self.error_message = None;
            self.should_launch = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_focus_is_textarea() {
        let app = App::new(vec!["Solo".into()], vec!["solo".into()], vec!["balanced".into()]);
        assert_eq!(app.focus, Focus::Textarea);
    }

    #[test]
    fn tab_cycles_focus_forward() {
        let mut app = App::new(vec!["Solo".into()], vec!["solo".into()], vec!["balanced".into()]);
        app.next_focus();
        assert_eq!(app.focus, Focus::Team);
        app.next_focus();
        assert_eq!(app.focus, Focus::Profile);
        app.next_focus();
        assert_eq!(app.focus, Focus::Textarea);
    }

    #[test]
    fn shift_tab_cycles_focus_backward() {
        let mut app = App::new(vec!["Solo".into()], vec!["solo".into()], vec!["balanced".into()]);
        app.prev_focus();
        assert_eq!(app.focus, Focus::Profile);
    }

    #[test]
    fn team_selector_wraps() {
        let mut app = App::new(
            vec!["Solo".into(), "quick-fix".into(), "solo-sprint".into()],
            vec!["solo".into(), "quick-fix".into(), "solo-sprint".into()],
            vec!["balanced".into()],
        );
        app.focus = Focus::Team;
        assert_eq!(app.team_index, 0);
        app.select_next();
        assert_eq!(app.team_index, 1);
        app.select_next();
        assert_eq!(app.team_index, 2);
        app.select_next();
        assert_eq!(app.team_index, 0); // wraps
    }

    #[test]
    fn profile_selector_wraps() {
        let mut app = App::new(
            vec!["Solo".into()],
            vec!["solo".into()],
            vec!["fast".into(), "balanced".into(), "max".into()],
        );
        app.focus = Focus::Profile;
        assert_eq!(app.profile_index, 0);
        app.select_next();
        assert_eq!(app.profile_index, 1);
        app.select_next();
        assert_eq!(app.profile_index, 2);
        app.select_next();
        assert_eq!(app.profile_index, 0);
    }

    #[test]
    fn selected_team_none_for_solo() {
        let app = App::new(vec!["Solo".into(), "quick-fix".into()], vec!["solo".into(), "quick-fix".into()], vec!["balanced".into()]);
        assert_eq!(app.selected_team(), None);
    }

    #[test]
    fn selected_team_some_for_non_solo() {
        let mut app = App::new(vec!["Solo".into(), "quick-fix".into()], vec!["solo".into(), "quick-fix".into()], vec!["balanced".into()]);
        app.team_index = 1;
        assert_eq!(app.selected_team(), Some("quick-fix"));
    }

    #[test]
    fn selected_profile_returns_label() {
        let app = App::new(vec!["Solo".into()], vec!["solo".into()], vec!["fast".into(), "balanced".into(), "max".into()]);
        assert_eq!(app.selected_profile(), "fast");
    }

    #[test]
    fn description_empty_initially() {
        let app = App::new(vec!["Solo".into()], vec!["solo".into()], vec!["balanced".into()]);
        assert!(app.description().is_empty());
    }

    #[test]
    fn try_launch_rejects_empty_description() {
        let mut app = App::new(vec!["Solo".into()], vec!["solo".into()], vec!["balanced".into()]);
        app.try_launch();
        assert!(!app.should_launch);
        assert!(app.error_message.is_some());
    }

    #[test]
    fn try_launch_accepts_non_empty_description() {
        let mut app = App::new(vec!["Solo".into()], vec!["solo".into()], vec!["balanced".into()]);
        app.textarea.insert_str("fix the login bug");
        app.try_launch();
        assert!(app.should_launch);
        assert!(app.error_message.is_none());
    }

    #[test]
    fn select_prev_wraps_team() {
        let mut app = App::new(
            vec!["Solo".into(), "quick-fix".into()],
            vec!["solo".into(), "quick-fix".into()],
            vec!["balanced".into()],
        );
        app.focus = Focus::Team;
        assert_eq!(app.team_index, 0);
        app.select_prev();
        assert_eq!(app.team_index, 1); // wraps to last
    }
}
