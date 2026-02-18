# Full Product Team

> Phase: **Teams**

5-agent product development team: researcher, coder, tester, reviewer, deployer. Orchestrates parallel workflows across all forja phases.

## Team Members

| Role | Phase | Model |
|------|-------|-------|
| Researcher | Research | opus |
| Coder | Code | sonnet |
| Tester | Test | sonnet |
| Code-Simplifier | Review | sonnet |
| Reviewer | Review | sonnet |
| Chronicler | Document | haiku |
| Deployer | Deploy | sonnet |

## How it works

The full-product team creates a task list with dependencies across all phases. The researcher explores the codebase and produces a plan first. After plan approval, the coder implements the feature, then the tester writes and runs tests, the code-simplifier refines for clarity, the reviewer checks quality/security, the chronicler documents decisions, and finally the deployer commits and creates a PR. Tasks are assigned by role and run in dependency order with parallel execution where possible.

## When to use

- New features that touch multiple files (10+ files)
- Features that need research + implementation + tests + review
- Multi-phase, cross-cutting changes
- When you want the full development lifecycle automated

## When NOT to use

- Simple one-file fixes (use a single session instead)
- Quick bug fixes (use the quick-fix team)
- Tasks where steps are heavily sequential with no parallelism

## Usage

```bash
/full-product Add user authentication with JWT
```

## Install

```bash
forja install teams/full-product/team
```
