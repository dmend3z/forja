# Architecture

Rust CLI for managing Claude Code agents. Clap 4 for arg parsing, serde for serialization, thiserror for errors. No async runtime except for the monitor module, which uses Tokio and Axum for real-time dashboard serving.

## Module Map

```
src/
├── main.rs              # Entry point: parses CLI, dispatches to commands, handles top-level errors
├── cli.rs               # Clap derive structs: Cli, Commands, TeamCommands
├── error.rs             # ForjaError (thiserror) + Result<T> type alias
├── paths.rs             # ForjaPaths: all filesystem paths (~/.forja/*, ~/.claude/*), ForjaMode (Project vs Global)
├── settings.rs          # Read/write ~/.claude/settings.json (agent teams env var)
├── output.rs            # Terminal output formatting and colored messages
├── tips.rs              # Random tips for status dashboard
├── wizard.rs            # Interactive init wizard (3 steps)
│
├── models/              # Data types (no business logic beyond ser/de)
│   ├── phase.rs         # Phase enum: Research, Code, Test, Review, Deploy, Teams
│   ├── skill.rs         # Skill struct + ContentType enum (Agent, Skill, Command)
│   ├── plugin.rs        # PluginJson: skill manifest format (skill.json + legacy plugin.json)
│   ├── registry.rs      # Registry: in-memory skill index with find_by_id() and search()
│   ├── state.rs         # ForjaState, TeamEntry, TeamMember + load/save/migration
│   ├── profile.rs       # Profile enum (Fast, Balanced, Max) + model resolution per phase
│   ├── plan.rs          # PlanMetadata, PlanPhase, PlanStatus + find_latest_pending() + find_plan_for_spec()
│   ├── config.rs        # ForjaConfig: version, mode, project_name, registry URL, local flag
│   ├── active_project.rs # Active project tracking for project-scoped state
│   ├── spec.rs          # SpecFile, SpecStatus, SpecFrontmatter + parse/discover/find/build_task_description
│   └── claude.rs        # Claude Code integration models
│
├── registry/            # Catalog scanning and git operations
│   ├── catalog.rs       # scan(): walks skills/<phase>/<tech>/<name>/, builds Registry
│   └── git.rs           # clone() and pull() via git subprocess
│
├── symlink/             # Symlink lifecycle management
│   ├── manager.rs       # SymlinkManager: install/uninstall/verify + state persistence wrappers
│   ├── auto_install.rs  # Auto-install agents on init
│   └── sync.rs          # Symlink sync operations
│
└── commands/            # One file per CLI subcommand
    ├── init.rs          # Create ~/.forja/, clone or symlink registry, auto-install all skills, detect stack
    ├── install.rs       # Scan catalog, create symlinks, update state (single, --all, or quiet)
    ├── uninstall.rs     # Remove symlinks by prefix, update state
    ├── search.rs        # Scan catalog, filter by query, display results
    ├── list.rs          # Show installed or all available skills grouped by phase
    ├── update.rs        # git pull on registry
    ├── info.rs          # Show skill details (phase, tech, description, content types)
    ├── guide.rs         # Show workflow phase guide (Research → Code → Test → Review → Deploy)
    ├── doctor.rs        # Health check: paths, symlinks, catalog count, teams, env var
    ├── status.rs        # No-args status: welcome pitch (not initialized) or dashboard (initialized)
    ├── plan.rs          # Load forja-plan command template, launch Claude Code session
    ├── execute.rs       # Load plan JSON, auto-install agents, build prompt, launch Claude Code
    ├── task.rs          # Direct task execution: solo or team mode with interactive picker
    ├── team.rs          # Team CRUD: create (wizard), preset, list, info, delete
    ├── sparks.rs        # Spec-driven pipeline: list/show/plan/execute/status for specs
    └── monitor/         # Real-time dashboard for agent teams
        ├── mod.rs       # forja monitor entry point
        ├── server.rs    # Axum HTTP server
        ├── watcher.rs   # Filesystem watcher for team/task changes
        ├── state.rs     # Monitor state management
        └── events.rs    # SSE event streaming
```

## Data Flow

