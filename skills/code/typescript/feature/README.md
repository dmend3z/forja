# TypeScript Coder

> Phase: **Code** | Tech: **typescript**

TypeScript specialist with strict types, no any, proper patterns. Follows existing conventions and prefers boring solutions.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| ts-coder | Read, Write, Edit, Bash, Glob, Grep, LSP | opus |

## What it does

The ts-coder agent writes strict, type-safe TypeScript code following project conventions. It reads CLAUDE.md, checks tsconfig.json for compiler options, studies existing files, and reuses existing types and utilities. It enforces strict mode (no `any`), uses named exports, defines interfaces before implementation, and prefers discriminated unions over optional fields.

## Usage

After installing with `forja install code/typescript/feature`:

```bash
# Use the ts-coder agent for TypeScript implementation
ts-coder
```

The agent follows these standards:
- Strict TypeScript with no `any` or unsafe casts
- Named exports (not default exports)
- Interface-first design
- Zod for runtime validation at boundaries
- Type narrowing with guards instead of assertions

## Install

```bash
forja install code/typescript/feature
```
