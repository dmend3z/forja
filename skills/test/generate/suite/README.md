# Test Suite Generator

> Phase: **Test** | Tech: **generate**

Generate unit and integration tests for existing code. Analyzes source to produce comprehensive test suites.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| test-generator | Read, Write, Edit, Bash, Grep, Glob | opus |

## What it does

The test-generator agent writes comprehensive test suites for existing code. It reads source files, identifies the test framework, studies existing test patterns, and generates tests covering happy path, edge cases, and error conditions. It creates both unit tests (isolated functions with mocked dependencies) and integration tests (multiple components working together).

## Usage

After installing with `forja install test/generate/suite`:

```bash
# Use the test-generator agent to generate tests
test-generator
```

The agent will:
- Read source code to understand behavior
- Match existing test patterns and naming
- Generate unit tests (isolated, mocked dependencies)
- Generate integration tests (real dependencies)
- Run tests to verify they pass

## Install

```bash
forja install test/generate/suite
```
