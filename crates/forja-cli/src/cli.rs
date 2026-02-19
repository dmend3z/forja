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
    /// Initialize forja (set up registry and install skills)
    #[command(
        long_about = "Initialize forja by creating a .forja/ directory, cloning the skills \
            registry, and installing all skills. By default, installs into the current project \
            (.forja/ + .claude/agents/). Use --global to install into ~/.forja/ + ~/.claude/agents/ \
            instead. Use --wizard for interactive setup (choose mode, phases, profile).",
        after_help = "\
EXAMPLES:
  forja init                          # Project-local install (default)
  forja init --global                 # Global install (~/.forja/)
  forja init --wizard                 # Interactive setup wizard
  forja init --registry-url <url>     # Use a custom registry"
    )]
    Init {
        /// Git URL for the skills registry
        #[arg(long)]
        registry_url: Option<String>,

        /// Run the interactive setup wizard (choose mode, phases, profile)
        #[arg(long)]
        wizard: bool,

        /// Install globally (~/.forja/ + ~/.claude/) instead of project-local
        #[arg(long)]
        global: bool,
    },

    /// Install a skill via symlink
    #[command(
        long_about = "Install a skill by creating symlinks from the registry into the project's \
            .claude/agents/ and .claude/commands/. Use --global to install into ~/.claude/ instead. \
            Skills are referenced by their full path: phase/tech/name.",
        after_help = "\
EXAMPLES:
  forja install code/rust/coder       # Install into project .claude/
  forja install --global code/rust/coder  # Install into ~/.claude/
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

        /// Install into ~/.claude/ instead of project-local .claude/
        #[arg(long)]
        global: bool,
    },

    /// Remove an installed skill
    #[command(
        long_about = "Remove a skill by deleting its symlinks from .claude/agents/ \
            and .claude/commands/. Use --global to remove from ~/.claude/ instead. \
            Prompts for confirmation unless --yes is passed.",
        after_help = "\
EXAMPLES:
  forja uninstall code/rust/coder     # Remove from project .claude/
  forja uninstall --global coder      # Remove from ~/.claude/
  forja uninstall coder -y            # Skip confirmation"
    )]
    Uninstall {
        /// Skill name or full path
        skill: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,

        /// Remove from ~/.claude/ instead of project-local .claude/
        #[arg(long)]
        global: bool,
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

    /// Generate project documentation (CLAUDE.md, AGENTS.md, README.md)
    #[command(
        long_about = "Generate or update project documentation by analyzing the codebase. \
            Auto-installs the doc-gen skill and launches Claude Code.",
        after_help = "\
EXAMPLES:
  forja docs                          # Generate all docs
  forja docs --scope claude-md        # Only CLAUDE.md
  forja docs --scope readme           # Only README.md
  forja docs --scope agents-md        # Only AGENTS.md"
    )]
    Docs {
        /// Scope to a single doc: claude-md, agents-md, readme
        #[arg(long)]
        scope: Option<String>,
    },

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
  forja plan                              # Open interactive TUI
  forja plan \"add user auth with JWT\"
  forja plan \"refactor the database layer\""
    )]
    Plan {
        /// Task description (omit to open interactive TUI)
        task: Option<String>,
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
        /// Task description (omit to open interactive TUI)
        task: Option<String>,

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
        long_about = "Execute a previously created plan phase by phase with checkpoints. \
            Defaults to the latest pending plan, or specify a plan ID. Use --resume to \
            continue from where a previous execution stopped.",
        after_help = "\
EXAMPLES:
  forja execute                       # Run the latest plan
  forja execute abc123                # Run a specific plan
  forja execute --resume              # Resume from last checkpoint
  forja execute --profile fast        # Use fast model profile"
    )]
    Execute {
        /// Plan ID (defaults to latest pending plan)
        #[arg()]
        plan_id: Option<String>,

        /// Model profile: fast, balanced, max
        #[arg(long, default_value = "balanced")]
        profile: String,

        /// Resume from last checkpoint (skip completed phases)
        #[arg(long)]
        resume: bool,
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

    /// Quick bug fix (shortcut for: forja task --team quick-fix)
    #[command(
        long_about = "Quick bug fix shortcut. Launches a quick-fix team (coder + deployer) \
            to fix the described issue.",
        after_help = "\
EXAMPLES:
  forja fix \"login button not responding\"
  forja fix \"null pointer in user service\" --profile fast"
    )]
    Fix {
        /// Bug description
        description: String,

        /// Model profile override
        #[arg(long)]
        profile: Option<String>,
    },

    /// Build a feature (shortcut for: forja task --team solo-sprint)
    #[command(
        long_about = "Feature build shortcut. Launches a solo-sprint team (coder-tester + reviewer) \
            to implement the described feature.",
        after_help = "\
EXAMPLES:
  forja build \"add user profile page\"
  forja build \"implement search API\" --profile max"
    )]
    Build {
        /// Feature description
        description: String,

        /// Model profile override
        #[arg(long)]
        profile: Option<String>,
    },

    /// Document decisions from recent changes
    #[command(
        long_about = "Extract and document significant decisions from recent git changes or \
            specific files. Auto-installs the chronicler skill and launches Claude Code to \
            write a structured decision log to docs/decisions/.",
        after_help = "\
EXAMPLES:
  forja chronicle                       # Extract decisions from recent git changes
  forja chronicle --from src/auth.rs    # Extract from specific file(s)"
    )]
    Chronicle {
        /// Extract decisions from specific file(s) instead of git changes
        #[arg(long)]
        from: Vec<String>,
    },

    /// Review uncommitted changes
    #[command(
        long_about = "Launch a code review of uncommitted changes. Auto-installs the reviewer \
            skill and launches Claude Code with a review prompt.",
        after_help = "\
EXAMPLES:
  forja review                          # Review all changes
  forja review --path src/              # Review only src/ changes"
    )]
    Review {
        /// Scope review to files matching this path filter
        #[arg(long)]
        path: Option<String>,

        /// Skip the decision chronicle step after review
        #[arg(long)]
        no_chronicle: bool,
    },

    /// Commit changes and create a PR
    #[command(
        long_about = "Ship your changes: commit with conventional commits, then create a PR. \
            Uses the committer and pr-creator skills sequentially.",
        after_help = "\
EXAMPLES:
  forja ship                            # Commit + PR (interactive)
  forja ship -m \"add auth endpoint\"     # Commit with hint + PR
  forja ship --commit-only              # Just commit, no PR"
    )]
    Ship {
        /// Commit message hint
        #[arg(short, long)]
        message: Option<String>,

        /// Only commit, don't create a PR
        #[arg(long)]
        commit_only: bool,

        /// Skip the decision chronicle step after ship
        #[arg(long)]
        no_chronicle: bool,
    },

    /// Validate skill structure and manifests
    #[command(
        long_about = "Lint skills for structural issues: missing manifests, invalid JSON, \
            malformed YAML frontmatter, naming conventions, and more.",
        after_help = "\
EXAMPLES:
  forja lint                            # Lint all skills in registry
  forja lint skills/code/rust/coder     # Lint a specific skill
  forja lint --warnings                 # Show warnings too"
    )]
    Lint {
        /// Path to a specific skill directory to lint
        path: Option<String>,

        /// Show warnings in addition to errors
        #[arg(long)]
        warnings: bool,
    },

    /// Scaffold a new skill
    #[command(
        long_about = "Create a new skill with the required directory structure and template files. \
            Interactive wizard by default, or use flags for scripting.",
        after_help = "\
EXAMPLES:
  forja new                             # Interactive wizard
  forja new my-skill --phase code --tech rust  # With flags
  forja new my-skill --phase code --tech rust --no-wizard"
    )]
    New {
        /// Skill name (kebab-case)
        name: Option<String>,

        /// Workflow phase (research, code, test, review, deploy)
        #[arg(long)]
        phase: Option<String>,

        /// Technology category (e.g. rust, nextjs, general)
        #[arg(long)]
        tech: Option<String>,

        /// Skip interactive prompts
        #[arg(long)]
        no_wizard: bool,
    },

    /// Show skill usage analytics
    #[command(
        long_about = "Display usage statistics: most-used skills, phase distribution, \
            recent activity, and installed-but-unused skills."
    )]
    Stats,

    /// Show skill changes since last update
    #[command(
        long_about = "Compare the registry before and after the last `forja update` to show \
            which skills were added, modified, or removed.",
        after_help = "\
EXAMPLES:
  forja update && forja diff            # Update then see changes"
    )]
    Diff,

    /// Reinstall modified skills after an update
    #[command(
        long_about = "Reinstall skills that were modified since the last `forja update`. \
            Re-creates symlinks to pick up agent/command changes.",
        after_help = "\
EXAMPLES:
  forja upgrade                         # Upgrade all modified installed skills
  forja upgrade coder                   # Upgrade a specific skill
  forja upgrade -y                      # Skip confirmation"
    )]
    Upgrade {
        /// Filter to a specific skill (partial match)
        skill: Option<String>,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Scan the codebase and recommend skills
    #[command(
        long_about = "Analyze the current project to detect technologies and recommend relevant \
            forja skills. Uses filesystem markers for instant results, with optional AI-powered \
            deep analysis via Claude CLI. Presents an interactive TUI for selecting which skills \
            to install.",
        after_help = "\
EXAMPLES:
  forja scan                            # Interactive scan with AI
  forja scan --basic                    # Deterministic only (no AI)
  forja scan --yes                      # Auto-install all recommended
  forja scan --json                     # Output JSON (for scripting)
  forja scan --all                      # Include already-installed skills"
    )]
    Scan {
        /// Auto-install all recommended skills without TUI
        #[arg(long, short = 'y')]
        yes: bool,

        /// Output scan results as JSON
        #[arg(long)]
        json: bool,

        /// Include already-installed skills in results
        #[arg(long)]
        all: bool,

        /// Skip AI analysis, use deterministic scan only
        #[arg(long)]
        basic: bool,
    },

    /// Validate all .forja/ files (schema, references, completeness)
    #[command(
        long_about = "Lint and validate all files in the .forja/ directory: schema validation, \
            reference integrity, completeness checks, and consistency rules. Must pass before \
            execution.",
        after_help = "\
EXAMPLES:
  forja validate                      # Validate the entire .forja/ framework"
    )]
    Validate,

    /// Spec-driven development pipeline
    #[command(
        long_about = "Manage implementation specs in .forja/specs/. Specs use YAML frontmatter \
            for structured metadata (status, track, acceptance criteria) and markdown bodies \
            for detailed context.",
        after_help = "\
EXAMPLES:
  forja specs list                    # List all specs with status
  forja specs show user-auth          # Display full spec details
  forja specs plan user-auth          # Generate execution plan from spec
  forja specs status                  # Show execution progress"
    )]
    Specs {
        #[command(subcommand)]
        command: SpecsCommands,
    },

    /// Manage work tracks
    #[command(
        long_about = "Work tracks group related specs into deliverable milestones. \
            Each track has a progress table linking to specs.",
        after_help = "\
EXAMPLES:
  forja tracks list                   # List all tracks with progress
  forja tracks show mvp               # Show track details and items"
    )]
    Tracks {
        #[command(subcommand)]
        command: TracksCommands,
    },

    /// Manage architecture decisions
    #[command(
        long_about = "Architecture Decision Records (ADRs) document significant technical \
            decisions linked to specs.",
        after_help = "\
EXAMPLES:
  forja decisions list                # List all decisions
  forja decisions show 001            # Show decision details"
    )]
    Decisions {
        #[command(subcommand)]
        command: DecisionsCommands,
    },

    /// View execution run history
    #[command(
        long_about = "Execution runs are logs of spec execution with full agent output. \
            Created automatically by `forja execute`.",
        after_help = "\
EXAMPLES:
  forja runs list                     # List all execution runs
  forja runs show run-20260218-auth   # Show run details and output"
    )]
    Runs {
        #[command(subcommand)]
        command: RunsCommands,
    },

    /// (Legacy alias for 'specs')
    #[command(hide = true)]
    Sparks {
        #[command(subcommand)]
        command: SparksCommands,
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

#[derive(Subcommand)]
pub enum SpecsCommands {
    /// List all specs with status
    List {
        /// Path to specs directory (defaults to .forja/specs/)
        #[arg(long)]
        path: Option<String>,
    },

    /// Display full spec details
    Show {
        /// Spec ID
        spec_id: String,
    },

    /// Generate execution plan from spec
    Plan {
        /// Spec ID
        spec_id: String,
    },

    /// Show execution progress
    Status {
        /// Spec ID (omit to show all specs)
        spec_id: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum TracksCommands {
    /// List all tracks with progress
    List,

    /// Show track details and item table
    Show {
        /// Track ID
        track_id: String,
    },
}

#[derive(Subcommand)]
pub enum DecisionsCommands {
    /// List all decisions
    List,

    /// Show decision details
    Show {
        /// Decision ID
        decision_id: String,
    },
}

#[derive(Subcommand)]
pub enum RunsCommands {
    /// List execution run history
    List,

    /// Show run details and output
    Show {
        /// Run ID (filename stem)
        run_id: String,
    },
}

/// Legacy alias — hidden, routes to the same code as Specs
#[derive(Subcommand)]
pub enum SparksCommands {
    /// List all specs with status
    List {
        /// Path to specs directory (defaults to docs/specs/)
        #[arg(long)]
        path: Option<String>,
    },

    /// Display full spec details
    Show {
        /// Spec ID
        spec_id: String,
    },

    /// Generate execution plan from spec
    Plan {
        /// Spec ID
        spec_id: String,
    },

    /// Execute a spec's plan
    Execute {
        /// Spec ID
        spec_id: String,

        /// Model profile: fast, balanced, max
        #[arg(long, default_value = "balanced")]
        profile: String,

        /// Resume from last checkpoint
        #[arg(long)]
        resume: bool,
    },

    /// Show execution progress
    Status {
        /// Spec ID (omit to show all specs)
        spec_id: Option<String>,
    },
}
