# Roadmap

**forja** is an agent manager for Claude Code. This roadmap lists planned features, improvements, and ideas — organized by priority and labeled by difficulty so contributors can find work that matches their interest and experience.

> Want to help? See [How to Contribute](#how-to-contribute) below, or jump straight to items marked `good-first-issue`.

---

## How to Contribute

1. **Pick an item** from the roadmap below. Items marked `good-first-issue` are the best starting point.
2. **Read the guides** — [Contributing](docs/CONTRIBUTING.md) covers dev setup, code style, and commit conventions. [Skill Authoring](docs/SKILL-AUTHORING.md) covers agent creation.
3. **Open an issue** to claim the item before starting work. This prevents duplicate effort.
4. **Submit a PR** following the acceptance criteria listed for each item.

Not sure where to start? Open an issue with your question — all questions are welcome.

### Labels

| Label | Meaning |
|-------|---------|
| `good-first-issue` | Small scope, clear requirements, minimal context needed |
| `medium` | Requires understanding of one or two modules |
| `hard` | Touches multiple modules, involves design decisions |

### Categories

| Category | Scope |
|----------|-------|
| `[CLI UX]` | Terminal output, commands, flags, user interaction |
| `[Skill Management]` | Install, uninstall, update, catalog, registry |
| `[Ecosystem]` | Cross-project features, integrations, extensibility |
| `[Developer Experience]` | Tooling for skill authors and CLI contributors |

---

## NOW — v0.2.x

Polish, stability, and contributor onboarding. These items improve what already exists.

### Improve error messages for common failures `[CLI UX]` `good-first-issue`

Several error paths return raw `io::Error` or generic messages. Surface actionable guidance when users hit common problems (missing `git`, uninitialized state, broken symlinks).

**Acceptance criteria:**
- `forja install` on uninitialized state suggests `forja init`
- Missing `git` binary shows install instructions per platform
- Broken symlink detected by `forja doctor` shows the affected skill and a fix command
- Error messages use consistent formatting (colored prefix + plain description)

**Tests:** Unit tests for each error variant's display message. Integration test for `forja install` without prior init.

---

### Add `--json` output flag to `forja list` and `forja search` `[CLI UX]` `good-first-issue`

Enable machine-readable output for scripting and tooling integration.

**Acceptance criteria:**
- `forja list --json` outputs a JSON array of installed skill objects
- `forja search <query> --json` outputs a JSON array of matching skill objects
- JSON schema includes: `id`, `name`, `description`, `phase`, `tech`, `installed`
- Human-readable output remains the default (no flag)

**Tests:** Unit tests asserting JSON structure matches schema. Integration test comparing `--json` output against known catalog state.

---

### Add `forja validate` command for skill authors `[Developer Experience]` `good-first-issue`

Validate a skill directory's structure before publishing — check for required files, valid `plugin.json`, and content type consistency.

**Acceptance criteria:**
- `forja validate <path>` checks a skill directory
- Validates: `.claude-plugin/plugin.json` exists and parses, at least one content type dir exists (`agents/`, `skills/`, `commands/`), no empty content dirs
- Reports all issues (not just the first one)
- Exit code 0 on success, 1 on failure

**Tests:** Unit tests with valid and invalid skill directory fixtures (using `tempfile`). Tests for each validation rule independently.

---

### Add `forja uninstall --all` flag `[Skill Management]` `good-first-issue`

Allow removing all forja-managed agents in one command, useful for clean reinstalls.

**Acceptance criteria:**
- `forja uninstall --all` removes all symlinks with `forja--` prefix
- Updates state to empty installed list
- Requires confirmation prompt before proceeding (skip with `--yes`)
- Prints count of removed agents

**Tests:** Integration test that installs agents, runs `uninstall --all`, and verifies state and filesystem are clean.

---

### Dry run mode for install and uninstall `[CLI UX]` `good-first-issue`

Preview what would change without modifying the filesystem.

**Acceptance criteria:**
- `forja install <skill> --dry-run` shows symlinks that would be created
- `forja uninstall <skill> --dry-run` shows symlinks that would be removed
- `forja install --all --dry-run` shows full installation plan
- No filesystem changes occur during dry run
- Output clearly labeled as dry run

**Tests:** Integration tests verifying filesystem is unchanged after dry run. Output content assertions.

---

### Add shell completions generation `[CLI UX]` `medium`

Generate shell completions for bash, zsh, fish, and PowerShell using clap's built-in completion generator.

**Acceptance criteria:**
- `forja completions <shell>` outputs completions to stdout
- Supported shells: `bash`, `zsh`, `fish`, `powershell`
- Completions cover all subcommands and flags
- README documents how to install completions

**Tests:** Integration test that runs `forja completions bash` and verifies non-empty output containing known subcommand names.

---

### Expand test coverage for existing commands `[Developer Experience]` `medium`

Several commands (`search`, `info`, `phases`, `status`) lack unit tests. Add coverage for the core logic paths.

**Acceptance criteria:**
- `search` module has tests for: exact match, partial match, no results, case sensitivity
- `info` module has tests for: valid skill, missing skill
- `status` module has tests for: initialized state, uninitialized state
- All tests use `tempfile` for filesystem isolation
- Coverage of `src/commands/` reaches 80%+

**Tests:** `#[cfg(test)]` modules in each command file, following existing patterns in `install.rs`.

---

## NEXT — v0.3.x

New capabilities that extend forja's core value.

### Skill versioning and update detection `[Skill Management]` `hard`

Track installed skill versions and detect when the registry has newer versions available.

**Acceptance criteria:**
- `forja list` shows version column for installed skills (from `plugin.json`)
- `forja update` compares installed versions against registry and reports available updates
- `forja update --apply` reinstalls skills with newer versions
- Version comparison uses semver (major.minor.patch)
- State file tracks installed version per skill

**Tests:** Unit tests for semver comparison. Integration tests for update detection with mock registry containing version bumps. State migration test from versionless to versioned state.

---

### Skill scaffolding with `forja new` `[Developer Experience]` `medium`

Generate a new skill directory with the correct structure, ready for authoring.

**Acceptance criteria:**
- `forja new <phase> <tech> <name>` creates `skills/<phase>/<tech>/<name>/` with boilerplate
- Generated files: `.claude-plugin/plugin.json` (with prompted name/description), `agents/<name>.md` (with template)
- Interactive prompts for name, description, and content types to include
- `--non-interactive` flag with `--name` and `--description` options for CI use
- Generated skill passes `forja validate`

**Tests:** Integration test that runs `forja new` with `--non-interactive`, then runs `forja validate` on the output. Template content assertions.

---

### Plan listing and viewing `[CLI UX]` `medium`

View previously created plans and their details.

**Acceptance criteria:**
- `forja plans` lists all plans with id, task summary, status, and creation date
- `forja plans show <id>` displays the full plan markdown
- Plans sorted by date, most recent first
- Status column shows: pending, executed, archived

**Tests:** Unit tests for plan listing and sorting. Integration tests with fixture plan files in a temp `plans/` directory.

---

### Plan archival and deletion `[CLI UX]` `medium`

Manage plan lifecycle — archive completed plans and delete old ones.

**Acceptance criteria:**
- `forja plans archive <id>` marks a plan as archived (updates JSON status)
- `forja plans delete <id>` removes plan files (with confirmation prompt, skip with `--yes`)
- Archived plans still visible in `forja plans` with status indicator
- Deleted plans removed from filesystem

**Tests:** Integration tests for status transitions (pending to archived). Filesystem cleanup verification for delete. Confirmation prompt skip with `--yes` flag.

---

### Selective skill installation by phase or tech `[Skill Management]` `medium`

Install subsets of the catalog instead of all-or-nothing.

**Acceptance criteria:**
- `forja install --phase code` installs all skills in the Code phase
- `forja install --tech rust` installs all Rust-related skills across phases
- Flags can be combined: `--phase code --tech rust`
- Already-installed skills are skipped silently
- Prints summary: installed count, skipped count

**Tests:** Integration tests with a mock catalog covering multiple phases and techs. Tests for filter combinations and skip behavior.

---

### Skill dependency declaration `[Ecosystem]` `hard`

Allow skills to declare dependencies on other skills, with automatic co-installation.

**Acceptance criteria:**
- `plugin.json` supports optional `dependencies` field: array of skill IDs
- `forja install <skill>` resolves and installs dependencies first
- Circular dependency detection with clear error message
- `forja info <skill>` shows dependency tree
- `forja uninstall <skill>` warns if other skills depend on it

**Tests:** Unit tests for dependency resolution and cycle detection. Integration tests with multi-skill dependency chains. Edge case: missing dependency in catalog.

---

## LATER — v1.0+

Ambitious features for ecosystem maturity. These require design discussion before implementation.

### Multi-registry support `[Ecosystem]` `hard`

Allow users to add multiple skill registries (e.g., company-internal, community).

**Acceptance criteria:**
- `forja registry add <name> <url>` adds a registry source
- `forja registry list` shows configured registries
- `forja registry remove <name>` removes a registry
- Catalog scan merges skills from all registries
- Name collisions across registries show clear error with resolution guidance
- Default registry (`dmend3z/forja`) is always present

**Tests:** Integration tests with multiple mock registries. Collision detection tests. Config persistence tests.

---

### Skill tags and filtering `[Skill Management]` `medium`

Rich filtering beyond phase and tech, using tags from `plugin.json`.

**Acceptance criteria:**
- `plugin.json` `keywords` field used as tags for filtering
- `forja search --tag <tag>` filters by tag
- `forja list --tag <tag>` filters installed skills by tag
- Tags displayed in `forja info` output
- Common tags documented in Skill Authoring guide

**Tests:** Unit tests for tag-based filtering. Integration tests with tagged mock skills.

---

### Agent capability discovery `[Ecosystem]` `hard`

Analyze a project's codebase and recommend which agents to install based on detected tech stack, existing patterns, and project size.

**Acceptance criteria:**
- `forja recommend` analyzes cwd and suggests agents
- Detection covers: languages, frameworks, test runners, CI config, package managers
- Output grouped by phase with relevance explanation
- `forja recommend --install` installs recommended agents directly
- Already-installed agents shown as "installed" in output

**Tests:** Integration tests with mock project directories containing various tech stack indicators. Recommendation logic unit tests.

---

### Skill usage analytics (local only) `[Developer Experience]` `medium`

Track which agents are used most frequently to help prioritize catalog improvements. All data stays local.

**Acceptance criteria:**
- `forja stats` shows usage counts per installed skill
- Data stored in `~/.forja/analytics.json` — never transmitted
- Tracks: install count, last-used timestamp (when referenced in `forja task` or `forja execute`)
- `forja stats --reset` clears analytics data
- Privacy section in README explains local-only nature

**Tests:** Unit tests for analytics read/write. Integration tests for event tracking during task execution.

---

### Exportable team configurations `[Ecosystem]` `medium`

Share team setups across projects and with other users.

**Acceptance criteria:**
- `forja team export <name>` writes team config to a portable JSON file
- `forja team import <file>` creates a team from an exported config
- Exported format includes: team name, members (skill IDs + roles), profile
- Import auto-installs missing skills referenced by the team
- Validates skill availability before import

**Tests:** Round-trip test: export then import produces identical team config. Import with missing skills triggers auto-install. Invalid file handling.

---

## Proposing New Items

Have an idea that's not on this list?

1. **Open an issue** with the title `[Roadmap] <your idea>`.
2. Include: description, motivation (what problem it solves), and rough difficulty estimate.
3. The maintainer will triage it into a priority tier.

Browse [open issues](https://github.com/dmend3z/forja/issues) to see what's already proposed.
