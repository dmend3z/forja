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

Forja ships with 6 built-in team configurations. Each preset maps agents to workflow phases and assigns models based on a profile.

### full-product

7 agents covering the full development lifecycle. Use for new features that touch multiple files and need research, implementation, testing, simplification, review, and deployment.

| # | Agent | Phase | Role |
|---|-------|-------|------|
| 1 | Researcher | RESEARCH | Explores codebase, maps architecture, identifies patterns and risks |
| 2 | Coder | CODE | Implements features following existing patterns |
| 3 | Tester | TEST | Writes tests (TDD), targets 80%+ coverage |
| 4 | Code-Simplifier | REVIEW | Simplifies complex code, removes duplication, improves readability |
| 5 | Reviewer | REVIEW | Reviews for correctness, security (OWASP), performance |
| 6 | Chronicler | DOCUMENT | Documents decisions made during the workflow (context, rationale, alternatives) |
| 7 | Deployer | DEPLOY | Creates conventional commits, pushes branch, opens PR |

**Usage:**
```bash
forja team preset full-product
forja task "add user authentication with JWT" --team full-product
```

Orchestration order: Researcher → Coder → Tester → Code-Simplifier → Reviewer → Chronicler → Deployer (deploy blocked by test, review, and chronicle).

### solo-sprint

3 agents for medium features (3-10 files) where you already understand the codebase and don't need a research phase.

| # | Agent | Phase | Role |
|---|-------|-------|------|
| 1 | Coder-Tester | CODE + TEST | Implements and tests in a single pass |
| 2 | Code-Simplifier | CODE | Simplifies and improves readability |
| 3 | Reviewer | REVIEW | Sprint-style review (concise, not a deep audit) |

**Usage:**
```bash
forja team preset solo-sprint --profile fast
forja task "add pagination to user list" --team solo-sprint
```

Orchestration order: Coder-Tester first, then Code-Simplifier, then Reviewer. If the reviewer requests changes, findings go back to the Coder-Tester (max 2 rounds).

### quick-fix

2 agents for hotfixes and simple bugs that need to ship fast.

| # | Agent | Phase | Role |
|---|-------|-------|------|
| 1 | Coder | CODE | Finds root cause, applies minimal fix, runs existing tests |
| 2 | Deployer | DEPLOY | Commits with `fix(scope): ...`, pushes branch, creates PR |

**Usage:**
```bash
forja team preset quick-fix
forja task "fix login redirect bug" --team quick-fix
```

Orchestration order: Coder fixes the bug, then Deployer commits and creates the PR.

### dispatch

1 dispatcher agent for parallel task orchestration. Lightweight by design — no TeamCreate, no shared task list, just you + N background Task agents.

| # | Agent | Phase | Role |
|---|-------|-------|------|
| 1 | Dispatcher | TEAMS | Decomposes work into independent tasks, spawns background agents in parallel (max 5), collects and synthesizes results |

The dispatcher follows a 5-step workflow: **Decompose** (break into discrete tasks) → **Map** (select agent type and model per task) → **Dispatch** (spawn all independent tasks in a single message with `run_in_background: true`) → **Continue** (return control immediately) → **Collect** (read output files and synthesize).

**Usage:**
```bash
forja team preset dispatch
forja task "add validation to all API endpoints" --team dispatch
forja task "research React, Vue, and Svelte for our frontend" --team dispatch
```

**When to use:** Research multiple approaches in parallel, fix N unrelated bugs simultaneously, run review + tests + security audit in parallel, explore different parts of an unfamiliar codebase.

**When NOT to use:** Sequential work where each step depends on the previous (use solo-sprint or full-product), tasks requiring agents to communicate mid-task (use a proper team with TeamCreate).

### tech-council

1 facilitator agent that spawns 5 engineering personas in parallel. Use when you need diverse technical perspectives on architecture, technology choices, or engineering trade-offs.

| # | Agent | Phase | Role |
|---|-------|-------|------|
| 1 | Council-Facilitator | REVIEW | Dispatches 5 background agents (Principal Engineer, Platform Engineer, Security Engineer, QA Lead, Performance Engineer) and synthesizes their analysis |

The facilitator spawns each persona as a background `general-purpose` agent with `model: opus`. Each persona analyzes the question from their specific bias, then the facilitator synthesizes a consensus, highlights tensions, and produces a recommendation.

**Usage:**
```bash
forja team preset tech-council
forja task "should we use GraphQL or REST for the new API?" --team tech-council
forja task "monolith vs microservices for our payment system?" --team tech-council
```

**When to use:** Architecture decisions, technology evaluations, migration planning, trade-off analysis, system design review.

**When NOT to use:** Pure implementation questions (use a coder), non-technical decisions (use biz-council), or questions with obvious answers.

### biz-council

1 facilitator agent that spawns 5 business personas in parallel. Use when you need diverse strategic perspectives on product decisions, go-to-market planning, or business model evaluation.

| # | Agent | Phase | Role |
|---|-------|-------|------|
| 1 | Strategic-Facilitator | REVIEW | Dispatches 5 background agents (Product Lead, Design Lead, Data/Analytics Lead, Growth Lead, Operations Lead) and synthesizes their analysis |

The facilitator spawns each persona as a background `general-purpose` agent with `model: opus`. Each persona evaluates the question from their specific bias, then the facilitator synthesizes a consensus, highlights tensions, and produces a strategic recommendation.

**Usage:**
```bash
forja team preset biz-council
forja task "pricing strategy for enterprise tier" --team biz-council
forja task "should we launch a free tier or go paid-only?" --team biz-council
```

**When to use:** Product decisions, go-to-market planning, resource allocation, strategic trade-offs, business model evaluation.

**When NOT to use:** Technical architecture decisions (use tech-council), pure implementation questions (use a coder), or decisions already made.

## Slash Command Teams

Some teams are available only as slash commands (not CLI presets). They provide specialized workflows accessed directly in Claude Code.

### /refactor

3 agents for structural code changes that preserve behavior. Use when you need to extract modules, reorganize files, or restructure code — but the external behavior must stay the same.

| # | Agent | Phase | Role |
|---|-------|-------|------|
| 1 | Analyzer | RESEARCH | Maps dependencies, callers, test coverage, public API surface. Read-only. |
| 2 | Refactorer | CODE | Executes the refactoring plan step-by-step. Runs tests after each change. |
| 3 | Reviewer | REVIEW | Verifies behavioral equivalence — flags regressions, API breaks. Does NOT review for security or performance. |

**Usage:**
```
/refactor Extract user validation logic into a separate module
```

Orchestration order: Analyzer produces a plan, Lead evaluates (stops if test coverage too low), Refactorer executes, Reviewer checks for regressions. Max 2 review rounds before escalating. No deployer — user commits when ready.

**Note:** Install with `forja install teams/refactor/team` to enable the `/refactor` slash command.

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
forja team preset dispatch
forja team preset tech-council
forja team preset biz-council
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
forja task "add validation to all API endpoints" --team dispatch
forja task "should we migrate to microservices?" --team tech-council
forja task "pricing strategy for enterprise tier" --team biz-council
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
| Large feature, needs research | `forja plan "..." && forja execute` or `forja task "..." --team full-product` |
| Restructuring code without changing behavior | `/refactor Extract module X` (slash command) |
| Multiple independent tasks in parallel | `forja task "..." --team dispatch` |
| Technical architecture decision | `forja task "..." --team tech-council` |
| Business strategy decision | `forja task "..." --team biz-council` |
| Recurring workflow with specific agents | `forja team create my-team`, then slash command |
| One-off task, no team needed | `forja task "..."` (solo mode) |
