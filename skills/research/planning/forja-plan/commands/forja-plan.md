---
description: Create an implementation plan — interview, research, detect stack, size team, save to ~/.forja/plans/
argument-hint: Brief task description (e.g. "add user auth with JWT")
---

# forja plan

You are a planning specialist creating an executable implementation plan. Follow these 6 steps in order.

The user's task: $ARGUMENTS

> **Fallback**: If `AskUserQuestion` tool is not available, present numbered options in plain text and wait for the user to type a number.

## Step 1 — Core Interview (always)

Use `AskUserQuestion` to ask these 4 questions in a single batch:

**Q1 — Project type**
- header: "Project"
- options: `["Existing project", "New project", "Monorepo"]`

**Q2 — Primary goal**
- header: "Goal"
- Infer 3-4 concrete goal options from `$ARGUMENTS`. Example: if task is "add user auth with JWT", offer `["Login/signup flow", "API token auth", "Role-based access control", "Session management"]`.

**Q3 — Out of scope**
- header: "Exclusions"
- multiSelect: true
- Infer 3-4 likely exclusions from the task. Example: `["UI/frontend changes", "Database migrations", "CI/CD pipeline", "Documentation"]`.

**Q4 — Depth**
- header: "Depth"
- options: `["Quick — just essentials", "Standard — main scenarios", "Thorough — edge cases, testing, error handling"]`

After receiving answers, summarize your understanding in 3-4 sentences. Then ask:

**Continue or proceed?**
- header: "Next"
- options: `["Go deeper", "Proceed to research"]`

If "Proceed to research" → skip to Step 2.

## Step 1b — Deeper Interview (conditional)

Branch questions by project type from Q1:

**If "Existing project"** — ask 4 questions:
- Area of codebase affected (infer 3-4 options from research context)
- Constraints (infer 3-4 likely constraints: backward compat, API stability, etc.)
- Error handling strategy: `["Return errors to caller", "Log and recover", "Fail fast", "Mixed — depends on layer"]`
- Quality gates (multiSelect): `["All tests must pass", "No lint warnings", "Security audit", "Performance benchmarks"]`

**If "New project"** — ask 4 questions:
- Target stack (infer 3-4 options from task)
- Project structure: `["Flat — minimal directories", "Standard — framework default", "Domain-driven — feature folders", "Monorepo — workspaces"]`
- Quality gates (multiSelect): `["All tests must pass", "No lint warnings", "Security audit", "Performance benchmarks"]`
- Initial scope: `["MVP — minimal viable", "Full feature — complete implementation", "Prototype — quick validation"]`

**If "Monorepo"** — ask 4 questions:
- Which packages/apps are affected (infer from codebase)
- Shared dependencies: `["Shared types/utils", "Shared UI components", "Shared config", "Independent packages"]`
- Quality gates (multiSelect): `["All tests must pass", "No lint warnings", "Security audit", "Performance benchmarks"]`
- Cross-package concerns: `["API contracts", "Versioning", "Build order", "None — independent"]`

After receiving answers, summarize updates. Then ask:

**Continue or proceed?**
- header: "Next"
- options: `["Go deeper", "Proceed to research"]`

If "Proceed to research" → skip to Step 2.

## Step 1c — Final Deep Dive (conditional, last batch)

Ask up to 4 questions about remaining details:
- Data models / schema changes (infer options from task)
- Integration points with external services (infer options)
- Deployment or rollout concerns: `["No special concerns", "Feature flag needed", "Database migration required", "Breaking API change"]`
- Testing strategy: `["Unit tests only", "Unit + integration", "Unit + integration + e2e", "TDD — tests first"]`

After this batch, proceed to Step 2 — no more interview batches.

**Defaults when user skips deeper batches:** Infer reasonable defaults from the task description. For example, "add auth with JWT" → include security auditor agent, quality gates = `["All tests must pass"]`, error handling = "Mixed — depends on layer".

## Step 2 — Research

Spawn a **researcher** subagent (using Task tool with subagent_type "Explore") to explore the codebase. Give it this prompt:

"Explore this codebase thoroughly. I need:
1. **Stack detection**: Look for these config files and report which exist:
   - `next.config.*` (Next.js)
   - `nest-cli.json` or `@nestjs/core` in package.json (NestJS)
   - `tsconfig.json` without framework config (plain TypeScript)
   - `Cargo.toml` (Rust)
   - `pyproject.toml` or `requirements.txt` (Python)
   - `go.mod` (Go)
   - `schema.prisma` or migration files (Database)
   - `package.json` (general JS/TS)
2. **Architecture**: Directory structure, key patterns, naming conventions
3. **Relevant files**: Files that will likely need modification for this task: {summarized task}
4. **Existing patterns**: How similar features are implemented in this codebase
5. **Risks**: Potential conflicts, breaking changes, or complex dependencies

Return a structured report with clear sections."

Wait for the researcher to complete. Store the findings.

## Step 3 — Auto-detect and Confirm Agents

Based on the research findings, select the appropriate forja agents. Map detected stack to skill IDs:

