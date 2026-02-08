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

1. Start the **Coder** — they find and fix the bug
2. Verify existing tests pass after the fix
3. Start the **Deployer** — they commit and create a PR
4. Report the PR URL to user

## When to Use

- Production bugs that need a quick fix
- Simple bugs with a known location
- Hotfixes that need to ship fast

## When NOT to Use

- Bugs requiring investigation across multiple systems (use full-product team)
- Bugs that need new tests written (use solo-sprint team)
- Features disguised as bug fixes