### `forja install <skill-id>`

```
main.rs
  Cli::parse() → Commands::Install { skill }
    ↓
commands/install.rs::run(skill_path)
    ↓
  1. ForjaPaths::ensure_initialized()     → validates ~/.forja/ exists
  2. load_installed_ids(&paths.state)      → reads state.json → Vec<String>
  3. check if skill already in installed_ids → error if duplicate
  4. catalog::scan(&paths.registry, &ids)  → walks skills/ dir, returns Registry
  5. registry.find_by_id(skill_path)       → Option<&Skill> (error if None)
  6. SymlinkManager::new(agents_dir, commands_dir)
  7. manager.install(skill)                → symlinks agents/*.md + commands/*.md
     - link name: forja--{phase}--{tech}--{name}--{file}.md
     - target dir: ~/.claude/agents/ and ~/.claude/commands/
  8. installed_ids.push(skill_path)
  9. save_installed_ids(&paths.state, &ids) → load full state, update .installed, write JSON
```

### `forja init`

```
commands/init.rs::run(registry_url)
    ↓
  1. ForjaPaths::new()                 → compute paths (does NOT require ~/.forja/ to exist)
  2. If cwd has skills/ dir:           → symlink cwd → ~/.forja/registry (local dev)
     Else:                             → git clone --depth 1 → ~/.forja/registry
  3. Write config.json                 → { registry_url, local: bool }
  4. Create ~/.forja/plans/
  5. save_installed_ids([], state)      → empty state.json
  6. Ensure ~/.claude/agents/
  7. install_all_quiet(&paths)          → scan catalog, install all agents, save state
  8. detect_stack(&cwd)                → check for Cargo.toml, package.json, next.config.*, etc.
  9. Print minimal output              → checkmarks + skill count + detected stack + "Try:" hint
```

### `forja` (no args)

```
commands/status.rs::run()
    ↓
  1. ForjaPaths::new()
  2. If ~/.forja/ does not exist:     → print welcome pitch with "forja init" prompt
  3. Else:
     a. load_installed_ids()          → count installed agents
     b. catalog::scan()               → count total available agents
     c. SymlinkManager::verify()      → check symlink health
     d. Print status dashboard        → skills count + health status + "forja task" hint
```

### `forja task` (team mode)

```
commands/task.rs::run(task, print, team, profile)
    ↓
  1. Verify `claude` CLI is available
  2. If --team flag:                   → run_team() directly
     If --print without --team:        → run_simple() (non-interactive)
     Else:                             → prompt_team_selection() with dialoguer
  3. run_team():
     a. Resolve team members from state (or preset fallback)
     b. Resolve profile (--profile flag > team's configured profile)
     c. Enable CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS if missing
     d. auto_install_agents() → scan catalog, install missing skills
     e. build_team_prompt() → structured markdown with agent references
     f. Launch: claude -- <prompt>
```

### `forja execute`

```
commands/execute.rs::run(plan_id, profile)
    ↓
  1. Find plan: by ID or find_latest_pending() (newest .json with status "pending")
  2. Override profile if non-default
  3. Auto-install missing agents referenced in plan.agents[]
  4. Enable agent teams env var if missing
  5. Read plan .md file (companion to .json)
  6. build_execution_prompt() → structured phases, quality gates, agent mapping
  7. Launch: claude -- <prompt>
  8. On success: mark plan status as "executed", save JSON
```

### `forja sparks plan <spec-id>`

```
commands/sparks.rs::plan(spec_id)
    ↓
  1. spec::find_spec(docs/specs/, spec_id)   → loads and parses the spec .md file
  2. spec::build_task_description(&spec)      → concatenates title, description, requirements, constraints, criteria, body
  3. Load forja-plan template from registry   → skills/research/planning/forja-plan/commands/forja-plan.md
  4. frontmatter::strip_frontmatter(&template) → removes YAML frontmatter from template
  5. Replace $ARGUMENTS with task description → inject spec content into plan prompt
  6. Append "source_spec" instruction         → tells Claude to link the plan back to the spec ID
  7. Launch: claude -- <prompt>               → generates plan JSON + plan .md in ~/.forja/plans/
```

