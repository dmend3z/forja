# Project Name

## Stack
- Framework:
- Language:
- Database:
- Hosting:

## Commands
```
npm run dev       # Start dev server
npm run build     # Production build
npm run test      # Run tests
npm run lint      # Lint code
```

## Architecture
```
src/
├── app/          # Pages and routes
├── components/   # UI components
├── lib/          # Shared utilities
├── services/     # Business logic
└── types/        # Type definitions
```

## Conventions
- Named exports (no default exports unless framework requires it)
- One component per file
- Colocate tests next to source files
- Error handling at system boundaries only

## Development Workflow (forja 5-Phase)

### 1. RESEARCH before coding
- Explore the codebase to understand existing patterns
- Research external docs for unfamiliar libraries
- Plan implementation for complex features
- Never start coding without understanding the context

### 2. CODE following project patterns
- Read this CLAUDE.md first
- Follow existing patterns — match naming, structure, and style
- Prefer boring, obvious solutions over clever abstractions
- Small, focused changes — one concern per function
- Reuse existing utilities before creating new ones
- No premature generalization

### 3. TEST with TDD
- Write tests FIRST for new features (Red-Green-Refactor)
- Generate tests for existing code that lacks coverage
- Target 80%+ test coverage
- Test behavior, not implementation details

### 4. REVIEW before deploying
- Run code review in a fresh context (not the same session that wrote the code)
- Check for security issues (OWASP Top 10)
- Check for performance issues (N+1, complexity, bundle size)
- Every finding needs a specific fix example

### 5. DEPLOY via git
- Conventional commits: type(scope): subject
- Create PRs with structured descriptions
- Verify CI passes before merge

### Teams
Use agent teams for complex tasks:
- `full-product` — 5 agents: researcher, coder, tester, reviewer, deployer
- `solo-sprint` — 2 agents: coder-tester + reviewer (medium features)
- `quick-fix` — 2 agents: coder + deployer (hotfixes)
- `refactor` — 3 agents: analyzer + refactorer + behavioral reviewer (structural changes)

Requires: `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` in settings.json

#### Model Enforcement
When spawning agents with the Task tool, ALWAYS pass the `model` parameter explicitly.
The `model:` field in agent YAML frontmatter is metadata only — NOT enforced at runtime.
If `model` is omitted, the child inherits the parent's model (usually Opus). Treat omission as a bug.

## Rules
- Surface assumptions before implementing non-trivial changes
- Stop and ask when requirements are ambiguous — don't guess
- Push back on bad approaches with concrete reasons
- When stuck, say what you tried and ask
- Touch only what you're asked to touch
- Don't refactor adjacent code as a side effect
- Don't add or remove comments outside the task scope
- Always show the complete error before attempting to fix it
- Commits in English, descriptive and clear
