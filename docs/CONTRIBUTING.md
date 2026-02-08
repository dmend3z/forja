# Contributing to forja

This guide covers the essentials for contributing to the forja project -- both the Rust CLI and the agent catalog.

## Development Setup

```bash
# Clone the monorepo
git clone https://github.com/dmend3z/forja.git
cd forja-skills

# Build the CLI
cargo build

# Initialize in local dev mode (detects skills/ dir and symlinks instead of cloning)
cargo run -- init
```

When you run `forja init` from inside the repo, it detects the local `skills/` directory, creates a symlink at `~/.forja/registry` pointing to your working copy, and auto-installs all agents. Changes to agent files are reflected immediately without re-installing.

### Prerequisites

- Rust (edition 2024)
- Git
- Claude Code (for testing agent behavior end-to-end)

## Project Structure

```
src/                  # CLI source code
  cli.rs              # Clap argument definitions
  main.rs             # Entry point and command dispatch
  error.rs            # ForjaError enum (thiserror)
  paths.rs            # ForjaPaths -- all filesystem paths
  settings.rs         # User settings
  commands/           # One file per CLI command
    status.rs           # No-args status display
  models/             # Data types: Skill, Phase, Plugin, State, Plan, Profile
  registry/           # Catalog scanner and git operations
  symlink/            # Symlink creation and management

skills/               # Agent catalog
  <phase>/<tech>/<name>/
    .claude-plugin/plugin.json
    agents/*.md
    skills/*/SKILL.md
    commands/*.md
```

For a detailed breakdown, see [docs/ARCHITECTURE.md](./ARCHITECTURE.md).

## How to Add a New Agent

1. Pick the phase (`research`, `code`, `test`, `review`, `deploy`, `teams`) and a tech category.
2. Create the directory: `skills/<phase>/<tech>/<name>/`
3. Add `.claude-plugin/plugin.json` with name, description, version, author, and keywords.
4. Add at least one content type: `agents/*.md`, `skills/*/SKILL.md`, or `commands/*.md`.
5. Test with `cargo run -- list --available` to verify the catalog scanner picks it up.
6. Install it with `cargo run -- install <phase>/<tech>/<name>` and test in Claude Code.

The agent ID format is `<phase>/<tech>/<name>` (e.g., `code/rust/feature`).

For the full authoring guide, see [docs/SKILL-AUTHORING.md](./SKILL-AUTHORING.md).

## How to Modify the CLI

Each CLI command lives in its own file under `src/commands/`. The pattern:

1. Add the command variant to `Commands` enum in `src/cli.rs`.
2. Create `src/commands/<name>.rs` with a `pub fn run(...) -> Result<()>`.
3. Register the module in `src/commands/mod.rs`.
4. Wire it in the `match` block in `src/main.rs`.

### Key types

- **`ForjaError`** (`src/error.rs`) -- all errors go through this enum. Use `thiserror` derives, not manual `impl`.
- **`ForjaPaths`** (`src/paths.rs`) -- canonical source for all filesystem paths (`~/.forja/`, `~/.claude/agents/`, etc.). Never hardcode paths.
- **`Result<T>`** -- alias for `std::result::Result<T, ForjaError>`, defined in `src/error.rs`.

### Running tests

```bash
cargo test
```

Tests use `tempfile` for isolated filesystem testing. Existing tests live alongside source files using `#[cfg(test)]` modules.

## Code Style

- **snake_case** for functions, variables, modules. **PascalCase** for types and enums.
- **No async** -- all operations are synchronous filesystem and git subprocess calls.
- **`ForjaPaths`** for all path construction -- no ad-hoc `home_dir().join(...)`.
- **`ForjaError`** for all errors -- add new variants as needed, use `#[from]` for automatic conversion.
- **`colored`** for terminal output -- use `.bold()`, `.green()`, `.yellow()`, `.cyan()` consistently.
- Derive `Debug`, `Clone` on data types. Add `PartialEq`, `Eq`, `Hash` when needed.
- Prefer boring, obvious solutions. No premature abstractions.

## Commit Conventions

Use [Conventional Commits](https://www.conventionalcommits.org/) in English:

```
type(scope): subject

# Examples
feat(cli): add doctor command for health checks
fix(symlink): handle missing agents directory
feat(skills): add nestjs coding agent
docs: update contributing guide
refactor(registry): simplify catalog scanning
test(models): add state serialization tests
```

**Types**: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

**Scopes**: `cli`, `skills`, `registry`, `symlink`, `models`, or a specific skill name.
