# Agent Authoring Guide

How to create a new forja agent from scratch.

## How Agents Work

A forja agent is a directory containing a `skill.json` manifest and one or more content files (agents, skills, commands). When you run `forja install <skill-id>`, forja symlinks the `.md` files from `agents/` and `commands/` into `~/.claude/agents/` and `~/.claude/commands/`, making them available to Claude Code.

Symlinks are prefixed with `forja--` to avoid name collisions. For example, installing `deploy/git/commit` creates:

```
~/.claude/agents/forja--deploy--git--commit--committer.md
```

## Directory Structure

Every skill lives under `skills/` following a three-level hierarchy:

```
skills/<phase>/<tech>/<name>/
```

- **phase** -- one of: `research`, `code`, `test`, `review`, `deploy`, `teams`
- **tech** -- the technology or domain category (e.g. `typescript`, `tdd`, `security`, `git`)
- **name** -- the skill name in kebab-case (e.g. `feature`, `workflow`, `auditor`)

The catalog currently contains 31 skills across 6 phases.

The skill ID is derived from this path: `<phase>/<tech>/<name>`. For example:

| Path | Skill ID |
|------|----------|
| `skills/code/rust/feature/` | `code/rust/feature` |
| `skills/test/tdd/workflow/` | `test/tdd/workflow` |
| `skills/deploy/git/commit/` | `deploy/git/commit` |
| `skills/teams/solo-sprint/team/` | `teams/solo-sprint/team` |

## Required: skill.json

Every skill should have a `skill.json` file at the root of the skill directory. This is the only required file -- without it (or the legacy fallback), forja will not detect the skill.

```
<name>/
  skill.json     # REQUIRED (preferred)
```

### Schema

```json
{
  "name": "skill-display-name",
  "description": "One-sentence description of what the skill does.",
  "version": "1.0.0",
  "author": { "name": "forja" },
  "keywords": ["phase", "tech", "relevant-term"]
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Display name shown in `forja list` and `forja info` |
| `description` | yes | One-sentence summary. Shown in listings and search results |
| `version` | yes | Semver string (e.g. "1.0.0") |
| `author` | yes | Object with `name` (string) |
| `keywords` | yes | Array of strings for categorization and search |

Real example from `deploy/git/commit`:

```json
{
  "name": "git-commit",
  "description": "Conventional commits with type, scope, and descriptive messages. Analyzes staged changes and creates well-formatted commits.",
  "version": "1.0.0",
  "author": { "name": "forja" },
  "keywords": ["deploy", "git", "commit"]
}
```

## Optional: agents/*.md

Agent files define Claude Code agent personas. Each `.md` file in the `agents/` directory gets symlinked into `~/.claude/agents/` on install.

```
<name>/
  agents/
    my-agent.md
```

### Frontmatter Format

Agent files use YAML frontmatter with four fields:

```yaml
---
name: agent-name
description: One-sentence description of what this agent does.
tools: Tool1, Tool2, Tool3
model: opus
---
```

| Field | Description |
|-------|-------------|
| `name` | Agent name as it appears in Claude Code's agent picker |
| `description` | Short description shown when selecting the agent |
| `tools` | Comma-separated list of tools the agent can use |
| `model` | Which model to use: `opus` for complex tasks, `sonnet` for simpler ones |

**Available tools**: `Read`, `Write`, `Edit`, `Bash`, `Glob`, `Grep`, `LSP`, `Task`

**Model guidance**:
- Use `opus` for agents that need deep reasoning (coding, research, review, testing)
- Use `sonnet` for agents that execute straightforward workflows (commits, PRs, deploys)

**Important**: The `model` field in frontmatter is metadata only. When spawning agents programmatically with the `Task` tool, you MUST pass the `model` parameter explicitly to enforce the model choice.

### Body Content

After the frontmatter, write the agent's system prompt in Markdown. Structure it as:

1. **Role statement** -- one sentence defining who the agent is
2. **Workflow** -- numbered steps the agent should follow
3. **Rules** -- constraints and guardrails

Real example from `deploy/git/commit/agents/committer.md`:

```markdown
---
name: committer
description: Creates conventional commits by analyzing staged changes. Determines commit type, scope, and writes descriptive messages.
tools: Bash
model: sonnet
---

You create git commits following Conventional Commits format.

## Workflow

1. Run `git status` and `git diff --staged` to understand changes
2. Run `git log --oneline -5` to match the repository's style
3. Determine the commit type and scope from the changes
4. Write a concise, descriptive commit message
5. Create the commit

## Commit Format

type(scope): subject (max 50 chars)

Body explaining HOW and WHY (wrap at 72 chars).

## Rules

- ALWAYS include scope in parentheses
- Present tense imperative verb: add, implement, fix, remove
- NO period at end of subject
- NEVER commit .env, credentials, or secrets
- Stage specific files, avoid `git add -A` unless instructed
```

## Optional: skills/*/SKILL.md

Skill prompts are reusable instruction sets that get loaded as context. They live in a nested `skills/<skill-name>/SKILL.md` directory inside the agent package.

```
<name>/
  skills/
    my-skill/
      SKILL.md
```

SKILL.md files use the same YAML frontmatter as agents, but with only `name` and `description`:

```yaml
---
name: skill-name
description: What this skill prompt provides
---

Markdown content with instructions, templates, or reference material.
```

Real example from `deploy/git/commit/skills/commit/SKILL.md`:

```markdown
---
name: commit
description: Create a conventional commit from staged changes
---

Analyze all staged changes and create a well-formatted conventional commit.

