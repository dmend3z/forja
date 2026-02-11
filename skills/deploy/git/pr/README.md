# Git Pull Request

> Phase: **Deploy** | Tech: **git**

Push branches and create Pull Requests via gh CLI with structured descriptions.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| pr-creator | Bash | sonnet |

## What it does

The pr-creator agent pushes branches and creates Pull Requests using the gh CLI. It checks branch tracking status, reviews changes with git log and git diff, pushes the branch with -u flag, creates the PR with a structured description (Summary, Changes, Test Plan), verifies creation, and outputs the PR URL.

## Usage

After installing with `forja install deploy/git/pr`:

```bash
# Use the pr-creator agent to create PRs
pr-creator
```

The agent will:
- Check current branch and remote status
- Review commits and changes
- Push branch to remote
- Create PR with structured description
- Output the PR URL

## Install

```bash
forja install deploy/git/pr
```
