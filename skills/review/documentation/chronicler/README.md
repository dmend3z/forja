# Decision Chronicler

> Phase: **Review** | Tech: **documentation**

Documents all decisions and their rationale during the team workflow. Writes a structured decision log to docs/decisions/.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| chronicler | Read, Write, Glob | haiku |

## What it does

The chronicler agent is a passive observer that documents decisions made during development workflows. It extracts decisions from git diffs, agent output files, or team lead summaries, and writes them to structured decision log files in `docs/decisions/`.

It identifies approach choices, trade-offs, scope boundaries, risks accepted, architecture decisions, and tool selections — ignoring routine implementation details.

## Usage

After installing with `forja install review/documentation/chronicler`:

```bash
# Document decisions from recent changes
forja chronicle

# Document decisions from specific files
forja chronicle --from src/auth.rs
```

The agent will:
- Read recent git changes or specified files
- Extract decisions (approach choices, trade-offs, rejections)
- Write a structured decision log to `docs/decisions/YYYY-MM-DD-{slug}.md`

## Auto-installed

The chronicler is auto-installed during `forja init` and runs as part of `forja review` and `forja ship` workflows (skip with `--no-chronicle`).

## Install

```bash
forja install review/documentation/chronicler
```
