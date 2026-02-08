use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "forja", about = "Skills marketplace for Claude Code", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize forja (create ~/.forja/, set up registry)
    Init {
        /// Git URL for the skills registry
        #[arg(long)]
        registry_url: Option<String>,
    },

    /// Install a skill via symlink
    Install {
        /// Skill path: phase/tech/skill (e.g. "code/nextjs/components")
        #[arg(conflicts_with = "all", required_unless_present = "all")]
        skill: Option<String>,

        /// Install all available skills
        #[arg(long)]
        all: bool,
    },

    /// Remove an installed skill
    Uninstall {
        /// Skill name or full path
        skill: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Search the catalog
    Search {
        /// Search query (matches name, description, phase, tech)
        query: String,
    },

    /// List skills
    List {
        /// Show all available skills instead of just installed
        #[arg(long)]
        available: bool,
    },

    /// Update the registry (git pull)
    Update,

    /// Show skill details
    Info {
        /// Skill name or full path
        skill: String,
    },

    /// Show the 5 workflow phases and their skills
    Phases,

    /// Verify installation health
    Doctor,

    /// Create an implementation plan (launches Claude Code for interview + research)
    Plan {
        /// Task description (e.g. "add user auth with JWT")
        task: String,
    },

    /// Run a task directly in Claude Code (no plan needed)
    Task {
        /// Task description (e.g. "fix the login bug")
        task: String,

        /// Run in non-interactive mode (output to stdout)
        #[arg(long)]
        print: bool,

        /// Optional team name (configured or preset: full-product, solo-sprint, quick-fix)
        #[arg(long)]
        team: Option<String>,

        /// Model profile override (only with --team)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Execute the latest plan (created by /forja-plan in Claude Code)
    Execute {
        /// Plan ID (defaults to latest pending plan)
        #[arg()]
        plan_id: Option<String>,

        /// Model profile: fast, balanced, max
        #[arg(long, default_value = "balanced")]
        profile: String,
    },

    /// Manage multi-agent teams
    Team {
        #[command(subcommand)]
        command: TeamCommands,
    },
}

#[derive(Subcommand)]
pub enum TeamCommands {
    /// Create a custom team via interactive wizard
    Create {
        /// Team name (used for the slash command)
        name: String,
    },

    /// Create a team from a built-in preset
    Preset {
        /// Preset name: full-product, solo-sprint, quick-fix
        name: String,

        /// Model profile: fast, balanced, max
        #[arg(long, default_value = "balanced")]
        profile: String,
    },

    /// List configured teams
    List,

    /// Show team details and model assignments
    Info {
        /// Team name
        name: String,
    },

    /// Delete a team and its slash command
    Delete {
        /// Team name
        name: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
}
