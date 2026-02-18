# Solo Sprint Team

> Phase: **Teams**

Lightweight 2-agent team: combined coder-tester and quick reviewer. For medium features that need tests and review.

## Team Members

| Role | Phase | Model |
|------|-------|-------|
| Coder-Tester | Code + Test | sonnet |
| Code-Simplifier | Review | sonnet |
| Reviewer | Review | sonnet |

## How it works

The solo-sprint team runs a 3-step workflow. The coder-tester implements the feature and writes tests together in one pass (targeting 80%+ coverage). Then the code-simplifier refines the code for clarity without changing behavior. Finally, the reviewer performs a quick code review checking correctness, error handling, and security basics. If changes are requested, they go back to the coder-tester (max 2 rounds).

## When to use

- Medium features (3-10 files)
- Moderate complexity tasks
- When you need implementation, tests, and review
- Faster than full-product, more thorough than quick-fix

## When NOT to use

- Large multi-phase features (use full-product team)
- Simple hotfixes (use quick-fix team)
- Refactoring work (use refactor team)

## Usage

```bash
/solo-sprint Add pagination to user list endpoint
```

## Install

```bash
forja install teams/solo-sprint/team
```
