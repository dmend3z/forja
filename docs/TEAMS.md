# Multi-Agent Teams

Teams let you orchestrate multiple Claude Code agents to work on a task together. Each agent has a specialized role (researcher, coder, tester, reviewer, deployer) and runs as a separate teammate within a single Claude Code session.

Teams require the experimental agent teams feature in Claude Code. Forja enables this automatically when you create or run a team, but you can also set it manually:

```json
// ~/.claude/settings.json
{
  "env": {
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
  }
}
```

## Team Presets

Forja ships with 3 built-in team configurations. Each preset maps agents to workflow phases and assigns models based on a profile.

### full-product

5 agents covering the full development lifecycle. Use for new features that touch multiple files and need research, implementation, testing, review, and deployment.

| # | Agent | Phase | Role |
|---|-------|-------|------|
| 1 | Researcher | RESEARCH | Explores codebase, maps architecture, identifies patterns and risks |
| 2 | Coder | CODE | Implements features following existing patterns |
| 3 | Tester | TEST | Writes tests (TDD), targets 80%+ coverage |
| 4 | Reviewer | REVIEW | Reviews for correctness, security (OWASP), performance |
| 5 | Deployer | DEPLOY | Creates conventional commits, pushes branch, opens PR |

Orchestration order: Researcher first, then Coder, then Tester and Reviewer in parallel, then Deployer after both approve.

### solo-sprint

2 agents for medium features (3-10 files) where you already understand the codebase and don't need a research phase.

| # | Agent | Phase | Role |
|---|-------|-------|------|
| 1 | Coder-Tester | CODE + TEST | Implements and tests in a single pass |
| 2 | Reviewer | REVIEW | Sprint-style review (concise, not a deep audit) |

Orchestration order: Coder-Tester first, then Reviewer. If the reviewer requests changes, findings go back to the Coder-Tester (max 2 rounds).

### quick-fix

2 agents for hotfixes and simple bugs that need to ship fast.

| # | Agent | Phase | Role |
|---|-------|-------|------|
| 1 | Coder | CODE | Finds root cause, applies minimal fix, runs existing tests |
| 2 | Deployer | DEPLOY | Commits with `fix(scope): ...`, pushes branch, creates PR |

Orchestration order: Coder fixes the bug, then Deployer commits and creates the PR.

## Model Profiles

Each profile controls which Claude model is assigned to each agent based on its phase. Thinking phases (Research, Review) benefit from stronger reasoning; execution phases (Code, Test, Deploy) prioritize speed.

| Profile | Thinking phases | Execution phases | Description |
|---------|-----------------|------------------|-------------|
| `fast` | sonnet | sonnet | Fastest, lowest cost |
| `balanced` | opus | sonnet | Opus for research and review, sonnet for the rest |
| `max` | opus | opus | Highest quality across all agents |

Thinking phases: **Research**, **Review**
Execution phases: **Code**, **Test**, **Deploy**

The default profile is `balanced`.

## Team Commands

### Create a team from a preset

```bash
forja team preset full-product
forja team preset solo-sprint --profile fast
forja team preset quick-fix --profile max
```

This creates a slash command at `~/.claude/commands/forja--team--<name>.md` and saves the team configuration to `~/.forja/state.json`. The `--profile` flag defaults to `balanced`.

### Create a custom team

```bash
forja team create my-team
```

Launches an interactive wizard:

1. Select agents from your installed skills (multi-select)
2. Choose a model profile (fast / balanced / max)
3. Review the summary and confirm

The wizard generates a slash command that orchestrates the selected agents in phase order (research before code, code before test, and so on).

### List configured teams

```bash
forja team list
```

Shows all teams with their profile, agent count, and whether the slash command file exists.

### Show team details

```bash
forja team info full-product
```

Displays the team's profile, all members with their skill IDs, and the assigned model for each agent.

### Delete a team

```bash
forja team delete my-team
```

Removes the slash command file and the team entry from state.

## Running a Team

There are three ways to run a team:

### Using the slash command

After creating a team (via `preset` or `create`), a slash command is available in Claude Code:

```
/forja--team--full-product Add user authentication with JWT
```

This tells Claude Code to spawn the agents defined in the team and orchestrate them in order.

### Using forja task

```bash
forja task "add user auth with JWT" --team full-product
forja task "fix the login bug" --team quick-fix --profile fast
```

The `--team` flag accepts any configured team name or a preset name. If the preset hasn't been configured yet, forja resolves it in-memory without persisting. The `--profile` flag overrides the team's default profile for this run.

Without `--team`, `forja task` shows an interactive picker listing all presets and configured teams.

### Using the plan/execute workflow

For complex tasks, split planning from execution:

```bash
# Step 1: Create a plan (interactive interview + codebase research)
forja plan "add user auth with JWT"
```

This launches a Claude Code session that:

1. Interviews you about the task (project type, goals, scope, depth)
2. Spawns a researcher subagent to explore the codebase
3. Auto-detects the stack and selects appropriate agents
4. Sizes the team (quick-fix / solo-sprint / full-product) based on complexity
5. Builds structured implementation phases with dependencies
6. Saves two files to `~/.forja/plans/`:
   - `{plan-id}.json` -- machine-readable metadata (agents, phases, quality gates)
   - `{plan-id}.md` -- human-readable plan document

```bash
# Step 2: Execute the plan
forja execute
forja execute --profile max
forja execute 20260208-143022-user-auth-jwt
```

Without a plan ID, `forja execute` picks up the latest pending plan. It:

1. Reads the plan's phases, agents, and quality gates
2. Auto-installs any missing agent skills
3. Enables the agent teams env var if needed
4. Launches a Claude Code session as the team orchestrator
5. Marks the plan as `executed` on success

The `--profile` flag overrides the plan's default profile (balanced).

## Choosing the Right Approach

| Scenario | Approach |
|----------|----------|
| Simple bug fix, known location | `forja task "..." --team quick-fix` |
| Medium feature, familiar codebase | `forja task "..." --team solo-sprint` |
| Large feature, needs research | `forja plan "..." && forja execute` |
| Recurring workflow with specific agents | `forja team create my-team`, then slash command |
| One-off task, no team needed | `forja task "..."` (solo mode) |