### `forja sparks execute <spec-id>`

```
commands/sparks.rs::execute(spec_id, profile, resume)
    ↓
  1. spec::find_spec(docs/specs/, spec_id)      → validate spec exists
  2. plan::find_plan_for_spec(plans_dir, spec_id) → find linked plan by source_spec field
  3. auto_install_missing(&paths, &skill_ids)     → install any missing agents
  4. settings::enable_teams_env_var() if missing  → ensure CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1
  5. If no phases:                                → exec_monolithic(): single claude session
     If phases exist:                             → exec_phased():
       a. Load or initialize checkpoint (--resume to continue)
       b. For each phase:
          - Skip if completed or dependency-failed
          - run_phase_with_retry(): 2 attempts, then handle_phase_failure() → Retry/Skip/Abort
          - run_quality_gates(): cargo test + cargo clippy (non-blocking)
       c. Save checkpoint at every state transition
  6. On all phases complete: mark plan as executed
```

### `forja monitor`

```
commands/monitor/mod.rs::run(port, auto_open)
    ↓
  1. Ensure ~/.claude/teams/ and ~/.claude/tasks/ directories exist
  2. Create DashboardState (state.rs) with broadcast channel (capacity 256)
  3. initial_scan():
     a. scan_teams() → read ~/.claude/teams/*/config.json → TeamSnapshot
        - also parse ~/.claude/teams/*/inboxes/*.json → MessageGroupSnapshot
     b. scan_tasks() → read ~/.claude/tasks/*/*.json → TaskSnapshot
        - maps task dirs to teams via leadSessionId from config.json
  4. Spawn filesystem watcher (watcher.rs) → notify crate watches ~/.claude/teams/ and ~/.claude/tasks/
  5. Start Axum HTTP server (server.rs):
     a. GET /                → embedded index.html (rust_embed from assets/)
     b. GET /assets/{*path}  → embedded static assets
     c. GET /api/events      → SSE stream (events.rs):
        - sends full Snapshot on connect, then streams live DashboardEvents
        - keepalive heartbeat every 15s
  6. If auto_open (default true): open::that(url)
  7. Graceful shutdown on Ctrl+C via tokio::signal
```

## Key Types

### `ForjaPaths` (`src/paths.rs`)

All filesystem paths used by the CLI. Computed on every invocation with auto-detection of project vs global mode.

| Field              | Path                        |
|--------------------|-----------------------------|
| `mode`             | `ForjaMode` (Global/Project)|
| `project_root`     | `Some(PathBuf)` in Project mode, `None` in Global |
| `forja_root`       | `~/.forja/` or `<project>/.forja/` |
| `registry`         | `{forja_root}/registry/`    |
| `config`           | `{forja_root}/config.json`  |
| `state`            | `{forja_root}/state.json`   |
| `plans`            | `{forja_root}/plans/`       |
| `claude_dir`       | `~/.claude/`                |
| `claude_agents`    | `~/.claude/agents/`         |
| `claude_commands`  | `~/.claude/commands/`       |

`claude_dir`, `claude_agents`, and `claude_commands` always resolve to `~/.claude/` regardless of mode.

Constructors (all return `Result<Self>`, can fail with `NoHomeDir`):
- `resolve()` → walks up from cwd looking for `.forja/config.json`, falls back to global
- `global()` → forces `~/.forja/` mode
- `from_project(root)` → forces `<root>/.forja/` mode
- `new()` → alias for `resolve()`
- `ensure_initialized()` → `resolve()` + errors if `forja_root` doesn't exist

Methods:
- `display_name()` → project directory name in Project mode, `"global"` in Global mode
- `global_forja_root()` → always returns `~/.forja/` regardless of mode (static)

### `ForjaMode` (`src/paths.rs`)

```rust
enum ForjaMode { Global, Project }
```

Determines whether forja operates on the global `~/.forja/` or a project-scoped `.forja/`. Auto-detected by `detect_project_root()` which walks up from the current working directory looking for `.forja/config.json`. If found, uses Project mode rooted at that directory; otherwise falls back to Global.

### `ForjaError` (`src/error.rs`)

