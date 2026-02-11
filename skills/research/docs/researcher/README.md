# Doc Researcher

> Phase: **Research** | Tech: **docs**

Research external documentation and APIs using web search and context fetching. Produces structured summaries with code examples.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| doc-researcher | Read, WebSearch, WebFetch | sonnet |

## What it does

The doc-researcher agent finds, reads, and summarizes external documentation and APIs. It searches for official documentation first, then community resources, extracts key APIs, setup instructions, working examples, and flags gotchas like breaking changes or common mistakes. The output is a structured research report with sources.

## Usage

After installing with `forja install research/docs/researcher`:

```bash
# Use the doc-researcher agent to research libraries or APIs
doc-researcher
```

Ask it to research any library, API, framework, or concept. It will produce a report with:
- Summary and overview
- Key APIs and concepts
- Setup and installation steps
- Code examples from the docs
- Gotchas, known issues, and breaking changes
- Sources with URLs

## Install

```bash
forja install research/docs/researcher
```
