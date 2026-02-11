# Git Commit

> Phase: **Deploy** | Tech: **git**

Conventional commits with type, scope, and descriptive messages. Analyzes staged changes and creates well-formatted commits.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| committer | Bash | sonnet |

## What it does

The committer agent creates conventional commits by analyzing staged changes. It runs git status and git diff to understand changes, checks recent commit history to match repository style, determines the commit type and scope, writes concise descriptive messages, and uses HEREDOC for multi-line messages. It NEVER commits secrets or uses git add -A without instruction.

## Usage

After installing with `forja install deploy/git/commit`:

```bash
# Use the committer agent to create commits
committer
```

The agent follows Conventional Commits format:
```
type(scope): subject (max 50 chars)

Body explaining HOW and WHY (wrap at 72 chars).

Co-Authored-By: Claude <noreply@anthropic.com>
```

Commit types: feat, fix, refactor, test, docs, chore, perf, ci

## Install

```bash
forja install deploy/git/commit
```
