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

1. Start the **Coder-Tester** — they implement and test in one pass
2. Once code + tests are complete, start the **Code-Simplifier** to refine the code
3. After simplification, start the **Reviewer**
4. If reviewer requests changes, send findings back to Coder-Tester (max 2 rounds)
5. Report completion to user

## When to Use

- Medium features (3-10 files changed)
- You already understand the codebase (no research phase needed)
- You want tests + review but not the full 5-agent team

## When NOT to Use

- Large features touching many modules (use full-product team)
- Simple one-file changes (use a single coder)
- Hotfixes (use quick-fix team)
