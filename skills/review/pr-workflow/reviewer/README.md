# PR Workflow Reviewer

> Phase: **Review** | Tech: **pr-workflow**

Full PR review lifecycle: create review, iterate on feedback, approve or request changes.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| pr-reviewer | Read, Grep, Glob, Bash | opus |

## What it does

The pr-reviewer agent manages the complete PR review lifecycle. It fetches the PR diff using gh CLI, reads each changed file for context, reviews against quality/security/performance criteria, writes structured review comments, and sets a verdict (APPROVE, REQUEST CHANGES, or COMMENT). It understands the intent before judging implementation.

## Usage

After installing with `forja install review/pr-workflow/reviewer`:

```bash
# Use the pr-reviewer agent for full PR review
pr-reviewer
```

The agent follows a three-pass process:
1. Understanding - Read PR description and map changes
2. Correctness - Check logic, edge cases, error handling
3. Quality - Review patterns, naming, performance, security

## Install

```bash
forja install review/pr-workflow/reviewer
```
