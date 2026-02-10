use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "forja",
    about = "Skills marketplace for Claude Code",
    version,
    after_help = "\
WORKFLOW PHASES:
  1. Research  Explore codebase, read docs, plan before coding
  2. Code      Write production-ready code following project patterns
  3. Test      TDD: write tests first, target 80%+ coverage
  4. Review    Check quality, security (OWASP), performance
  5. Deploy    Conventional commits, structured PRs, CI verification

Get started: forja guide"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize forja (create ~/.forja/, set up registry)
    #[command(
        long_about = "Initialize forja by creating the ~/.forja/ directory and setting up the \
            skills registry. On first run, clones the default registry. If a local skills/ \
            directory exists, creates a symlink instead (dev mode).",
        after_help = "\
EXAMPLES:
  forja init                          # Use default registry
  forja init --registry-url <url>     # Use a custom registry"
    )]
    Init {
        /// Git URL for the skills registry
        #[arg(long)]
        registry_url: Option<String>,

        /// Skip the interactive wizard and use global mode (~/.forja/)
        #[arg(long)]
        global: bool,
    },

    /// Install a skill via symlink
    #[command(
        long_about = "Install a skill by creating symlinks from the registry into \
            ~/.claude/agents/ and ~/.claude/commands/. Skills are referenced by their \
            full path: phase/tech/name.",
        after_help = "\
EXAMPLES:
  forja install code/rust/coder       # Install a specific skill
  forja install --all                 # Install every available skill
  forja install review/general/reviewer"
    )]
    Install {
        /// Skill path: phase/tech/skill (e.g. "code/nextjs/components")
        #[arg(conflicts_with = "all", required_unless_present = "all")]
        skill: Option<String>,

        /// Install all available skills
        #[arg(long)]
        all: bool,
    },

    /// Remove an installed skill
    #[command(
        long_about = "Remove a skill by deleting its symlinks from ~/.claude/agents/ \
            and ~/.claude/commands/. Prompts for confirmation unless --yes is passed.",
        after_help = "\
EXAMPLES:
  forja uninstall code/rust/coder     # Remove with confirmation
  forja uninstall coder -y            # Skip confirmation"
    )]
    Uninstall {
        /// Skill name or full path
        skill: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Search the catalog
    #[command(
        long_about = "Search the skills catalog by name, description, phase, or technology. \
            Results show install status and skill descriptions.",
        after_help = "\
EXAMPLES:
  forja search rust                   # Find Rust skills
  forja search \"code review\"          # Find review skills
  forja search nextjs                 # Find Next.js skills"
    )]
    Search {
        /// Search query (matches name, description, phase, tech)
        query: String,
    },

    /// List skills
    #[command(
        long_about = "List installed skills, or browse all available skills organized \
            by workflow phase with --available.",
        after_help = "\
EXAMPLES:
  forja list                          # Show installed skills
  forja list --available              # Browse all skills by phase"
    )]
    List {
        /// Show all available skills instead of just installed
        #[arg(long)]
        available: bool,
    },

    /// Update the registry (git pull)
    #[command(long_about = "Pull the latest skill definitions from the registry. \
            In dev mode (symlinked registry), this is a no-op.")]
    Update,

    /// Show skill details
    #[command(
        long_about = "Display detailed information about a skill: description, phase, \
            technology, included agents, slash commands, and install status.",
        after_help = "\
EXAMPLES:
  forja info code/rust/coder          # Full path
  forja info coder                    # Short name (if unique)"
    )]
    Info {
        /// Skill name or full path
        skill: String,
    },

    /// Show the 5 workflow phases and their skills
    Phases,

    /// Verify installation health
    #[command(
        long_about = "Run diagnostic checks on your forja installation: directory structure, \
            registry status, symlink health, catalog integrity, and team configuration. \
            Each check shows PASS/FAIL with remediation steps for failures."
    )]
    Doctor,

    /// Getting started guide -- learn the 5-phase workflow
    #[command(
        long_about = "Interactive guide to the forja 5-phase development workflow. \
            Shows commands, examples, and tips for each phase: Research, Code, Test, \
            Review, and Deploy.",
        after_help = "\
EXAMPLES:
  forja guide                         # Full guide (all phases)
  forja guide --phase code            # Just the Code phase
  forja guide --phase review          # Just the Review phase"
    )]
    Guide {
        /// Show only a specific phase (research, code, test, review, deploy)
        #[arg(long)]
        phase: Option<String>,
    },

    /// Create an implementation plan (launches Claude Code for interview + research)
    #[command(
        long_about = "Create an implementation plan by launching Claude Code in an \
            interactive interview. The plan is saved for later execution with `forja execute`.",
        after_help = "\
EXAMPLES:
  forja plan \"add user auth with JWT\"
  forja plan \"refactor the database layer\""
    )]
    Plan {
        /// Task description (e.g. "add user auth with JWT")
        task: String,
    },

    /// Run a task directly in Claude Code (no plan needed)
    #[command(
        long_about = "Run a task directly by launching Claude Code with the appropriate \
            skills. Optionally use a team for complex tasks or --print for non-interactive output.",
        after_help = "\
EXAMPLES:
  forja task \"fix the login bug\"
  forja task \"add API endpoint\" --team solo-sprint
  forja task \"explain the auth flow\" --print"
    )]
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
    #[command(
        long_about = "Execute a previously created plan. Defaults to the latest pending \
            plan, or specify a plan ID. Use --profile to control model selection.",
        after_help = "\
EXAMPLES:
  forja execute                       # Run the latest plan
  forja execute abc123                # Run a specific plan
  forja execute --profile fast        # Use fast model profile"
    )]
    Execute {
        /// Plan ID (defaults to latest pending plan)
        #[arg()]
        plan_id: Option<String>,

        /// Model profile: fast, balanced, max
        #[arg(long, default_value = "balanced")]
        profile: String,
    },

    /// Real-time dashboard for monitoring agent teams
    #[command(
        long_about = "Launch a real-time web dashboard that monitors active Claude Code agent \
            teams. Watches team configs, task progress, and inter-agent messages, streaming \
            updates live to your browser via SSE.",
        after_help = "\
EXAMPLES:
  forja monitor                     # Start on default port 3030
  forja monitor --port 8080         # Use custom port
  forja monitor --no-open           # Don't auto-open browser"
    )]
    Monitor {
        /// Port to bind the dashboard server
        #[arg(long, default_value = "3030")]
        port: u16,

        /// Don't auto-open the browser
        #[arg(long)]
        no_open: bool,
    },

    /// Manage multi-agent teams
    #[command(
        long_about = "Create, configure, and manage multi-agent teams for complex tasks. \
            Teams coordinate multiple AI agents with different roles.",
        after_help = "\
PRESETS:
  full-product   5 agents: researcher, coder, tester, reviewer, deployer
  solo-sprint    2 agents: coder-tester + reviewer (medium features)
  quick-fix      2 agents: coder + deployer (hotfixes)

EXAMPLES:
  forja team preset solo-sprint       # Create from preset
  forja team create my-team           # Interactive wizard
  forja team list                     # List all teams"
    )]
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
