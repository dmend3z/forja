# Quick Fix Team

> Phase: **Teams**

Minimal 2-agent team for hotfixes: coder fixes the issue, deployer commits and creates PR.

## Team Members

| Role | Phase | Model |
|------|-------|-------|
| Coder | Code | sonnet |
| Deployer | Deploy | sonnet |

## How it works

The quick-fix team runs a 3-step workflow for fast hotfix delivery. The coder finds the root cause, fixes the bug with minimal changes, and runs existing tests to verify no regressions. Then the deployer creates a conventional commit with type 'fix', pushes to a new branch, and creates a PR describing what was broken, what caused it, and how it was fixed.

## When to use

- Fast hotfixes (1-3 files, single concern)
- Bug fixes with clear error messages
- When you need minimal testing and rapid deployment
- Production issues that need quick turnaround

## When NOT to use

- New features (use solo-sprint or full-product)
- Complex bugs requiring investigation
- Changes that need comprehensive testing

## Usage

```bash
/quick-fix Fix TypeError in login handler when email is missing
```

## Install

```bash
forja install teams/quick-fix/team
```
