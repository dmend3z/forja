# Dispatch Team

> Phase: **Teams**

Parallel task dispatcher — fan-out independent work to background agents while you keep working in the main session.

## How it works

The dispatch pattern decomposes your request into discrete independent tasks and fans them out to specialized background agents. There's no TeamCreate, no shared task list, and no dedicated lead agent — just you dispatching tasks in parallel using the Task tool with `run_in_background: true`. The workflow is: (1) Decompose the request into independent tasks, (2) Map each task to the appropriate subagent_type and model, (3) Dispatch all tasks in a single message, (4) Monitor progress using TaskOutput, (5) Synthesize results when all agents complete.

## When to use

- Multiple independent tasks that can run in parallel
- Research + implementation + review happening simultaneously
- Exploring multiple approaches to compare results
- When you want to stay in control while delegating work

## When NOT to use

- All tasks depend on each other (do sequentially instead)
- Single task (no need for parallelism)
- When you need tight orchestration (use a team instead)

## Usage

```bash
/dispatch Research authentication patterns, implement rate limiting, and review security issues
```

The agent will decompose this into 3 independent tasks and dispatch them in parallel.

## Install

```bash
forja install teams/dispatch/team
```
