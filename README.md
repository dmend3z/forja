# forja

[![CI](https://github.com/dmend3z/forja/actions/workflows/ci.yml/badge.svg)](https://github.com/dmend3z/forja/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![npm](https://img.shields.io/npm/v/forja-cli)](https://www.npmjs.com/package/forja-cli)

**Agent manager for [Claude Code](https://claude.com/claude-code).** Stop configuring, start shipping. 25 curated agents across 5 dev phases — Research, Code, Test, Review, Deploy.

> **forja** (Portuguese: *forge*) — forge your Claude Code setup in seconds.

<!-- TODO: Add demo GIF showing `forja init` → immediate usage -->

## Quick Start

### Homebrew (macOS/Linux)

```bash
brew install dmend3z/forja/forja
forja init
```

### npm

```bash
npm install -g forja-cli
forja init
```

### Shell installer

```bash
curl -fsSL https://raw.githubusercontent.com/dmend3z/forja/main/install.sh | sh
```

That's it. All 25 agents are installed and ready to use:

```bash
forja task "add user authentication with JWT"
```

### With Teams (optional)

Run complex tasks with coordinated multi-agent teams:

```bash
forja task "build user dashboard" --team full-product
```

Teams auto-configure on first use. Presets: `full-product`, `solo-sprint`, `quick-fix`, `refactor`.

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

forja auto-installs agents for every phase of development. You get specialized agents for your stack, TDD workflows, security audits, and team configs — all wired up and ready.

## What You Get

| Phase | Agents | What it covers |
|-------|--------|----------------|
| **Research** | 4 | Codebase exploration, docs research, architecture planning |
| **Code** | 8 | TypeScript, Python, Go, Rust, Next.js, NestJS, database, general |
| **Test** | 4 | TDD workflow, test generation, E2E Playwright, coverage analysis |
| **Review** | 5 | Code quality, security audit, performance, PR workflow, simplification |
| **Deploy** | 3 | Git commits, PR creation, post-deploy verification |
| **+Teams** | 4 configs | Multi-agent team configurations |

Run `forja list --available` to see all agents with descriptions.

## Commands

```
forja                              # Status dashboard (or welcome if not initialized)
forja init                         # Initialize + install all agents
forja task <task>                  # Run a task in Claude Code
forja task <task> --team <name>    # Run with a specific team
forja plan <task>                  # Create an implementation plan
forja execute                      # Execute the latest plan
forja list                         # Show installed agents
forja list --available             # Show all available agents
forja search <query>               # Search agents
forja info <skill-id>              # Show agent details
forja install <skill-id>           # Install a single agent
forja uninstall <skill-id>         # Remove an agent
forja update                       # Update the registry
forja doctor                       # Verify installation health
forja phases                       # Show the 5 workflow phases
```

## Agent Teams

Run complex tasks with coordinated multi-agent teams:

| Team | Agents | Use case |
|------|--------|----------|
| **full-product** | researcher + coder + tester + reviewer + deployer | Full features |
| **solo-sprint** | coder-tester + quick-reviewer | Medium features |
| **quick-fix** | coder + deployer | Hotfixes |
| **refactor** | analyzer + refactorer + behavioral reviewer | Structural changes |

```bash
forja team preset full-product
forja task "build user dashboard" --team full-product
```

> Requires `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` in Claude Code settings.

## How It Works

```
~/.forja/
├── registry/     # Cloned agent registry (or symlink to local)
├── config.json   # Settings
└── state.json    # Installed agent IDs

~/.claude/agents/
├── forja--research--codebase--explorer--researcher.md  → registry
├── forja--test--tdd--workflow--tdd-guide.md            → registry
└── ...
```

`forja init` clones the agent registry and symlinks all agents into `~/.claude/agents/`. Claude Code picks them up automatically — no restart needed. The `forja--` prefix prevents name collisions with other agents.

## Profiles

Control model assignments per phase:

| Profile | Thinking (Research, Review) | Execution (Code, Test, Deploy) |
|---------|----------------------------|-------------------------------|
| **fast** | sonnet | sonnet |
| **balanced** (default) | opus | sonnet |
| **max** | opus | opus |

## Documentation

| Guide | Description |
|-------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | System design and data flow |
| [Agent Authoring](docs/SKILL-AUTHORING.md) | Create and publish agents |
| [Teams](docs/TEAMS.md) | Multi-agent team configs |
| [Contributing](docs/CONTRIBUTING.md) | Development setup |

## Requirements

- [Claude Code](https://claude.com/claude-code)
- `git` (for registry cloning)
- `gh` CLI (optional, for PR agents)

## License

MIT
