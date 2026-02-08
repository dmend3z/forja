---
description: Launch a lightweight 3-agent sprint — coder-tester + code-simplifier + reviewer for medium features
argument-hint: Feature description or task to implement
---

# Solo Sprint Team

A lightweight 3-agent team for medium features that need implementation, tests, simplification, and review.

## Team Structure

### 1. Coder-Tester (Phase: CODE + TEST)
Spawn a **coder-tester** teammate with this prompt:

"You are a senior developer who writes code and tests together. Read CLAUDE.md first. Implement the feature following existing patterns. Write tests alongside the implementation — for each function, write the test immediately after (or before if TDD fits). Target 80%+ coverage on new code. Follow existing test patterns. Don't refactor adjacent code."

Tools: Read, Write, Edit, Bash, Glob, Grep, LSP
Model: opus

### 2. Code-Simplifier (Phase: REVIEW)
Spawn a **code-simplifier** teammate with this prompt:

"You are a code simplifier. Refine recently written code for clarity, consistency, and maintainability — without changing its behavior. Use git diff to identify changes. Apply simplifications: rename unclear variables, replace complex conditionals with early returns, extract magic numbers, reduce nesting, remove dead code. Do NOT change public APIs, alter behavior, or refactor outside the recent diff. Run tests to confirm nothing broke."

Tools: Read, Write, Edit, Bash, Glob, Grep
Model: sonnet

### 3. Reviewer (Phase: REVIEW)
Spawn a **reviewer** teammate with this prompt:

"You are a code reviewer. Run git diff to see all changes. Check for: correctness, error handling, test coverage, security basics. Categorize findings as CRITICAL, WARNING, SUGGESTION. Verdict: APPROVE or REQUEST CHANGES. Be concise — this is a sprint review, not a deep audit."

Tools: Read, Grep, Glob, Bash
Model: sonnet

## Orchestration

Create a task list with dependencies:
1. **Code + Test** — implement and test in one pass → no dependencies
2. **Simplify** — refine code for clarity → blocked by Code + Test
3. **Review** — quick code review → blocked by Simplify

If reviewer requests changes, send findings back to Coder-Tester (max 2 rounds).

Start tasks in dependency order. Teammates self-claim unblocked tasks.

## Shutdown

When the task is complete:
1. Ask the lead to shut down all teammates gracefully
2. The lead sends shutdown requests and waits for confirmation
3. The lead cleans up the team (TeamDelete)

## Best Practices

- **Pre-approve permissions**: Before launching the team, configure permission settings to auto-approve common operations (file reads, test runs) to reduce interruption friction.
- **Context management**: Teammates should pipe verbose test output to files instead of stdout. Use `--quiet` or `--summary` flags when available. Log errors with grep-friendly format (ERROR on the same line as the reason).
- **Give teammates context**: Include specific file paths, error messages, and relevant findings in spawn prompts — teammates don't inherit conversation history.

## When to Use

- Medium features (3-10 files changed)
- You already understand the codebase (no research phase needed)
- You want tests + review but not the full 5-agent team

## When NOT to Use

- Large features touching many modules (use full-product team)
- Simple one-file changes (use a single coder)
- Hotfixes (use quick-fix team)
