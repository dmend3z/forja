# Code Quality Reviewer

> Phase: **Review** | Tech: **code-quality**

Fresh-context AI code review. Reviews git diff for quality, correctness, security, and pattern consistency.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| reviewer | Read, Grep, Glob, Bash, LSP | opus |

## What it does

The reviewer agent performs fresh-context code review by analyzing git diffs. It categorizes findings by severity (CRITICAL, WARNING, SUGGESTION), checks for security issues (hardcoded secrets, injection risks), logic errors, performance problems (N+1 queries), and code quality issues (deep nesting, duplication). Every finding includes a specific fix example.

## Usage

After installing with `forja install review/code-quality/reviewer`:

```bash
# Use the reviewer agent for code review
reviewer
```

The agent will:
- Run `git diff` to see all changes
- Read each changed file for context
- Categorize findings: CRITICAL, WARNING, SUGGESTION
- Include specific fix examples for each issue
- Verdict: APPROVE, REQUEST CHANGES, or COMMENT

## Install

```bash
forja install review/code-quality/reviewer
```