Centralized error enum with `thiserror`. Variants: `Io`, `Json`, `NoHomeDir`, `NotInitialized`, `SkillNotFound`, `AlreadyInstalled`, `NotInstalled`, `Git`, `TeamNotFound`, `TeamAlreadyExists`, `InvalidSettings`, `PromptCancelled`, `Dialoguer`, `NoPlansFound`, `PlanNotFound`, `ClaudeCliNotFound`, `AmbiguousSkillName`, `PhaseExecutionFailed`, `Monitor`, `NoChangesToReview`, `InvalidSkillName`, `LintFailed`, `InvalidArgument`, `Yaml`, `InvalidSpec`, `SpecNotFound`.

Module defines `type Result<T> = std::result::Result<T, ForjaError>`.

Methods:
- `hint()` → returns an actionable message for each variant (e.g., `NotInitialized` → `"Run: forja init"`)
- `exit_code()` → returns a specific exit code per category: `2` (not initialized), `3` (not found), `4` (IO/JSON), `5` (monitor), `1` (all others)

### `ForjaState` (`src/models/state.rs`)

```rust
struct ForjaState {
    version: u32,              // always 2
    installed: Vec<String>,    // skill IDs like "code/general/feature"
    teams: HashMap<String, TeamEntry>,
    active_profile: Option<String>,
}
```

`load_state()` handles migration from v1 format (bare `Vec<String>`) to the v2 struct. `save_state()` writes pretty-printed JSON.

### `Registry` (`src/models/registry.rs`)

In-memory index of all available agents. Built fresh by `catalog::scan()` on every command invocation. Methods: `find_by_id(&str)`, `search(&str)` (matches against id, name, description, phase, tech).

### `Skill` (`src/models/skill.rs`)

```rust
struct Skill {
    id: String,                    // "phase/tech/name"
    name: String,                  // from skill manifest
    description: String,           // from skill manifest
    phase: Phase,
    tech: String,
    path: PathBuf,                 // absolute path to skill directory
    installed: bool,               // set during catalog scan from state
    content_types: Vec<ContentType>, // Agent, Skill, Command (detected from subdirs)
}
```

### `PluginJson` (`src/models/plugin.rs`)

Serde struct for skill manifest JSON. Scanner reads `skill.json` first, then legacy `.claude-plugin/plugin.json`. Fields: `name`, `description`, `version?`, `author?`, `license?`, `keywords?`.

### `Phase` (`src/models/phase.rs`)

```rust
enum Phase { Research, Code, Test, Review, Deploy, Teams }
```

`is_thinking_phase()` returns `true` for Research and Review (used by Profile model resolution).

### `Profile` (`src/models/profile.rs`)

```rust
enum Profile { Fast, Balanced, Max }
```

`resolve_model(phase)` maps profile + phase to a model string:
- **Fast**: all sonnet
- **Balanced**: opus for thinking phases (Research, Review), sonnet otherwise
- **Max**: all opus

### `PlanMetadata` (`src/models/plan.rs`)

```rust
struct PlanMetadata {
    id: String,                // "YYYYMMDD-HHMMSS-slug"
    created: String,
    status: PlanStatus,        // Pending | Executed | Archived
    task: String,
    team_size: String,         // preset name: full-product, solo-sprint, quick-fix, refactor
    profile: String,
    agents: Vec<PlanAgent>,    // { skill_id, role }
    stack: Option<PlanStack>,  // { language, framework? }
    quality_gates: Vec<String>,
    phases: Vec<PlanPhase>,    // { name, agent_role, files_to_create/modify, instructions, depends_on }
}
```

`find_latest_pending()` scans `~/.forja/plans/`, sorts by filename (timestamp prefix), returns newest with status `Pending`.

`find_plan_for_spec(plans_dir, spec_id)` scans all plan JSON files and returns the most recent one with `source_spec` matching the given spec ID.

### `SpecFile` (`src/models/spec.rs`)

```rust
struct SpecFile {
    frontmatter: SpecFrontmatter,  // id, title, description, priority?, tags, requirements, constraints, success_criteria
    body: String,                  // markdown content after frontmatter
    status: SpecStatus,            // Draft | Planning | Ready | Executing | Complete | Failed
}
```

