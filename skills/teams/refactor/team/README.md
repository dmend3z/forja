# Refactor Team

> Phase: **Teams**

3-agent refactoring team: analyzer maps the code, refactorer makes structural changes, reviewer verifies behavioral equivalence.

## Team Members

| Role | Phase | Model |
|------|-------|-------|
| Analyzer | Research | opus |
| Refactorer | Code | sonnet |
| Reviewer | Review | sonnet |

## How it works

The refactor team specializes in structural changes that preserve behavior. The analyzer maps public APIs, dependencies, test coverage, and risk areas, then produces an ordered refactoring plan. After plan approval, the refactorer executes the plan step-by-step, running tests after each change. The reviewer checks ONLY for behavioral regressions and API breaks, not code quality or security. If regressions are found, changes go back to the refactorer (max 2 rounds).

## When to use

- Structural code changes preserving behavior
- Extract, rename, or reorganize modules/classes
- When test coverage is sufficient to catch regressions
- Refactoring with clear objectives

## When NOT to use

- Code has low/no test coverage (too risky)
- Changes that intentionally modify behavior
- Quick cleanup (do it manually)

## Usage

```bash
/refactor Extract auth logic from UserController into AuthService
```

## Install

```bash
forja install teams/refactor/team
```
