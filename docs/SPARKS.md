# Sparks: Spec-Driven Execution

Sparks is a pipeline that turns spec files into AI-driven implementation plans and executes them with agent teams. The workflow: write a spec, generate a plan, execute it phase by phase with quality gates.

## Quick Start

```bash
# List all specs in docs/specs/
forja sparks list

# Show a spec's details
forja sparks show user-auth

# Generate an execution plan from a spec
forja sparks plan user-auth

# Execute the plan (with retry-then-ask on failure)
forja sparks execute user-auth

# Check execution progress
forja sparks status user-auth
```

## Spec File Format

Spec files live in `docs/specs/` as markdown files with YAML frontmatter. The frontmatter defines structured metadata; the markdown body provides free-form context.

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Unique identifier (kebab-case recommended) |
| `title` | String | Human-readable title |
| `description` | String | One-line summary |

### Optional Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `priority` | String | none | Priority level (e.g., `high`, `medium`, `low`) |
| `tags` | List | `[]` | Categorization tags |
| `requirements` | List | `[]` | What the implementation must do |
| `constraints` | List | `[]` | Restrictions on how it's implemented |
| `success_criteria` | List | `[]` | Verifiable conditions for completion |

### Example Spec

```markdown
---
id: user-auth
title: Add User Authentication
description: Implement JWT-based authentication
priority: high
tags:
  - auth
  - security
requirements:
  - JWT token generation
  - Login endpoint
constraints:
  - Must use existing user table
success_criteria:
  - Users can log in and receive a token
  - Protected routes reject unauthenticated requests
---

# User Authentication

This spec describes the authentication system.

## Details

Use bcrypt for password hashing.
```

## Commands

### `forja sparks list [--path <dir>]`

Discover and list all specs. Default directory: `docs/specs/` relative to the project root.

```
Specs

  ID          Title                   Priority  Status
  search-api  Search API              medium    draft
  user-auth   User Authentication     high      draft
```

### `forja sparks show <spec-id>`

Display full details for a spec: metadata, requirements, constraints, success criteria, and the markdown body.

### `forja sparks plan <spec-id>`

Generate an execution plan from a spec. This:

1. Loads the spec and builds a structured task description from its fields
2. Loads the `forja-plan` skill template
3. Replaces the `$ARGUMENTS` placeholder with the task description
4. Launches a Claude Code session to generate the plan
5. The plan JSON is saved in `~/.forja/plans/` with a `source_spec` field linking it back

### `forja sparks execute <spec-id> [--profile <profile>] [--resume]`

Execute the linked plan. The pipeline:

1. Loads the spec and finds its linked plan (via `source_spec` in plan JSON)
2. Auto-installs any missing agents referenced in the plan
3. Enables the agent teams environment variable if needed
4. For phased plans, executes each phase sequentially with:
   - **Retry-then-ask**: first failure retries automatically; second failure prompts the user with Retry / Skip / Abort
   - **Quality gates**: `cargo test --workspace` and `cargo clippy --workspace` after each phase (non-blocking warnings)
   - **Checkpointing**: progress saved after every state transition; use `--resume` to continue after interruption
5. For monolithic plans (no phases), runs a single Claude session

**Profiles**: `fast` (all sonnet), `balanced` (default — opus for thinking, sonnet for coding), `max` (all opus).

### `forja sparks status [<spec-id>]`

Without a spec ID, shows a summary table of all specs with derived status:

| Status | Meaning |
|--------|---------|
| `draft` | No linked plan |
| `ready` | Plan exists, not started |
| `executing` | Phase(s) in progress |
| `N/M phases` | Partial completion |
| `failed` | One or more phases failed |
| `complete` | All phases done |

With a spec ID, shows detailed phase-by-phase progress with colored indicators and quality gate status.

## Workflow

```
1. Write spec     docs/specs/my-feature.md
       |
2. Plan           forja sparks plan my-feature
       |
3. Review plan    forja sparks status my-feature
       |
4. Execute        forja sparks execute my-feature
       |
5. Monitor        forja sparks status my-feature
```

## Spec-Plan Linkage

Plans are linked to specs via the `source_spec` field in the plan JSON (`~/.forja/plans/*.json`). When `forja sparks plan` generates a plan, it instructs Claude to include this field. The `find_plan_for_spec()` function scans the plans directory and returns the most recent plan matching the spec ID.

## Error Handling

- **Missing spec**: error with hint to run `forja sparks list`
- **Missing plan**: error with hint to run `forja sparks plan <id>`
- **Phase failure**: retry once automatically, then prompt user (Retry / Skip / Abort)
- **Resume after crash**: `forja sparks execute <id> --resume` picks up from the last checkpoint
