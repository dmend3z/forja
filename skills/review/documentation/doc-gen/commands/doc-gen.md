---
description: Generate or update project documentation (CLAUDE.md, AGENTS.md, README.md) by analyzing the codebase
argument-hint: "Optional scope: claude-md, agents-md, readme (default: all)"
---

# doc-gen

You are a documentation generator. Analyze the codebase and generate accurate, grounded documentation.

Scope: **$ARGUMENTS** (if empty, generate all documents)

---

## Step 1 — Read the Codebase

Before writing anything, gather facts:

1. **Detect project type** from config files:
   - `Cargo.toml` → Rust | `package.json` → Node.js/TypeScript | `pyproject.toml` → Python | `go.mod` → Go
   - Note frameworks (Next.js, NestJS, FastAPI, etc.) from dependencies

2. **Read existing docs** — if CLAUDE.md, AGENTS.md, or README.md exist, read them fully. Note custom sections.

3. **Map structure** — use Glob for top-level dirs, entry points, test dirs, config files.

4. **Scan agents** — check `.claude/agents/*.md`, extract frontmatter (name, description, tools, model).

5. **Extract commands** — from `package.json` scripts, `Makefile` targets, or `Cargo.toml`.

6. **Decide per file**: create / update (surgical) / skip.

---

## Step 2 — Generate Documentation

Based on the scope argument:

### If scope is `claude-md` or empty:

Generate/update **CLAUDE.md** with: Stack, Commands, Architecture, Conventions, and Rules.

The Rules section must always include:
- Think before coding — read the codebase and plan before writing
- Simplicity first — if it can be 50 lines, don't write 200
- Surgical changes — touch only what the task requires
- Goal-driven — define done before starting; verify each output
- Surface assumptions before non-trivial changes
- Stop and ask when ambiguous — don't guess
- Show the complete error before attempting to fix

### If scope is `agents-md` or empty:

Generate/update **AGENTS.md** — but only if `.claude/agents/` has files. Skip entirely otherwise.

Format: table with Agent, Description, Tools, Model columns.

### If scope is `readme` or empty:

Generate/update **README.md** with: project name, description, requirements, setup, usage, development.

### Update behavior:
- Only rewrite stale sections — preserve custom content
- Don't reorder existing sections
- Use Edit tool for surgical updates, not Write for full replacement
- Leave `TODO` for anything you couldn't determine — never guess

---

## Step 3 — Self-Review

1. Reread each file you wrote or modified
2. Verify every claim is grounded in something you actually read
3. Run `git diff` to confirm delta matches expectations
4. Output summary:

```
| File       | Action  | Sections Changed          |
|------------|---------|---------------------------|
```
