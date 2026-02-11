# Explore Codebase

> Phase: **Research** | Tech: **codebase**

Deep codebase exploration before writing code. Maps structure, traces patterns, identifies conventions, outputs structured report.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| researcher | Read, Glob, Grep, LSP, Bash | opus |

## What it does

The researcher agent performs read-only codebase exploration to build a complete mental model before any changes. It reads CLAUDE.md, maps directory structure, detects the stack from config files, traces recurring patterns using Grep, and checks recent git history. The output is a structured report covering stack, architecture, conventions, key files, risks, and recommended approaches.

## Usage

After installing with `forja install research/codebase/explorer`:

```bash
# Use the researcher agent to explore your codebase
researcher
```

The agent will produce a report with:
- Stack detection (framework, language, database, hosting)
- Architecture patterns and directory structure
- Conventions (naming, exports, error handling, testing)
- Key files and their purposes
- Risks and potential issues
- Recommended approach for upcoming work

## Install

```bash
forja install research/codebase/explorer
```
