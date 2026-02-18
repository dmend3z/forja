use crate::error::Result;

/// Quick bug fix shortcut — delegates to `task::run()` with the `quick-fix` team.
pub fn run(description: &str, profile: Option<&str>) -> Result<()> {
    crate::commands::task::run(description, false, Some("quick-fix"), profile)
}
