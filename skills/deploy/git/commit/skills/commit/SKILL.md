---
name: commit
description: Create a conventional commit from staged changes
---

Analyze all staged changes and create a well-formatted conventional commit.

1. Run `git status` to see what's staged
2. Run `git diff --staged` to understand the changes
3. Run `git log --oneline -5` to match the repo's commit style
4. Determine: type (feat/fix/refactor/etc), scope, and subject
5. Create the commit with a descriptive message and Co-Authored-By trailer
6. Verify with `git log -1`
