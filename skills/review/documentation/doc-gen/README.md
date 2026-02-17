# Documentation Generator

> Phase: **Review** | Tech: **documentation**

Generates and updates CLAUDE.md, AGENTS.md, and README.md by analyzing the codebase. Surgical updates preserve custom content.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| doc-gen | Read, Write, Edit, Bash, Glob, Grep | sonnet |

## What it does

The doc-gen agent reads your codebase (config files, directory structure, installed agents, dev scripts) and generates accurate project documentation. It follows a strict read-first workflow: gather facts, write minimal output, then self-review every claim.

- **CLAUDE.md** — stack, commands, architecture, conventions, and Karpathy-style coding rules
- **AGENTS.md** — table of installed agents with description, tools, and model (only if agents exist)
- **README.md** — project overview, requirements, setup, usage, and development commands

When updating existing docs, it only rewrites stale sections and preserves custom content you've added.

## Slash Command

| Command | Description |
|---------|-------------|
| `/doc-gen` | Generate all docs (CLAUDE.md, AGENTS.md, README.md) |
| `/doc-gen claude-md` | Generate only CLAUDE.md |
| `/doc-gen agents-md` | Generate only AGENTS.md |
| `/doc-gen readme` | Generate only README.md |

## Usage

After installing with `forja install review/documentation/doc-gen`:

```bash
# Generate all documentation
/doc-gen

# Generate only CLAUDE.md
/doc-gen claude-md

# Use the agent in team orchestration
# (via Task tool with subagent_type: "doc-gen")
```

The agent will:
- Read config files to detect project stack
- Map directory structure and entry points
- Scan installed agents in `.claude/agents/`
- Extract dev commands from package.json/Makefile/Cargo.toml
- Generate or surgically update documentation
- Self-review all output against actual codebase state

## Install

```bash
forja install review/documentation/doc-gen
```
