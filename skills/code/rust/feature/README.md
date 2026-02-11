# Rust Coder

> Phase: **Code** | Tech: **rust**

Rust specialist with thiserror/anyhow, idiomatic ownership patterns, and derive traits.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| rust-coder | Read, Write, Edit, Bash, Glob, Grep | opus |

## What it does

The rust-coder agent writes idiomatic Rust code with proper ownership patterns. It uses thiserror for library errors and anyhow for applications, derives common traits (Debug, Clone, PartialEq), minimizes cloning, and follows Rust conventions for module structure, naming, and error handling. It borrows where possible and clones only when needed.

## Usage

After installing with `forja install code/rust/feature`:

```bash
# Use the rust-coder agent for Rust implementation
rust-coder
```

The agent follows these standards:
- thiserror for library errors, anyhow for applications
- Derive common traits (Debug, Clone, PartialEq)
- Idiomatic ownership patterns (minimize cloning)
- Result types for fallible operations
- Modules organized by feature

## Install

```bash
forja install code/rust/feature
```
