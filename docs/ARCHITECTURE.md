# Architecture

Rust CLI for managing Claude Code skills. Clap 4 for arg parsing, serde for serialization, thiserror for errors. No async runtime -- all operations are filesystem I/O and git subprocesses.

## Module Map

```
src/
├── main.rs              # Entry point: parses CLI, dispatches to commands, handles top-level errors
├── cli.rs               # Clap derive structs: Cli, Commands, TeamCommands
├── error.rs             # ForjaError (thiserror) + Result<T> type alias
├── paths.rs             # ForjaPaths: all filesystem paths (~/.forja/*, ~/.claude/*)
├── settings.rs          # Read/write ~/.claude/settings.json (agent teams env var)
│
├── models/              # Data types (no business logic beyond ser/de)
│   ├── phase.rs         # Phase enum: Research, Code, Test, Review, Deploy, Teams
│   ├── skill.rs         # Skill struct + ContentType enum (Agent, Skill, Command)
│   ├── plugin.rs        # PluginJson: mirrors .claude-plugin/plugin.json format
│   ├── registry.rs      # Registry: in-memory skill index with find_by_id() and search()
│   ├── state.rs         # ForjaState, TeamEntry, TeamMember + load/save/migration
│   ├── profile.rs       # Profile enum (Fast, Balanced, Max) + model resolution per phase
│   └── plan.rs          # PlanMetadata, PlanPhase, PlanStatus + find_latest_pending()
│
├── registry/            # Catalog scanning and git operations
│   ├── catalog.rs       # scan(): walks skills/<phase>/<tech>/<name>/, builds Registry
│   └── git.rs           # clone() and pull() via git subprocess
│
├── symlink/             # Symlink lifecycle management
│   └── manager.rs       # SymlinkManager: install/uninstall/verify + state persistence wrappers
│
└── commands/            # One file per CLI subcommand
    ├── init.rs          # Create ~/.forja/, clone or symlink registry, auto-install all skills, detect stack
    ├── install.rs       # Scan catalog, create symlinks, update state (single, --all, or quiet)
    ├── uninstall.rs     # Remove symlinks by prefix, update state
    ├── search.rs        # Scan catalog, filter by query, display results
    ├── list.rs          # Show installed or all available skills grouped by phase
    ├── update.rs        # git pull on registry
    ├── info.rs          # Show skill details (phase, tech, description, content types)
    ├── phases.rs        # Display the 5+1 workflow phases with descriptions
    ├── doctor.rs        # Health check: paths, symlinks, catalog count, teams, env var
    ├── status.rs        # No-args status: welcome pitch (not initialized) or dashboard (initialized)
    ├── plan.rs          # Load forja-plan command template, launch Claude Code session
    ├── execute.rs       # Load plan JSON, auto-install agents, build prompt, launch Claude Code
    ├── task.rs          # Direct task execution: solo or team mode with interactive picker
    └── team.rs          # Team CRUD: create (wizard), preset, list, info, delete
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
  7. install_all_quiet(&paths)          → scan catalog, install all skills, save state
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
     a. load_installed_ids()          → count installed skills
     b. catalog::scan()               → count total available skills
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

## Key Types

### `ForjaPaths` (`src/paths.rs`)

All filesystem paths used by the CLI. Computed from `$HOME` on every invocation.

| Field            | Path                      |
|------------------|---------------------------|
| `forja_root`     | `~/.forja/`               |
| `registry`       | `~/.forja/registry/`      |
| `config`         | `~/.forja/config.json`    |
| `state`          | `~/.forja/state.json`     |
| `plans`          | `~/.forja/plans/`         |
| `claude_dir`     | `~/.claude/`              |
| `claude_agents`  | `~/.claude/agents/`       |
| `claude_commands` | `~/.claude/commands/`    |

Two constructors: `new()` (always succeeds) and `ensure_initialized()` (errors if `~/.forja/` missing).

### `ForjaError` (`src/error.rs`)

Centralized error enum with `thiserror`. Variants: `Io`, `Json`, `NoHomeDir`, `NotInitialized`, `SkillNotFound`, `AlreadyInstalled`, `NotInstalled`, `Git`, `TeamNotFound`, `TeamAlreadyExists`, `PromptCancelled`, `Dialoguer`, `NoPlansFound`, `PlanNotFound`, `ClaudeCliNotFound`.

Module defines `type Result<T> = std::result::Result<T, ForjaError>`.

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

In-memory index of all available skills. Built fresh by `catalog::scan()` on every command invocation. Methods: `find_by_id(&str)`, `search(&str)` (matches against id, name, description, phase, tech).

### `Skill` (`src/models/skill.rs`)

```rust
struct Skill {
    id: String,                    // "phase/tech/name"
    name: String,                  // from plugin.json
    description: String,           // from plugin.json
    phase: Phase,
    tech: String,
    path: PathBuf,                 // absolute path to skill directory
    installed: bool,               // set during catalog scan from state
    content_types: Vec<ContentType>, // Agent, Skill, Command (detected from subdirs)
}
```

### `PluginJson` (`src/models/plugin.rs`)

Serde struct mirroring `.claude-plugin/plugin.json`. Fields: `name`, `description`, `version?`, `author?`, `license?`, `keywords?`.

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

## Design Decisions

### No async runtime

All operations are synchronous: filesystem reads, git subprocesses, and interactive prompts (dialoguer). There is no network I/O beyond shelling out to `git`. This keeps the binary small and the code simple.

### Scan-on-demand catalog (no cache)

`catalog::scan()` walks the `skills/<phase>/<tech>/<name>/` directory tree and reads every `plugin.json` on every invocation. No index file, no cache. With <100 skills the scan completes in under 50ms, making caching unnecessary complexity.

### Symlink prefix `forja--`

All symlinks created by forja are prefixed with `forja--`. The full naming pattern is:

```
forja--{phase}--{tech}--{name}--{filename}.md
```

Slashes in the skill ID (`code/general/feature`) are replaced with `--`. This avoids collisions with user-created agents/commands, and enables `uninstall` and `doctor` to identify forja-managed symlinks by prefix without maintaining a separate manifest.

### State tracked in `~/.forja/state.json`

A single JSON file stores installed skill IDs, team configurations, and the active profile. The `save_installed_ids()` wrapper in `symlink/manager.rs` loads the full state before writing, ensuring that updating the installed list does not clobber team data.

### State migration (v1 to v2)

The original state format was a bare `Vec<String>` of skill IDs. When `load_state()` encounters this format, it transparently wraps it into a `ForjaState` with `version: 2` and empty teams. This migration happens in-memory on read; the v2 format is written back on the next state save.

### Local dev mode

`forja init` detects if the current working directory contains a `skills/` folder. If so, it creates a symlink from `~/.forja/registry` to the CWD instead of cloning from git. This enables development against the monorepo without duplicating the skill catalog.

### Teams env var management

Agent teams require `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` in `~/.claude/settings.json`. The `settings` module reads and writes this file, preserving existing settings. Team commands auto-enable this env var when needed.

### Commands slash command naming

Team slash commands are written to `~/.claude/commands/` with the pattern `forja--team--{name}.md`. These are separate from agent symlinks and contain structured markdown (frontmatter + orchestration instructions) generated by the `team.rs` module.

### Auto-install on init

`forja init` installs all skills automatically after setting up the registry. The `install_all_quiet()` function in `install.rs` reuses the same catalog scan and symlink logic as `install --all` but returns `(installed, skipped)` counts instead of printing per-skill output. This eliminates the manual browse-choose-install loop.

### Stack detection

`detect_stack()` in `init.rs` checks the cwd for framework files (`next.config.*`, `nuxt.config.*`, etc.) and language markers (`Cargo.toml`, `go.mod`, `tsconfig.json`, etc.). Two-layer approach: frameworks first, then languages, joined with " + ". Returns `None` when no recognized files exist.

### No-args contextual status

Running `forja` with no subcommand shows a status dashboard (initialized) or welcome pitch (not initialized). Implemented by making the clap `command` field `Option<Commands>`. `--help` is unaffected since clap handles it before our code.
