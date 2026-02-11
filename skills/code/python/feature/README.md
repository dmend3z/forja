# Python Coder

> Phase: **Code** | Tech: **python**

Python specialist for FastAPI/Django with type hints, pydantic models, and clean architecture.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| py-coder | Read, Write, Edit, Bash, Glob, Grep | opus |

## What it does

The py-coder agent writes typed, well-structured Python following project conventions. It detects the framework (FastAPI, Django, Flask) from project structure, uses type hints on all functions, leverages Pydantic for data models, and follows framework-specific patterns like FastAPI routers with dependency injection or Django's MVT structure.

## Usage

After installing with `forja install code/python/feature`:

```bash
# Use the py-coder agent for Python implementation
py-coder
```

The agent follows these standards:
- Type hints on all functions (parameters and return types)
- Pydantic BaseModel for DTOs and validation
- Dataclasses for internal models
- f-strings for formatting
- pathlib.Path instead of os.path

## Install

```bash
forja install code/python/feature
```