1. Run `git status` to see what's staged
2. Run `git diff --staged` to understand the changes
3. Run `git log --oneline -5` to match the repo's commit style
4. Determine: type (feat/fix/refactor/etc), scope, and subject
5. Create the commit with a descriptive message and Co-Authored-By trailer
6. Verify with `git log -1`
```

## Optional: commands/*.md

Slash commands let users invoke agents from Claude Code with `/command-name`. Each `.md` file in `commands/` becomes a slash command.

```
<name>/
  commands/
    my-command.md
```

Command frontmatter:

```yaml
---
description: Short description shown in the command picker
argument-hint: What the user should pass as an argument
---
```

Commands are symlinked into `~/.claude/commands/` on install.

Note: commands from `teams`-phase skills are managed separately. Installing a team skill with `forja install teams/<tech>/<name>` does not symlink its commands -- use `forja team add <team-name>` instead.

## Complete Skill Layout

Here is every possible file in a skill package:

```
skills/<phase>/<tech>/<name>/
  skill.json              # REQUIRED -- skill manifest
  agents/
    agent-name.md         # Agent persona (symlinked to ~/.claude/agents/)
  skills/
    skill-name/
      SKILL.md            # Skill prompt (loaded as context)
  commands/
    command-name.md       # Slash command (symlinked to ~/.claude/commands/)
```

Most skills only use one or two of these content types. A typical skill has `skill.json` + one agent file.

## Naming Conventions

- **Directories**: kebab-case (`code-quality`, `tdd-workflow`, `solo-sprint`)
- **Agent files**: kebab-case matching the agent name (`researcher.md`, `lint-fixer.md`)
- **Skill IDs**: derived from path, slash-separated (`code/typescript/feature`, `review/security/auditor`)
- **skill.json name**: kebab-case display name (`git-commit`, `code-reviewer`, `tdd-workflow`)
- **Agent names in frontmatter**: kebab-case (`committer`, `researcher`, `security-auditor`)

## Concrete Example: Creating a Linting Skill

Let's walk through creating a new skill that runs linting and auto-fixes issues.

### 1. Create the directory structure

```
skills/review/linting/fixer/
  skill.json
  agents/
    lint-fixer.md
```

### 2. Write skill.json

```json
{
  "name": "lint-fixer",
  "description": "Runs project linter, analyzes violations, and applies auto-fixes following project config.",
  "version": "1.0.0",
  "author": { "name": "forja" },
  "keywords": ["review", "linting", "code-quality", "auto-fix"]
}
```

### 3. Write the agent file

`agents/lint-fixer.md`:

```markdown
---
name: lint-fixer
description: Runs the project linter and applies auto-fixes. Respects existing lint config.
tools: Read, Bash, Glob, Grep
model: sonnet
---

You are a linting specialist. You find and fix lint violations.

## Workflow

1. Detect the linter from config files (eslint, ruff, clippy, golangci-lint)
2. Run the linter and capture output
3. Categorize violations by severity
4. Apply auto-fixes where safe
5. Re-run linter to verify fixes
6. Report remaining issues that need manual attention

## Rules

- NEVER change lint config without asking
- Respect existing .eslintrc / ruff.toml / clippy.toml settings
- Auto-fix only when the fix is safe and obvious
- Report issues you cannot auto-fix with file path and line number
```

### 4. Test it locally

From the forja-skills repo root:

```bash
# Initialize forja in local dev mode (creates symlink + installs all skills)
cargo run -- init

# If you created this skill after running init, install it explicitly
cargo run -- install review/linting/fixer

# Verify everything is healthy
cargo run -- doctor
```

`forja doctor` checks that all symlinks point to valid files. If you see `PASS` for symlinks, your agent is installed correctly.

### 5. Verify in Claude Code

Open Claude Code and check the agent picker -- your `lint-fixer` agent should appear. You can also check the symlink directly:

```bash
ls -la ~/.claude/agents/ | grep forja--review--linting--fixer
```

You should see:

```
forja--review--linting--fixer--lint-fixer.md -> /Users/you/mndz-skills/skills/review/linting/fixer/agents/lint-fixer.md
```

## Local Development Workflow

When you run `forja init` from a directory that contains a `skills/` folder, forja creates a symlink from `~/.forja/registry` to your local repo instead of cloning from GitHub. This means:

- Changes to agent files take effect immediately (symlinks point to your working copy)
- No need to push, pull, or re-install after editing
- `forja doctor` validates that all symlinks are still healthy

Typical cycle:

```bash
# One-time setup
cargo run -- init                    # Detects local skills/, creates symlink, installs all skills

# Edit-test loop
vim skills/.../agents/my-agent.md    # Edit the agent
# Open Claude Code -- changes are live immediately

# Adding a new skill after init
cargo run -- install <skill-id>      # Install a skill created after initial setup

# Verify
cargo run -- doctor                  # Check symlink health
```

## Checklist

Before submitting a new skill:

- [ ] Directory follows `skills/<phase>/<tech>/<name>/` structure
- [ ] `skill.json` exists with all required fields (`name`, `description`, `version`, `author`, `keywords`)
- [ ] Agent `.md` files have valid frontmatter (`name`, `description`, `tools`, `model`)
- [ ] Agent prompt includes a clear role statement, workflow, and rules
- [ ] `cargo run -- install <skill-id>` succeeds without errors
- [ ] `cargo run -- doctor` shows all symlinks healthy
- [ ] Agent appears in Claude Code's agent picker
- [ ] Tested the agent end-to-end in a real scenario
