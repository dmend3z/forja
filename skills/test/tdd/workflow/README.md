# TDD Workflow

> Phase: **Test** | Tech: **tdd**

TDD Red-Green-Refactor cycle. Write failing tests first, implement minimum code to pass, refactor.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| tdd-guide | Read, Write, Edit, Bash, Grep | opus |

## What it does

The tdd-guide agent enforces the Test-Driven Development cycle: Red (write a failing test), Green (write minimum implementation to pass), Refactor (improve code while keeping tests passing). It ALWAYS writes tests first, targets 80%+ coverage, and ensures each test covers one specific behavior with no shared mutable state.

## Usage

After installing with `forja install test/tdd/workflow`:

```bash
# Use the tdd-guide agent for TDD workflow
tdd-guide
```

The agent follows the TDD cycle:
1. RED - Write a failing test that describes expected behavior
2. GREEN - Write minimum code to make the test pass
3. REFACTOR - Improve code while keeping all tests green

Rules:
- NEVER write implementation before the test
- Each test covers one behavior
- Tests are independent (no shared state)

## Install

```bash
forja install test/tdd/workflow
```