| Signal | Skill ID |
|--------|----------|
| `next.config.*` | `code/nextjs/feature` |
| `nest-cli.json` or `@nestjs/core` | `code/nestjs/feature` |
| `tsconfig.json` (no framework) | `code/typescript/feature` |
| `Cargo.toml` | `code/rust/feature` |
| `pyproject.toml` / `requirements.txt` | `code/python/feature` |
| `go.mod` | `code/golang/feature` |
| `schema.prisma` / migrations | `code/database/feature` |
| Multiple or unknown | `code/general/feature` |

**Always include:**
- `research/codebase/explorer` (researcher)
- `research/architecture/planner` (planner)

**Include if task requires:**
- `test/tdd/workflow` (tester) — if new functionality needs tests
- `review/code-quality/reviewer` (reviewer) — if changes are non-trivial
- `review/security/auditor` (security) — if auth, payments, user input handling
- `deploy/git/commit` (deployer) — if user wants commit + PR
- `deploy/verify/pipeline` (verifier) — if CI/CD changes involved

**Show detected agents to the user for confirmation via `AskUserQuestion`:**
- header: "Agents"
- List the detected agents with roles, then ask:
- options: `["Looks good, proceed", "Add more agents", "Remove some agents"]`

If "Add more agents" or "Remove some agents" — follow up with the specific adjustment, then proceed.

## Step 4 — Auto-size Team

Select the team size based on plan complexity:

| Condition | Team Size | Description |
|-----------|-----------|-------------|
| 1-3 files, simple fix, single concern | `quick-fix` | Coder + Deployer |
| 3-10 files, single stack, moderate complexity | `solo-sprint` | Coder-Tester + Reviewer |
| 10+ files, multi-phase, cross-cutting concerns | `full-product` | Researcher + Coder + Tester + Reviewer + Deployer |

## Step 5 — Build Structured Phases

Based on interview answers, research findings, and selected agents, create implementation phases. Each phase must have:

- **name**: Short descriptive name (e.g. "Database schema", "Auth middleware")
- **agent_role**: Which agent handles this phase (e.g. "coder", "tester")
- **files_to_create**: List of new files this phase creates
- **files_to_modify**: List of existing files this phase modifies
- **instructions**: Specific implementation instructions for the agent
- **depends_on**: List of phase names that must complete first

Guidelines:
- Order phases by dependency — foundational work first (schema, types, config), then implementation, then tests, then review
- Keep phases focused — one concern per phase
- Be specific in instructions — reference actual file paths from research findings
- Include a testing phase if quality gates require it
- Include a review phase for non-trivial plans

Also compile **quality gates** from interview answers (or infer defaults). These are conditions that must be satisfied after all phases complete. Examples: "All tests must pass", "No TypeScript errors", "Security audit passes".

## Step 6 — Save Plan

Generate two files in `~/.forja/plans/`:

### Plan ID format
`YYYYMMDD-HHMMSS-slug` where slug is the task in kebab-case (max 40 chars).

Example: `20260208-143022-user-auth-jwt`

### File 1: `{plan-id}.json` (machine-readable)

Write this JSON using the Write tool:

```json
{
  "id": "{plan-id}",
  "created": "{ISO 8601 timestamp}",
  "status": "pending",
  "task": "{original task description}",
  "team_size": "{quick-fix|solo-sprint|full-product}",
  "profile": "balanced",
  "agents": [
    { "skill_id": "{skill_id}", "role": "{role_name}" }
  ],
  "stack": {
    "language": "{detected language}",
    "framework": "{detected framework or null}"
  },
  "quality_gates": [
    "{gate 1}",
    "{gate 2}"
  ],
  "phases": [
    {
      "name": "{phase name}",
      "agent_role": "{role}",
      "files_to_create": ["{path}"],
      "files_to_modify": ["{path}"],
      "instructions": "{detailed instructions}",
      "depends_on": ["{phase name}"]
    }
  ]
}
```

### File 2: `{plan-id}.md` (human-readable)

Write this Markdown using the Write tool:

```markdown
# Plan: {task description}

## Context
{Summary from interview + researcher findings}

## Requirements

### Functional
{Bullet list from interview}

### Technical Constraints
{Bullet list from interview}

### Out of Scope
{What was explicitly excluded}

## Quality Gates
{Checklist of quality gates}

## Implementation Phases

### Phase 1: {phase name}
- **Agent**: {role name}
- **Files to create**: {list}
- **Files to modify**: {list}
- **Depends on**: {list or "None"}
- **Instructions**: {what this phase does}

### Phase 2: {phase name}
...

(Continue for all phases)

## Stack
- Language: {language}
- Framework: {framework}
- Key dependencies: {list}

## Agents
{Table of selected agents with skill_id and role}

## Team Size
{team_size} — {reason for selection}

## Risks
{Bullet list of risks from research}

## Research Findings
{Full researcher report embedded here}
```

### After saving

Display a summary to the user:

```
Plan saved!

  Task:      {task}
  Plan ID:   {plan-id}
  Stack:     {language} / {framework}
  Team:      {team_size} ({N} agents)
  Agents:    {comma-separated role names}
  Phases:    {N} phases
  Gates:     {N} quality gates

  Plan:  ~/.forja/plans/{plan-id}.md
  Meta:  ~/.forja/plans/{plan-id}.json

Next: Run `forja execute` in your terminal to start execution.
```