Parsed from markdown files in `docs/specs/` via `parse_spec()` which splits YAML frontmatter from the body. Discovery functions:
- `discover_specs(dir)` — finds all `.md` files in a directory, returns sorted by ID
- `find_spec(dir, id)` — discovers all specs then filters by ID
- `load_spec(path)` — loads a single spec file from disk
- `build_task_description(spec)` — concatenates all spec fields into a structured prompt for plan generation

### `ForjaConfig` (`src/models/config.rs`)

```rust
struct ForjaConfig {
    version: u32,              // always 2
    mode: ForjaMode,           // Global or Project
    project_name: Option<String>, // directory name in Project mode
    registry_url: String,      // default: "https://github.com/dmend3z/forja.git"
    local: bool,               // true when registry is a local symlink
}
```

Stored at `{forja_root}/config.json`. Backward-compatible: missing `version`/`mode` fields get sane defaults via serde.

### `ActiveProject` (`src/models/active_project.rs`)

```rust
struct ActiveProject {
    project_name: String,      // directory name
    project_root: PathBuf,     // absolute path to project
    synced_at: u64,            // unix timestamp of last sync
}
```

Stored at `~/.forja/active_project.json` (always global). Tracks which project currently owns the `~/.claude/agents/` symlinks, enabling project switching without stale symlinks.

### Claude Code Integration Models (`src/models/claude.rs`)

Read-only serde structs for parsing Claude Code's internal files. Used by the monitor module.

- **`ClaudeTeamConfig`** — parses `~/.claude/teams/<name>/config.json`: team name, description, created_at, lead_agent_id, lead_session_id, members list
- **`ClaudeTeamMember`** — agent_id, name, agent_type, model, color, joined_at, prompt
- **`ClaudeInboxMessage`** — parses `~/.claude/teams/<name>/inboxes/<member>.json`: from, text, timestamp, color, read
- **`ClaudeTask`** — parses `~/.claude/tasks/<team-id>/<n>.json`: id, subject, description, active_form, status, owner, blocks, blocked_by

### `DashboardState` (`src/commands/monitor/state.rs`)

Thread-safe state container for the monitor dashboard. Holds `Arc<RwLock<...>>` maps of teams, tasks, and messages indexed by team name. Uses a `broadcast::Sender<DashboardEvent>` to push updates to SSE subscribers.

### `DashboardEvent` (`src/commands/monitor/events.rs`)

```rust
enum DashboardEvent {
    Snapshot { teams, tasks, messages },  // sent on initial SSE connection
    TeamUpdated { team },
    TeamDeleted { team_name },
    TaskUpdated { team_name, task },
    TaskDeleted { team_name, task_id },
    MessageReceived { team_name, recipient, message },
    Heartbeat,
}
```

Serialized as JSON with `#[serde(tag = "type")]` and streamed via SSE to dashboard clients.

## Design Decisions

### Sync-first with async monitor

All core operations are synchronous: filesystem reads, git subprocesses, and interactive prompts (dialoguer). There is no network I/O beyond shelling out to `git`. This keeps the binary small and the code simple.

The monitor module is the exception: it uses Tokio and Axum to serve a real-time dashboard via HTTP and Server-Sent Events (SSE). This is isolated to the `commands/monitor/` subtree and does not affect the rest of the CLI.

### Scan-on-demand catalog (no cache)

`catalog::scan()` walks the `skills/<phase>/<tech>/<name>/` directory tree and reads every skill manifest (`skill.json`, with legacy `.claude-plugin/plugin.json` fallback) on every invocation. No index file, no cache. With 31 skills (24 individual skills + 7 team configs) the scan completes in under 50ms, making caching unnecessary complexity.

### Symlink prefix `forja--`

All symlinks created by forja are prefixed with `forja--`. The full naming pattern is:

```
forja--{phase}--{tech}--{name}--{filename}.md
```

Slashes in the skill ID (`code/general/feature`) are replaced with `--`. This avoids collisions with user-created agents/commands, and enables `uninstall` and `doctor` to identify forja-managed symlinks by prefix without maintaining a separate manifest.

