# Playwright E2E Tests

> Phase: **Test** | Tech: **e2e**

Playwright E2E tests with Page Object Model, auto-waiting, and trace-based debugging.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| e2e-tester | Read, Write, Edit, Bash, Grep, Glob | opus |

## What it does

The e2e-tester agent writes reliable, maintainable Playwright browser tests using the Page Object Model. It creates Page Objects for UI components, writes test specs with proper isolation, uses Playwright's auto-waiting features, and enables trace collection for debugging. Tests are organized by user flows and run across multiple browsers.

## Usage

After installing with `forja install test/e2e/playwright`:

```bash
# Use the e2e-tester agent for Playwright E2E tests
e2e-tester
```

The agent follows these patterns:
- Page Object Model for reusable UI abstractions
- Auto-waiting (no manual sleeps or timeouts)
- Test isolation (each test starts fresh)
- User-centric selectors (getByRole, getByLabel)
- Trace collection for debugging failures

## Install

```bash
forja install test/e2e/playwright
```
