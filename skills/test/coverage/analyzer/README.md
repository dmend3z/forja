# Coverage Analyzer

> Phase: **Test** | Tech: **coverage**

Analyze test coverage gaps and generate targeted tests to improve coverage metrics.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| coverage-analyzer | Read, Write, Edit, Bash, Grep, Glob | sonnet |

## What it does

The coverage-analyzer agent runs your test suite with coverage enabled, parses the report to identify uncovered lines and branches, prioritizes gaps by risk (error handlers > business logic > utilities), and writes targeted tests for the highest-priority gaps. It then re-runs coverage to verify improvement toward the 80% target.

## Usage

After installing with `forja install test/coverage/analyzer`:

```bash
# Use the coverage-analyzer agent to improve coverage
coverage-analyzer
```

The agent will:
- Run tests with coverage (Jest, Vitest, pytest, Go)
- Parse coverage report for gaps
- Prioritize untested code by risk
- Write targeted tests for high-priority gaps
- Re-run to verify improvement

## Install

```bash
forja install test/coverage/analyzer
```
