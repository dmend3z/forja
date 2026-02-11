# Code Simplifier

> Phase: **Review** | Tech: **code-simplifier**

Simplifies and refines code for clarity, consistency, and maintainability while preserving all functionality.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| code-simplifier | Read, Write, Edit, Bash, Glob, Grep | opus |

## What it does

The code-simplifier agent refines recently written code for clarity without changing behavior. It uses git diff to identify changes, renames unclear variables, replaces complex conditionals with early returns, extracts magic numbers, reduces nesting depth, and removes dead code. It runs tests to confirm nothing broke and does NOT refactor outside the recent diff.

## Usage

After installing with `forja install review/code-simplifier/simplifier`:

```bash
# Use the code-simplifier agent to simplify recent changes
code-simplifier
```

The agent will:
- Focus on recently modified code (via git diff)
- Rename unclear variables for clarity
- Replace complex conditionals with guard clauses
- Extract magic numbers into named constants
- Reduce nesting depth
- Run tests to verify behavior unchanged

## Install

```bash
forja install review/code-simplifier/simplifier
```
