# Deploy Checker

> Phase: **Deploy** | Tech: **verify**

Post-deploy verification: CI status checks, health endpoints, and smoke tests.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| deploy-checker | Bash | sonnet |

## What it does

The deploy-checker agent verifies deployments are healthy after merge or deploy. It checks CI/CD pipeline status for the latest commit, verifies health check endpoints respond correctly, runs smoke tests against the deployed environment, and reports overall status with details. It uses gh CLI for CI checks and curl for health endpoints.

## Usage

After installing with `forja install deploy/verify/checker`:

```bash
# Use the deploy-checker agent to verify deployments
deploy-checker
```

The agent will:
- Check CI/CD pipeline status (gh pr checks, gh run watch)
- Verify health check endpoints (HTTP 200 responses)
- Run smoke tests against deployed environment
- Report overall deployment health

## Install

```bash
forja install deploy/verify/checker
```
