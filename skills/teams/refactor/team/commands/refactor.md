---
description: Launch a 3-agent refactoring team — analyzer maps dependencies, refactorer restructures code, reviewer verifies behavioral equivalence
argument-hint: Refactoring target and objective (e.g. "extract auth logic from UserController into AuthService")
---

# Refactor Team

A 3-agent team for structural code changes that preserve behavior. The key difference from other teams: the reviewer checks for **behavioral regressions**, not code quality or security.

## Team Structure

### 1. Analyzer (Phase: RESEARCH)
Spawn an **analyzer** teammate with this prompt:

"You are a refactoring analyst. Read CLAUDE.md first. For the given refactoring target, produce a structured analysis: (1) Public API surface — every function, method, type, and export that external code depends on, (2) Dependency map — what the target imports and what imports the target, (3) Test coverage — which behaviors are covered by tests and which are not, (4) Risk areas — parts with no tests, complex side effects, or implicit contracts, (5) Ordered refactoring steps — a step-by-step plan where each step is independently testable. Do NOT modify any files."

Tools: Read, Glob, Grep, LSP, Bash (read-only)
Model: opus

### 2. Refactorer (Phase: CODE)
Spawn a **refactorer** teammate with this prompt:

"You are a structural refactorer. Read CLAUDE.md first. Execute the refactoring plan step-by-step. After EACH step, run the full test suite to confirm nothing broke. Rules: (1) Never change behavior — same inputs must produce same outputs, (2) Never modify test files — tests are the behavioral contract, (3) If a test fails after a step, revert that step and report the failure, (4) Preserve all public API signatures unless the plan explicitly calls for a rename (with a migration path), (5) Keep changes small and incremental — one concern per step."

Tools: Read, Write, Edit, Bash, Glob, Grep, LSP
Model: sonnet

### 3. Reviewer (Phase: REVIEW)
Spawn a **reviewer** teammate with this prompt:

"You are a behavioral equivalence reviewer for refactoring. Run git diff to see all changes. Your ONLY job is to verify that behavior did not change. Check for: (1) REGRESSION — a test that passed before now fails, or behavior clearly changed, (2) API BREAK — public function signatures, return types, error types, or exports changed, (3) STRUCTURAL — the refactoring goal was not fully achieved or left dead code, (4) INCOMPLETE — some steps from the plan were skipped. Do NOT review for security, performance, or code style — that is out of scope. Flag any modified test files as a finding (tests are the contract and should not change). Verdict: APPROVE (no regressions) or REQUEST CHANGES (regression or API break found)."

Tools: Read, Grep, Glob, Bash
Model: sonnet

## Orchestration

Create a task list with dependencies:
1. **Analyze** — map dependencies, API surface, test coverage → no dependencies
2. **Review plan** — lead evaluates the plan: if test coverage is too low or risks too high → stop and report to user
3. **Refactor** — execute plan step-by-step, run tests after each change → blocked by Review plan. Require plan approval before making changes.
4. **Behavioral review** — verify no regressions via diff + test analysis → blocked by Refactor

If Reviewer finds REGRESSION or API BREAK → send back to Refactorer (max 2 rounds).
After 2 failed rounds → escalate to user with findings.
Report completion — user commits when ready (no deployer).

Start tasks in dependency order. Teammates self-claim unblocked tasks.

## Shutdown

When the task is complete:
1. Ask the lead to shut down all teammates gracefully
2. The lead sends shutdown requests and waits for confirmation
3. The lead cleans up the team (TeamDelete)

## Best Practices

- **Pre-approve permissions**: Before launching, suggest the user allow: file reads, file writes (Refactorer only), test execution, `git diff`. The Analyzer is read-only and should not need write permissions.
- **Context management**: Teammates should pipe verbose test output to files instead of stdout. Use `--quiet` or `--summary` flags when available. Log errors with grep-friendly format (ERROR on the same line as the reason).
- **Give teammates context**: Include specific file paths, error messages, and relevant findings in spawn prompts — teammates don't inherit conversation history.
- **Enforce models**: When spawning each teammate with the Task tool, you MUST pass the `model` parameter explicitly. Agent YAML frontmatter `model:` is NOT enforced at runtime — the only binding control is the Task tool's `model` parameter. Treat omitting it as a bug.
- **Avoid file conflicts**: The Analyzer is read-only; the Refactorer owns all target files exclusively. Never spawn the Refactorer concurrently with any other agent that writes to the same files.
- **Lead stays coordinator**: The lead evaluates the Analyzer's plan (task 2) and approves or rejects it. The lead does not implement any refactoring steps itself.

## When to Use

- Extracting modules, services, or layers from existing code
- Renaming and reorganizing file structure
- Breaking up large files or functions
- Replacing implementation while keeping the same interface
- Moving code between directories or packages

## When NOT to Use

- Adding new features (use solo-sprint or full-product)
- Fixing bugs (use quick-fix)
- Changes that intentionally modify behavior (that's a feature, not a refactor)
- Code with no tests — add tests first, then refactor
