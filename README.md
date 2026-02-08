# forja

Skills marketplace for [Claude Code](https://claude.com/claude-code). Install curated skills, agents, and team configs organized around a 5-phase development workflow.

> **forja** (Portuguese: *forge*) — forge your Claude Code setup with the right skills for each phase of development.

## Why forja?

Claude Code supports plugins, agents, and skills — but managing them across projects is manual. forja gives you:

- **24 curated skills** organized into 5 workflow phases
- **Agent team configs** for multi-agent development (research → code → test → review → deploy)
- **One CLI** to install, uninstall, search, and manage everything
- **Symlink-based** — skills live in a central registry, installed via symlinks into `~/.claude/agents/`

## Quick Start

```bash
# Build from source
git clone https://github.com/forja-dev/forja-skills.git
cd forja-skills
cargo install --path .

# Initialize (detects local skills/ dir automatically)
forja init

# Browse what's available
forja phases
forja list --available

# Install a skill
forja install test/tdd/workflow

# Check health
forja doctor
```

## The 5-Phase Workflow

forja organizes skills around the workflow phases that matter:

| # | Phase | Skills | What it covers |
|---|-------|--------|----------------|
| 1 | **Research** | 4 | Codebase exploration, docs research, architecture planning, plan orchestration |
| 2 | **Code** | 8 | Language-specific agents (TypeScript, Python, Go, Rust, Next.js, NestJS, database, general) |
| 3 | **Test** | 4 | TDD workflow, test generation, E2E with Playwright, coverage analysis |
| 4 | **Review** | 5 | Code quality, security audit, performance, PR workflow, code simplification |
| 5 | **Deploy** | 3 | Git commits, PR creation, post-deploy verification |
| + | **Teams** | 4 | Multi-agent team configurations |

## Skill Catalog

### Research
/f
| Skill | ID | Description |
|-------|----|-------------|
| Codebase Explorer | `research/codebase/explorer` | Maps structure, traces patterns, outputs exploration report |
| Docs Researcher | `research/docs/researcher` | Research external docs and APIs via web search |
| Architecture Planner | `research/architecture/planner` | Implementation plans with phases, files, and dependency maps |

### Code

| Skill | ID | Description |
|-------|----|-------------|
| General | `code/general/feature` | Auto-detects stack, follows existing conventions |
| TypeScript | `code/typescript/feature` | Strict types, no `any`, proper patterns |
| Python | `code/python/feature` | FastAPI/Django, type hints, pydantic models |
| Go | `code/golang/feature` | Standard layout, interface-first, error wrapping |
| Rust | `code/rust/feature` | thiserror/anyhow, idiomatic ownership |
| Next.js | `code/nextjs/feature` | App Router, Server Components, Tailwind v4 |
| NestJS | `code/nestjs/feature` | Modules, services, controllers, Prisma |
| Database | `code/database/feature` | Schemas, migrations, queries (Prisma, Drizzle, SQL) |

### Test

| Skill | ID | Description |
|-------|----|-------------|
| TDD Workflow | `test/tdd/workflow` | Red-Green-Refactor cycle, tests first |
| Test Generator | `test/generate/suite` | Generate tests for existing code |
| E2E Playwright | `test/e2e/playwright` | Page Object Model, auto-waiting, traces |
| Coverage Analyzer | `test/coverage/analyzer` | Coverage gap analysis + targeted test generation |

### Review

| Skill | ID | Description |
|-------|----|-------------|
| Code Quality | `review/code-quality/reviewer` | Fresh-context review for quality and correctness |
| Security Auditor | `review/security/auditor` | OWASP Top 10, secrets, injection, auth |
| Performance | `review/performance/analyzer` | Complexity, N+1, re-renders, bundle size |
| PR Workflow | `review/pr-workflow/reviewer` | Full PR review lifecycle |

### Deploy

| Skill | ID | Description |
|-------|----|-------------|
| Git Commit | `deploy/git/commit` | Conventional commits with structured messages |
| Git PR | `deploy/git/pr` | Push + create PRs via `gh` CLI |
| Deploy Verify | `deploy/verify/checker` | CI status, health endpoints, smoke tests |

### Teams

| Config | ID | Agents |
|--------|----|--------|
| Full Product | `teams/full-product/team` | researcher + coder + tester + reviewer + deployer |
| Solo Sprint | `teams/solo-sprint/team` | coder-tester + quick-reviewer |
| Quick Fix | `teams/quick-fix/team` | coder + deployer |
| Refactor | `teams/refactor/team` | analyzer + refactorer + behavioral reviewer |

> Agent teams require `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` in your Claude Code settings.

## Commands

```
forja init                    # Initialize (~/.forja/, registry link)
forja install <phase/tech/skill>  # Install a skill via symlink
forja uninstall <skill-id>    # Remove an installed skill
forja list                    # Show installed skills
forja list --available        # Show all available skills
forja search <query>          # Search by name, description, phase, or tech
forja info <skill-id>         # Show detailed skill information
forja phases                  # Show the 5 workflow phases
forja update                  # Update the registry (git pull)
forja doctor                  # Verify installation health
```

## How It Works

```
~/.forja/
├── registry/     # Cloned skills repo (or symlink to local)
├── config.json   # Registry URL and settings
└── state.json    # Array of installed skill IDs

~/.claude/agents/
├── forja--research--codebase--explorer--researcher.md  # Symlink → registry
├── forja--test--tdd--workflow--tdd-guide.md            # Symlink → registry
└── ...
```

1. `forja init` clones the skills repo into `~/.forja/registry/` (or symlinks to a local `skills/` dir for development)
2. `forja install` creates symlinks from `~/.claude/agents/` to the skill's agent files
3. Claude Code picks up the agents automatically — no restart needed
4. `forja uninstall` removes the symlinks and updates state

The `forja--` prefix on symlinks prevents name collisions with other agents.

## Skill Structure

Each skill follows Claude Code's plugin format:

```
skills/<phase>/<tech>/<name>/
├── .claude-plugin/
│   └── plugin.json       # Name, description, version, author
├── agents/               # Agent .md files (symlinked on install)
│   └── agent-name.md     # Frontmatter: name, description, tools, model
├── skills/               # Slash command skills
│   └── skill-name/
│       └── SKILL.md
└── commands/             # Slash commands
    └── command-name.md
```

## Development

```bash
# Clone and build
git clone https://github.com/forja-dev/forja-skills.git
cd forja-skills
cargo build

# Init detects local skills/ dir and uses symlink
./target/debug/forja init

# Run tests
cargo test

# Release build
cargo build --release
```

## Requirements

- Rust 1.85+ (edition 2024)
- [Claude Code](https://claude.com/claude-code)
- `git` (for registry cloning and updates)
- `gh` CLI (optional, for PR-related skills)

## License

MIT
