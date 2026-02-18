---
description: Generate spec files from a meeting transcript — extracts features, groups into structured specs with YAML frontmatter
argument-hint: Path to transcript file or "paste" to read from stdin
---

# generate-specs

You are a spec generation specialist. You take meeting transcripts and produce structured, actionable developer spec files.

**Input:** $ARGUMENTS

> **Fallback**: If `AskUserQuestion` is unavailable, present numbered options in plain text and wait for the user to type a number.

---

## Step 1 — Load Transcript

Determine the input source:

- If `$ARGUMENTS` is a file path → Read the file
- If `$ARGUMENTS` is "paste" or empty → Ask the user to paste the transcript content
- If `$ARGUMENTS` contains the transcript inline → Use it directly

Once you have the transcript, confirm: "Loaded transcript ({N} words). Analyzing..."

---

## Step 2 — Project Context

Use `AskUserQuestion` to ask these questions in a **single batch**:

| # | header | question | notes |
|---|--------|----------|-------|
| Q1 | Project | Which project are these specs for? | options: infer from current directory name, or ask. The specs will be saved to `{project}/docs/specs/` |
| Q2 | Owners | Who are the default owners? | Free text — names of people responsible |
| Q3 | Prefix | Spec ID prefix/numbering | options: `["Auto-number from existing specs (Recommended)", "Start from SPEC-001", "Custom prefix"]` |
| Q4 | Depth | How detailed should specs be? | options: `["Concise — goals + requirements only", "Standard — full sections (Recommended)", "Thorough — include technical notes, open questions, success metrics"]` |

---

## Step 3 — Extract & Group

Analyze the transcript and extract:

1. **Action items** — anything that needs to be built, changed, fixed, or investigated
2. **Non-dev items** — operational/PM tasks (flag separately, don't generate specs for these)
3. **Group related items** into logical specs — each spec should be a cohesive feature or workstream

For each proposed spec, determine:
- **id**: kebab-case slug (e.g., `spec-001-video-thumbnail-preview`)
- **title**: clear, descriptive title
- **priority**: high (ship blocker / cost impact / user-facing bugs), medium (important but not blocking), low (nice-to-have)
- **description**: 1-sentence summary of what and why
- **tags**: 2-4 relevant tags
- **requirements**: must-have deliverables (from transcript discussion)
- **constraints**: explicit non-goals or out-of-scope items mentioned
- **success_criteria**: measurable outcomes mentioned or implied

Present the proposed spec list to the user:

```
Found {N} action items → grouped into {M} specs:

1. [HIGH] spec-001-xxx — Title (items #1, #4, #5)
2. [MED]  spec-002-yyy — Title (items #3, #2)
3. [LOW]  spec-003-zzz — Title (item #20)

Non-dev items (not converted to specs):
- Item description → Owner
```

Use `AskUserQuestion`:
- "Does this grouping look correct?"
- Options: `["Looks good, generate specs (Recommended)", "Merge some specs together", "Split a spec into multiple", "Remove some specs"]`

Adjust based on feedback. Repeat until approved.

---

## Step 4 — Generate Spec Files

For each approved spec, generate a markdown file with this exact structure:

```markdown
---
id: {id}
title: "{title}"
description: "{description}"
priority: {high|medium|low}
tags:
  - {tag1}
  - {tag2}
requirements:
  - {requirement1}
  - {requirement2}
constraints:
  - {constraint1}
success_criteria:
  - "{criteria1}"
---

# SPEC-{NNN}: {Title}

> **Meeting items:** {item references}
> **Owner:** {owners}
> **Priority:** {P0|P1|P2} ({reason})

---

## 1. Problem Statement

{Extracted from transcript — what's broken, what's missing, why it matters}

## 2. Goals

- **{Goal 1}** — {brief explanation}
- **{Goal 2}** — {brief explanation}

## 3. Non-Goals

- {Explicit exclusion 1}
- {Explicit exclusion 2}

## 4. User Stories

- As a {role}, I want {action} so that {benefit}

## 5. Requirements

### Must-Have (P0)

**5.1 {Requirement Group}**
- [ ] {Specific deliverable}
- [ ] {Acceptance criteria}

### Nice-to-Have (P1)

**5.N {Optional requirement}**
- [ ] {Deliverable}

## 6. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| {metric} | {target} | {how to measure} |

## 7. Open Questions

- [ ] **{Owner}:** {Question that needs answering}

## 8. Technical Notes

{Implementation hints, code references, architecture notes from transcript}
```

**Rules:**
- If depth is "Concise" → only include sections 1-3 and 5 (Problem, Goals, Non-Goals, Requirements)
- If depth is "Standard" → include all sections except 8 (Technical Notes)
- If depth is "Thorough" → include all sections
- Every requirement must have a checkbox `- [ ]` for tracking
- Every requirement group must have an acceptance criteria line
- Success metrics must be measurable (numbers, not vague)
- Open questions must tag an owner

---

## Step 5 — Save & Generate Index

1. Create `docs/specs/` directory if it doesn't exist
2. Write each spec file: `docs/specs/SPEC-{NNN}-{slug}.md`
3. Generate or update `docs/specs/SPEC-000-index.md` with a summary table:

```markdown
# Developer Specs — {Project Name}

> Generated from meeting transcript ({date})

## Specs Overview

| Spec | Title | Priority | Owner | Tags |
|------|-------|----------|-------|------|
| [SPEC-001](./SPEC-001-slug.md) | Title | P0 | Owner | tag1, tag2 |

## Priority Guide

- **P0 (Ship Blocker):** SPEC-XXX — reason
- **P1 (Important):** SPEC-XXX — reason
- **P2 (Nice-to-Have):** SPEC-XXX — reason

## Suggested Implementation Order

1. **SPEC-XXX** — reason for ordering
```

4. Display summary:

```
Specs generated!

  Project:    {project name}
  Location:   docs/specs/
  Specs:      {N} specs ({X} high, {Y} medium, {Z} low)

  Files created:
    - docs/specs/SPEC-000-index.md
    - docs/specs/SPEC-001-slug.md
    - ...

Next: Open the Forja Desktop app to browse specs, or run a spec as a spark.
```
