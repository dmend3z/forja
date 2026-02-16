pub mod app;

use forja_core::error::Result;
use forja_core::models::profile::Profile;

pub struct TaskOutput {
    pub description: String,
    pub team: Option<String>,
    pub profile: Profile,
}

pub fn launch() -> Result<Option<TaskOutput>> {
    todo!("TUI not yet implemented")
}
