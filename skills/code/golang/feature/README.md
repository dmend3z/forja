# Go Coder

> Phase: **Code** | Tech: **golang**

Go specialist with standard project layout, interface-first design, and proper error wrapping.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| go-coder | Read, Write, Edit, Bash, Glob, Grep | opus |

## What it does

The go-coder agent writes idiomatic Go code following standard project layout and conventions. It uses interface-first design, wraps errors with fmt.Errorf for context, accepts interfaces and returns structs, and follows naming conventions like MustX for panic-on-error functions. Error handling is explicit with proper wrapping chains.

## Usage

After installing with `forja install code/golang/feature`:

```bash
# Use the go-coder agent for Go implementation
go-coder
```

The agent follows these standards:
- Standard Go project layout (cmd, internal, pkg)
- Interface-first design (accept interfaces, return structs)
- Error wrapping with fmt.Errorf
- Table-driven tests
- Explicit error handling (no silent failures)

## Install

```bash
forja install code/golang/feature
```
