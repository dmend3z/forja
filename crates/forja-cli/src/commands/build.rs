use forja_core::error::Result;

/// Build a feature shortcut — delegates to `task::run()` with the `solo-sprint` team.
pub fn run(description: &str, profile: Option<&str>) -> Result<()> {
    crate::commands::task::run(Some(description), false, Some("solo-sprint"), profile)
}
