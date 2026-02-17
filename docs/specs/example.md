---
id: example-feature
title: Add Example Feature
description: A sample spec demonstrating the expected format for forja sparks
priority: medium
tags:
  - example
  - documentation
requirements:
  - Parse spec files from docs/specs/
  - Display spec details in the CLI
  - Generate execution plans from specs
constraints:
  - Must work without network access for local specs
  - No new async runtime dependencies
success_criteria:
  - forja sparks list shows all discovered specs
  - forja sparks show <id> displays full spec details
  - forja sparks plan <id> generates an execution plan
---

# Example Feature

This is a sample spec file that demonstrates the expected format for `forja sparks`.

## Background

Spec files live in `docs/specs/` and use YAML frontmatter to define structured metadata.
The markdown body below the frontmatter provides free-form context, design notes,
and implementation details that get injected into the AI planning prompt.

## Design Notes

- Specs are the input to the planning pipeline
- Each spec maps to exactly one execution plan
- The `id` field must be unique across all specs in the directory

## Acceptance Criteria

The success criteria in the frontmatter are checked after execution completes.
They should be concrete and verifiable.
