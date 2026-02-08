# forja

[![CI](https://github.com/dmend3z/forja/actions/workflows/ci.yml/badge.svg)](https://github.com/dmend3z/forja/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![npm](https://img.shields.io/npm/v/forja-cli)](https://www.npmjs.com/package/forja-cli)

**Skills marketplace for [Claude Code](https://claude.com/claude-code).** One command to install 28 curated skills for research, coding, testing, code review, and deployment.

> **forja** (Portuguese: *forge*) — forge your Claude Code setup in seconds.

<!-- TODO: Add demo GIF showing `forja init` → immediate usage -->

## Quick Start

### npm (recommended)

```bash
npm install -g forja-cli
forja init
```

### Shell installer

```bash
curl -fsSL https://raw.githubusercontent.com/dmend3z/forja/main/install.sh | sh
```

That's it. All 28 skills are installed and ready to use:

```bash
forja task "add user authentication with JWT"
```

## Why forja?

**Without forja** — manual setup, 10+ minutes:

```
Search for Claude Code plugins → Read docs → Download individually →
Configure each one → Figure out which phase needs what → Start coding
```

**With forja** — one command, 30 seconds:

```
forja init → Start coding
```

forja auto-installs skills for every phase of development. You get specialized agents for your stack, TDD workflows, security audits, and team configs — all wired up and ready.

## What You Get

| Phase | Skills | What it covers |
|-------|--------|----------------|
| **Research** | 4 | Codebase exploration, docs research, architecture planning |
| **Code** | 8 | TypeScript, Python, Go, Rust, Next.js, NestJS, database, general |
| **Test** | 4 | TDD workflow, test generation, E2E Playwright, coverage analysis |
| **Review** | 5 | Code quality, security audit, performance, PR workflow, simplification |
| **Deploy** | 3 | Git commits, PR creation, post-deploy verification |
| **Teams** | 4 | Multi-agent team configurations |

Run `forja list --available` to see all skills with descriptions.

## Commands

```
forja                              # Status dashboard (or welcome if not initialized)
forja init                         # Initialize + install all skills
forja task <task>                  # Run a task in Claude Code
forja task <task> --team <name>    # Run with a specific team
forja plan <task>                  # Create an implementation plan
forja execute                      # Execute the latest plan
forja list                         # Show installed skills
forja list --available             # Show all available skills
forja search <query>               # Search skills
forja info <skill-id>              # Show skill details
forja install <skill-id>           # Install a single skill
forja uninstall <skill-id>         # Remove a skill
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
├── registry/     # Cloned skills repo (or symlink to local)
├── config.json   # Settings
└── state.json    # Installed skill IDs

~/.claude/agents/
├── forja--research--codebase--explorer--researcher.md  → registry
├── forja--test--tdd--workflow--tdd-guide.md            → registry
└── ...
```

`forja init` clones the skills registry and symlinks all agents into `~/.claude/agents/`. Claude Code picks them up automatically — no restart needed. The `forja--` prefix prevents name collisions with other agents.

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
| [Skill Authoring](docs/SKILL-AUTHORING.md) | Create and publish skills |
| [Teams](docs/TEAMS.md) | Multi-agent team configs |
| [Contributing](docs/CONTRIBUTING.md) | Development setup |

## Requirements

- [Claude Code](https://claude.com/claude-code)
- `git` (for registry cloning)
- `gh` CLI (optional, for PR skills)

## License

MIT
