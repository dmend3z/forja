<p align="center">
  <img src="images/banner.png" alt="forja — forge your Claude Code setup" width="600">
</p>

<p align="center">
  <a href="https://github.com/dmend3z/forja/actions/workflows/ci.yml"><img src="https://github.com/dmend3z/forja/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
  <a href="https://www.npmjs.com/package/forja-cli"><img src="https://img.shields.io/npm/v/forja-cli" alt="npm"></a>
</p>

<h3 align="center">Forge once. Reuse forever.</h3>

<p align="center">
  Agent manager for <a href="https://claude.com/claude-code">Claude Code</a>. Stop configuring, start shipping.
</p>

---

Anthropic shipped [agent teams](https://code.claude.com/docs/en/agent-teams) in Claude Code — multiple AI sessions working in parallel, coordinated through a shared task list. Powerful, but a lot of setup: feature flags, per-agent prompts, team structures, workflow orchestration.

**forja gives you all of that in one command.** 24 curated agents across 5 dev phases — Research, Code, Test, Review, Deploy — plus 7 ready-made team configs, all wired up and ready to go.

> **forja** (Portuguese: *forge*)

## Quick Start

### Install

**Homebrew (macOS/Linux)**

```bash
brew install dmend3z/forja/forja
```

**npm**

```bash
npm install -g forja-cli
```

**Shell installer**

```bash
curl -fsSL https://raw.githubusercontent.com/dmend3z/forja/main/install.sh | sh
```

### Initialize

```bash
forja init
```

That's it — all 31 skills are installed and ready. Run `forja` with no arguments to see your status dashboard:

```
  forja

  Mode:    global
  Skills:  31/31 installed
  Health:  all symlinks OK
```

Want more control? Use `forja init --wizard` to pick your mode (project vs global), phases, and model profile interactively.

### Start building

```bash
# Plan a feature — forja interviews you, researches your codebase, builds a plan
forja plan "add user authentication with JWT"

# Execute it — agents work phase by phase: research → code → test → review → deploy
forja execute

# Or skip planning for quick tasks
forja task "fix the typo in the login page"

# Bring a whole team for bigger tasks
forja task "build user dashboard" --team full-product
```

## Two Ways to Work

### `forja plan` + `forja execute` (recommended)

Structured, multi-phase execution. When you run `forja plan`, it:

1. **Interviews you** — project type, goals, exclusions
2. **Researches your codebase** — detects stack, maps architecture
3. **Selects agents** — picks agents based on your stack (Next.js, Rust, Python, etc.)
4. **Sizes the team** — `quick-fix`, `solo-sprint`, or `full-product` based on complexity
5. **Builds phased steps** — files to create/modify, dependencies, instructions
6. **Saves the plan** — JSON + Markdown in `~/.forja/plans/`

Then `forja execute` runs the plan — agents work in order, passing context between phases.

```bash
forja plan "refactor auth to use OAuth2"
# answer a few questions, review the plan

forja execute
# agents handle research → code → test → review → deploy

forja execute --resume
# pick up where you left off if execution was interrupted
```

### `forja task` (quick mode)

For simple fixes where planning is overkill:

```bash
forja task "fix the typo in the login page"
```

Add `--team` for team execution without a plan:

```bash
forja task "build user dashboard" --team full-product
```

Use `--print` for non-interactive output (piping, CI):

```bash
forja task "explain the auth flow" --print
```

## Why forja?

**Without forja** — manual setup, 10+ minutes:

```
Search for Claude Code agents → Read docs → Download individually →
Configure each one → Figure out which phase needs what → Start coding
```

**With forja** — one command, 30 seconds:

```
forja init → Start coding
```

Specialized agents for your stack, TDD workflows, security audits, and team configs — all wired up and ready.

## What You Get

| Phase | Skills | What it covers |
|-------|--------|----------------|
| **Research** | 4 | Codebase exploration, docs research, architecture planning, implementation plans |
| **Code** | 8 | TypeScript, Python, Go, Rust, Next.js, NestJS, database, general |
| **Test** | 4 | TDD workflow, test generation, E2E Playwright, coverage analysis |
| **Review** | 5 | Code quality, security audit, performance, PR workflow, code simplification |
| **Deploy** | 3 | Git commits, PR creation, post-deploy verification |
| **Teams** | 7 | Multi-agent team configs (see [Agent Teams](#agent-teams)) |
| | **31 total** | |

Run `forja list --available` to browse all skills by phase, or `forja search rust` to find skills for your stack.

## Project Mode vs Global Mode

forja can run in two modes:

| Mode | Config location | Best for |
|------|----------------|----------|
| **Project** | `.forja/` in your repo | Per-project skill sets, team-specific configs |
| **Global** | `~/.forja/` in your home dir | Shared setup across all projects |

- `forja init` installs globally by default.
- `forja init --wizard` lets you choose project mode, which creates `.forja/` in your current directory.
- forja auto-detects: if it finds `.forja/config.json` in the current directory (or any parent), it uses project mode. Otherwise, it falls back to global.

Both modes install agents into `~/.claude/agents/` — Claude Code always reads from there.

## Commands

```
forja                              # Status dashboard (or welcome if not initialized)
forja init                         # Initialize + install all skills
forja init --wizard                # Interactive setup (choose mode, phases, profile)
forja plan <task>                  # Create an implementation plan (recommended)
forja execute                      # Execute the latest plan
forja execute <plan-id>            # Execute a specific plan
forja execute --resume             # Resume from last checkpoint
forja task <task>                  # Run a task directly (quick mode)
forja task <task> --team <name>    # Run with a specific team
forja task <task> --print          # Non-interactive output
forja list                         # Show installed skills
forja list --available             # Show all available skills by phase
forja search <query>               # Search skills by name, description, phase, or tech
forja info <skill-id>              # Show skill details
forja install <skill-id>           # Install a single skill
forja install --all                # Install every available skill
forja uninstall <skill-id>         # Remove a skill
forja update                       # Update the registry (git pull)
forja doctor                       # Verify installation health
forja guide                        # Getting started guide (all phases)
forja guide --phase <name>         # Guide for a specific phase (research, code, test, review, deploy)
forja monitor                      # Real-time web dashboard for agent teams
forja monitor --port <port>        # Use custom port (default: 3030)
forja team preset <name>           # Create team from preset
forja team create <name>           # Create custom team (interactive wizard)
forja team list                    # List configured teams
forja team info <name>             # Show team details and model assignments
forja team delete <name>           # Delete a team
```

## Agent Teams

Run complex tasks with coordinated multi-agent teams:

| Team | Agents | Use case |
|------|--------|----------|
| **full-product** | 6: researcher, coder, tester, code-simplifier, reviewer, deployer | Full features, end-to-end |
| **solo-sprint** | 3: coder-tester, code-simplifier, reviewer | Medium features |
| **quick-fix** | 2: coder, deployer | Hotfixes and small patches |
| **refactor** | 3: analyzer, refactorer, behavioral reviewer | Structural changes that preserve behavior |
| **dispatch** | 1: dispatcher | Fan-out independent tasks to parallel agents |
| **tech-council** | 1: facilitator (summons 5 engineering personas) | Architecture and technical decisions |
| **biz-council** | 1: facilitator (summons 5 business personas) | Product and strategy decisions |

### Using presets

Six teams have CLI preset shortcuts:

```bash
forja team preset full-product          # Create from preset
forja task "build user dashboard" --team full-product

forja team preset quick-fix
forja task "fix the login redirect" --team quick-fix

forja team preset tech-council
forja task "should we migrate to microservices?" --team tech-council
```

Available presets: `full-product`, `solo-sprint`, `quick-fix`, `dispatch`, `tech-council`, `biz-council`.

The **refactor** team is available as a slash command (`/refactor`) after installing the `teams/refactor/team` skill — it doesn't need a preset since it includes its own orchestration prompt.

### Custom teams

Build your own team by picking from installed skills:

```bash
forja team create my-team      # Interactive wizard: select agents + profile
forja team list                # See all configured teams
forja team info my-team        # View members and model assignments
forja team delete my-team      # Remove a team
```

### Monitoring teams

Watch your agents work in real-time:

```bash
forja monitor                  # Opens a web dashboard at localhost:3030
forja monitor --port 8080      # Use a custom port
forja monitor --no-open        # Start the server without auto-opening the browser
```

The monitor streams team configs, task progress, and inter-agent messages live via SSE.

> Agent teams require `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` in Claude Code settings. forja prompts you to enable it automatically on first use.

## How It Works

```
~/.forja/
├── registry/     # Cloned agent registry (or symlink to local)
├── config.json   # Settings
└── state.json    # Installed skill IDs + team configs

~/.claude/agents/
├── forja--research--codebase--explorer--researcher.md  -> registry
├── forja--test--tdd--workflow--tdd-guide.md            -> registry
└── ...
```

`forja init` clones the agent registry and symlinks agents into `~/.claude/agents/`. Claude Code picks them up automatically — no restart needed. The `forja--` prefix prevents name collisions with other agents.

In local development, if forja detects a `skills/` directory in the current repo, it creates a symlink to it instead of cloning — changes to skill files take effect immediately.

## Profiles

Control model assignments per phase:

| Profile | Thinking (Research, Review) | Execution (Code, Test, Deploy) |
|---------|----------------------------|-------------------------------|
| **fast** | sonnet | sonnet |
| **balanced** (default) | opus | sonnet |
| **max** | opus | opus |

Set during `forja init --wizard`, or override per-command:

```bash
forja execute --profile fast
forja team preset solo-sprint --profile max
```

## Documentation

| Guide | Description |
|-------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | System design and data flow |
| [Skill Authoring](docs/SKILL-AUTHORING.md) | Create and publish skills |
| [Teams](docs/TEAMS.md) | Multi-agent team configs |
| [Contributing](docs/CONTRIBUTING.md) | Development setup |

## Requirements

- [Claude Code](https://claude.com/claude-code)
- `git` (for registry cloning)
- `gh` CLI (optional, for PR agents)

## License

MIT
