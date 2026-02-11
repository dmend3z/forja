# Forja Plan

> Phase: **Research** | Tech: **planning**

Planning pipeline: interview, research, auto-detect agents, auto-size team, save executable plan to ~/.forja/plans/

## Slash Command

| Command | Description |
|---------|-------------|
| /forja-plan | Creates an executable implementation plan through 5 steps |

## How it works

The forja-plan command runs a 5-step planning pipeline: (1) Interview - asks questions about project type, goals, exclusions, and planning depth. (2) Research - spawns an Explore subagent to map the codebase and detect stack. (3) Agents & Team Sizing - auto-detects required skills and suggests team size. (4) Build Phases - creates implementation phases with dependencies. (5) Save Plan - generates both JSON metadata and Markdown plan files in `~/.forja/plans/`.

## Usage

After installing with `forja install research/planning/forja-plan`:

```bash
/forja-plan Add user authentication with JWT
```

The command will:
- Interview you with 4-8 questions (depending on depth)
- Research the codebase to detect stack and patterns
- Suggest agents and team size
- Build implementation phases with dependencies
- Save executable plan to `~/.forja/plans/{plan-id}.json` and `.md`

## Install

```bash
forja install research/planning/forja-plan
```
