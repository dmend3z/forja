# Performance Analyzer

> Phase: **Review** | Tech: **performance**

Performance review covering algorithmic complexity, N+1 queries, unnecessary re-renders, and bundle size.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| perf-analyzer | Read, Grep, Glob, Bash, LSP | opus |

## What it does

The perf-analyzer agent finds performance bottlenecks before they hit production. It analyzes algorithmic complexity, identifies N+1 queries and missing indexes, detects unnecessary React re-renders, checks bundle size impact, and reviews for missing pagination or caching. Findings include measured or estimated performance impact.

## Usage

After installing with `forja install review/performance/analyzer`:

```bash
# Use the perf-analyzer agent for performance review
perf-analyzer
```

The agent checks for:
- N+1 database queries
- Missing indexes on filtered/sorted columns
- Unbounded queries without LIMIT
- Unnecessary React re-renders
- Missing memoization
- Bundle size bloat

## Install

```bash
forja install review/performance/analyzer
```
