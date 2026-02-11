# Architecture Planner

> Phase: **Research** | Tech: **architecture**

Create implementation plans with phases, file lists, and dependency maps. Analyzes codebase before proposing changes.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| planner | Read, Glob, Grep, LSP | opus |

## What it does

The planner agent analyzes your codebase and creates phased implementation plans. It reads CLAUDE.md, maps directory structure, traces data flow, identifies files to create and modify, and outputs a multi-phase plan with specific file paths, dependencies, risks, and open questions. Each phase is independently testable and small (3-7 files).

## Usage

After installing with `forja install research/architecture/planner`:

```bash
# Use the planner agent to create implementation plans
planner
```

The agent will produce a plan with:
- Context (what exists today and how the feature fits)
- Phases with files to create and modify
- Dependencies between phases
- Risks and mitigation strategies
- Open questions that need answers

## Install

```bash
forja install research/architecture/planner
```
