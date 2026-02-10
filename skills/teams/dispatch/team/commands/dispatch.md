---
description: Dispatch independent tasks to parallel background agents — you stay in control
argument-hint: List of tasks or a multi-part request to parallelize
---

# Dispatch — Parallel Task Delegation

You are now operating as a **tech lead dispatching work to background agents**. Instead of doing everything sequentially, you decompose the request into independent tasks and fan them out to specialized agents running in the background.

No TeamCreate. No shared task list. No dedicated lead agent. Just you + N background Task agents. Lightweight by design.

## Workflow

### 1. Decompose

Break the user's request into discrete tasks. For each task, determine:
- **What**: Clear, specific description of the deliverable
- **Independent?**: Can it run without waiting for other tasks?
- **Agent type**: Which subagent_type fits best
- **Model**: Which model to use (see table below)

Separate independent tasks (parallel) from dependent tasks (sequential). If ALL tasks are dependent on each other, this is the wrong tool — do them sequentially instead.

### 2. Map — Agent Selection

| Task Type | subagent_type | model |
|-----------|---------------|-------|
| Codebase exploration | Explore | opus |
| External research | Explore | opus |
| Implementation | coder or stack-specific | opus |
| Test writing | general-purpose | opus |
| Code review | code-reviewer | sonnet |
| Architecture planning | Plan | opus |
| Deployment / git ops | general-purpose | sonnet |

### 3. Dispatch

Spawn all independent tasks in a **SINGLE message** with multiple Task tool calls, each with `run_in_background: true`.

Critical rules for each spawn:
- **MUST pass `model` explicitly** — agent YAML `model:` is NOT enforced at runtime
- **Prompt must be self-contained** — include file paths, error messages, and all relevant context. Background agents don't see this conversation.
- **Name each agent clearly** — use descriptive names like "research-auth-patterns" or "fix-login-bug"

Example dispatch pattern:
```
Task 1: { subagent_type: "Explore", model: "opus", run_in_background: true, prompt: "..." }
Task 2: { subagent_type: "coder", model: "opus", run_in_background: true, prompt: "..." }
Task 3: { subagent_type: "code-reviewer", model: "sonnet", run_in_background: true, prompt: "..." }
```

### 4. Continue

After dispatching, immediately tell the user:
- How many agents were spawned and what each is doing
- The output file path for each agent (from the Task tool response)
- That they can keep working — agents run independently

Do NOT block the conversation waiting for agents to finish.

### 5. Collect

When the user asks for results (or when you notice agents have completed):
- Read each agent's output file using the Read tool
- Synthesize results into a concise summary
- Flag any conflicts or issues between agent outputs
- If any agent failed, explain what happened and offer to retry

## Shutdown

No explicit shutdown needed. Background agents self-terminate when their task completes. No TeamDelete required.

## Best Practices

- **Pre-approve permissions**: Suggest the user configure permission settings to auto-approve common operations before dispatching, to avoid N agents each waiting for approval.
- **Self-contained prompts**: The #1 cause of bad agent output is insufficient context in the prompt. Include: the goal, relevant file paths, error messages, constraints, and what format to return results in.
- **Right-size the work**: Each dispatched task should be substantial enough to justify a separate agent (not a 2-line change) but focused enough to complete independently.
- **Max 5 agents**: More creates diminishing returns. Permission approval fatigue alone makes >5 impractical.
- **Model enforcement**: ALWAYS pass `model` in the Task tool call. This is non-negotiable.

## When to Use

- Research multiple libraries or approaches in parallel
- Fix N unrelated bugs simultaneously
- Update docs across independent modules
- Analyze multiple Sentry issues at once
- Run code review + test generation + security audit in parallel
- Explore different parts of an unfamiliar codebase

## When NOT to Use

- Sequential work where each step depends on the previous one (use solo-sprint or full-product)
- A single focused task (just do it directly)
- Tasks requiring shared mutable state between agents (use a proper team with TeamCreate)
- When you need agents to communicate with each other mid-task (use full-product team)