### State tracked in `~/.forja/state.json`

A single JSON file stores installed agent IDs, team configurations, and the active profile. The `save_installed_ids()` wrapper in `symlink/manager.rs` loads the full state before writing, ensuring that updating the installed list does not clobber team data.

### State migration (v1 to v2)

The original state format was a bare `Vec<String>` of agent IDs. When `load_state()` encounters this format, it transparently wraps it into a `ForjaState` with `version: 2` and empty teams. This migration happens in-memory on read; the v2 format is written back on the next state save.

### Local dev mode

`forja init` detects if the current working directory contains a `skills/` folder. If so, it creates a symlink from `~/.forja/registry` to the CWD instead of cloning from git. This enables development against the monorepo without duplicating the agent catalog.

### Teams env var management

Agent teams require `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` in `~/.claude/settings.json`. The `settings` module reads and writes this file, preserving existing settings. Team commands auto-enable this env var when needed.

### Commands slash command naming

Team slash commands are written to `~/.claude/commands/` with the pattern `forja--team--{name}.md`. These are separate from agent symlinks and contain structured markdown (frontmatter + orchestration instructions) generated by the `team.rs` module.

### Auto-install on init

`forja init` installs all agents automatically after setting up the registry. The `install_all_quiet()` function in `install.rs` reuses the same catalog scan and symlink logic as `install --all` but returns `(installed, skipped)` counts instead of printing per-agent output. This eliminates the manual browse-choose-install loop.

### Stack detection

`detect_stack()` in `init.rs` checks the cwd for framework files (`next.config.*`, `nuxt.config.*`, etc.) and language markers (`Cargo.toml`, `go.mod`, `tsconfig.json`, etc.). Two-layer approach: frameworks first, then languages, joined with " + ". Returns `None` when no recognized files exist.

### Project vs global mode

`ForjaPaths::resolve()` walks up from the current working directory looking for `.forja/config.json`. If found, it uses Project mode rooted at that directory; otherwise it falls back to Global (`~/.forja/`). Project mode enables per-project team configurations and plans while sharing the global `~/.claude/agents/` installation. The `ActiveProject` struct tracks which project currently owns the agent symlinks, preventing stale links when switching projects.

### No-args contextual status

Running `forja` with no subcommand shows a status dashboard (initialized) or welcome pitch (not initialized). Implemented by making the clap `command` field `Option<Commands>`. `--help` is unaffected since clap handles it before our code.

### Sparks: spec-to-plan linkage

Specs are linked to plans via the `source_spec` field on `PlanMetadata`. When `forja sparks plan` generates a plan, it injects an instruction for Claude to include `"source_spec": "<spec-id>"` in the plan JSON. The `find_plan_for_spec()` function scans the plans directory in reverse chronological order and returns the first match, so the most recent plan wins when a spec has been re-planned.

### Sparks: retry-then-ask error handling

Phase execution uses a two-tier retry strategy: first failure retries automatically (no user interaction); second failure pauses and prompts the user with three options via `dialoguer::Select`: Retry (fresh attempt), Skip (mark as skipped, continue), or Abort (halt execution). This balances resilience (transient failures auto-recover) with control (persistent failures require a human decision). Checkpoints are saved at every state transition so `--resume` works after any interruption.

### Sparks: non-blocking quality gates

After each completed phase, `cargo test --workspace` and `cargo clippy --workspace` run as quality gates. Results are reported (pass/fail with colored indicators) but do not block execution. The rationale: the user already chose to proceed past any phase failures, and test regressions may be addressed by subsequent phases.

### Real-time monitoring

`forja monitor` starts a local Axum HTTP server with SSE streaming for real-time team activity visibility. The watcher uses the `notify` crate to observe `~/.claude/teams/` (team configs and inboxes) and `~/.claude/tasks/` (task JSON files), broadcasting `DashboardEvent`s to connected SSE clients. Static assets (HTML/CSS/JS) are embedded in the binary via `rust_embed`. This enables multi-terminal workflows where one terminal runs agents while another monitors progress.
