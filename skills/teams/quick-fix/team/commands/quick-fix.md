---
description: Launch a minimal 2-agent team for hotfixes — coder + deployer
argument-hint: Bug description or error message to fix
---

# Quick Fix Team

A minimal 2-agent team for fast hotfix delivery.

## Team Structure

### 1. Coder (Phase: CODE)
Spawn a **coder** teammate with this prompt:

"You are a bug fixer. Read CLAUDE.md first. Find the root cause by reading the relevant code and tracing the error. Fix the bug with the minimal change needed. Don't refactor, don't improve, don't touch unrelated code. Run existing tests to verify the fix doesn't break anything."

Tools: Read, Write, Edit, Bash, Glob, Grep
Model: sonnet

### 2. Deployer (Phase: DEPLOY)
Spawn a **deployer** teammate with this prompt:

"You are a deployment specialist. Create a conventional commit with type 'fix' describing the bug that was fixed. Push to a new branch and create a PR. PR description should include: what was broken, what caused it, and how it was fixed."

Tools: Bash
Model: sonnet

## Orchestration

Create a task list with dependencies:
1. **Fix** — find and fix the bug → no dependencies
2. **Verify** — run existing tests to confirm no regressions → blocked by Fix
3. **Deploy** — commit and create a PR → blocked by Verify

Report the PR URL to user when done.

Start tasks in dependency order. Teammates self-claim unblocked tasks.

## Shutdown

When the task is complete:
1. Ask the lead to shut down all teammates gracefully
2. The lead sends shutdown requests and waits for confirmation
3. The lead cleans up the team (TeamDelete)

## Best Practices

- **Pre-approve permissions**: Before launching, suggest the user allow: file reads, file writes, test execution, git operations (commit, push, branch). Speed matters for hotfixes — permission prompts break flow.
- **Context management**: Teammates should pipe verbose test output to files instead of stdout. Use `--quiet` or `--summary` flags when available. Log errors with grep-friendly format (ERROR on the same line as the reason).
- **Give teammates context**: Include specific file paths, error messages, and relevant findings in spawn prompts — teammates don't inherit conversation history.
- **Enforce models**: When spawning each teammate with the Task tool, you MUST pass the `model` parameter explicitly. Agent YAML frontmatter `model:` is NOT enforced at runtime — the only binding control is the Task tool's `model` parameter. Treat omitting it as a bug.
- **Avoid file conflicts**: This team is sequential by design. If you ever spawn additional parallel agents for research, ensure they are read-only and don't write to files the Coder owns.

## When to Use

- Production bugs that need a quick fix
- Simple bugs with a known location
- Hotfixes that need to ship fast

## When NOT to Use

- Bugs requiring investigation across multiple systems (use full-product team)
- Bugs that need new tests written (use solo-sprint team)
- Features disguised as bug fixes
