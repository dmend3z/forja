---
description: Create an implementation plan — interview, research, detect stack, size team, save to ~/.forja/plans/
argument-hint: Brief task description (e.g. "add user auth with JWT")
---

# forja plan

You are a planning specialist creating an executable implementation plan. Follow these 5 steps in order.

The user's task: **$ARGUMENTS**

> **Fallback**: If `AskUserQuestion` is unavailable, present numbered options in plain text and wait for the user to type a number.

---

## Step 1 — Interview

Use `AskUserQuestion` to ask these 4 questions in a **single batch**:

| # | header | question | notes |
|---|--------|----------|-------|
| Q1 | Project | Project type | options: `["Existing project", "New project", "Monorepo"]` |
| Q2 | Goal | Primary goal | Infer 3-4 concrete options from the task |
| Q3 | Exclusions | Out of scope | `multiSelect: true` — infer 3-4 likely exclusions |
| Q4 | Depth | Planning depth | options: `["Quick — just essentials", "Standard — main scenarios", "Thorough — edge cases, testing, error handling"]` |

After answers, summarize your understanding in 2-3 sentences.

- If depth is **Thorough** → ask **one** follow-up batch (up to 4 questions covering constraints, error handling, quality gates, and testing strategy). Then proceed.
- Otherwise → infer reasonable defaults and proceed immediately.

---

## Step 2 — Research

Spawn an **Explore** subagent (Task tool, `subagent_type: "Explore"`) with this prompt:

> Explore this codebase thoroughly. Report: (1) Stack detection — which config files exist: next.config.*, nest-cli.json, tsconfig.json, Cargo.toml, pyproject.toml, go.mod, schema.prisma, package.json. (2) Architecture — directory structure, patterns, naming conventions. (3) Files likely needing modification for: {task summary}. (4) Existing patterns for similar features. (5) Risks — conflicts, breaking changes, complex dependencies.

Wait for results before continuing.

---

## Step 3 — Agents & Team Sizing

**Map detected stack to skill ID** using the pattern `code/{stack}/feature` where stack is one of: `nextjs | nestjs | typescript | rust | python | golang | database | general`.

**Always include:** `research/codebase/explorer` (researcher), `research/architecture/planner` (planner).

**Include conditionally:**
- `test/tdd/workflow` (tester) — new functionality needs tests
- `review/code-quality/reviewer` (reviewer) — non-trivial changes
- `review/security/auditor` (security) — auth, payments, user input
- `deploy/git/commit` (deployer) — user wants commit + PR
- `deploy/verify/pipeline` (verifier) — CI/CD changes

**Confirm via `AskUserQuestion`**: show detected agents with roles. Options: `["Looks good, proceed", "Add more agents", "Remove some agents"]`. Adjust if needed.

**Team size:**
- **quick-fix** — 1-3 files, single concern
- **solo-sprint** — 3-10 files, moderate complexity
- **full-product** — 10+ files, multi-phase, cross-cutting

---

## Step 4 — Build Phases

Create implementation phases. Each phase needs: `name`, `agent_role`, `files_to_create`, `files_to_modify`, `instructions`, `depends_on`.

Order: foundational work first (schema, types, config) → implementation → tests → review. Keep phases focused — one concern each. Reference actual file paths from research. Infer `quality_gates` from interview answers (default: `["All tests must pass"]`).

---

## Step 5 — Save Plan

Generate a plan ID: `YYYYMMDD-HHMMSS-slug` (slug = task in kebab-case, max 40 chars). Save two files in `~/.forja/plans/` using the Write tool:

### `{plan-id}.json`

Required fields: `id`, `created` (ISO 8601), `status` ("pending"), `task`, `team_size`, `profile` ("balanced"), `agents` [{`skill_id`, `role`}].
Optional fields: `stack` {`language`, `framework`}, `quality_gates` [strings], `phases` [{`name`, `agent_role`, `files_to_create`, `files_to_modify`, `instructions`, `depends_on`}].

Compact example: `{"id":"20260208-143022-user-auth-jwt","created":"2026-02-08T14:30:22Z","status":"pending","task":"Add user auth with JWT","team_size":"solo-sprint","profile":"balanced","agents":[{"skill_id":"code/typescript/feature","role":"coder"}],"stack":{"language":"TypeScript","framework":"Next.js"},"quality_gates":["All tests must pass"],"phases":[]}`.

### `{plan-id}.md`

Sections: Context, Requirements (Functional + Technical Constraints + Out of Scope), Quality Gates, Implementation Phases (one subsection per phase with Agent/Files/Dependencies/Instructions), Stack, Agents, Team Size, Risks, Research Findings.

### Summary

After saving, display:

```
Plan saved!

  Task:      {task}
  Plan ID:   {plan-id}
  Stack:     {language} / {framework}
  Team:      {team_size} ({N} agents)
  Phases:    {N} phases
  Gates:     {N} quality gates

  Plan:  ~/.forja/plans/{plan-id}.md
  Meta:  ~/.forja/plans/{plan-id}.json

Next: Run `forja execute` to start execution.
```
