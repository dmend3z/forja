# Database Coder

> Phase: **Code** | Tech: **database**

Database specialist for schemas, migrations, and queries. Supports Prisma, Drizzle, and raw SQL.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| db-coder | Read, Write, Edit, Bash, Glob, Grep | opus |

## What it does

The db-coder agent creates database schemas, migrations, and queries. It detects your ORM or query builder (Prisma, Drizzle, raw SQL), follows existing schema patterns, creates proper indexes, enforces foreign key constraints, and writes migrations that can be safely rolled back. The agent understands data modeling best practices.

## Usage

After installing with `forja install code/database/feature`:

```bash
# Use the db-coder agent for database implementation
db-coder
```

The agent follows these standards:
- Schema design with proper constraints and indexes
- Reversible migrations
- Query optimization (avoid N+1, use joins)
- ORM-specific patterns (Prisma schema, Drizzle queries)
- Data integrity enforcement

## Install

```bash
forja install code/database/feature
```
