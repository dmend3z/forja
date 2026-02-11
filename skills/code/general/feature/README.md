# General Coder

> Phase: **Code** | Tech: **general**

General-purpose coding agent. Auto-detects project stack and writes code following existing conventions.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| coder | Read, Write, Edit, Bash, Glob, Grep, LSP | opus |

## What it does

The coder agent is a general-purpose implementation specialist that auto-detects your project stack and adapts to existing conventions. It reads CLAUDE.md, identifies the tech stack from config files, studies existing code patterns, and reuses utilities before creating new ones. It follows the principle of boring, obvious solutions over clever abstractions.

## Usage

After installing with `forja install code/general/feature`:

```bash
# Use the coder agent for general implementation
coder
```

The agent follows these principles:
- Auto-detect stack from config files
- Follow existing patterns (naming, structure, style)
- Prefer boring solutions over abstractions
- Small, focused changes (one concern per function)
- Reuse existing utilities

## Install

```bash
forja install code/general/feature
```
