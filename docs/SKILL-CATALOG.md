# Forja Skill Catalog

Browse all 31 skills in the forja ecosystem. Each skill includes specialized agents, commands, or team configurations designed for specific phases of the development workflow.

## Quick Reference

All 31 skills at a glance:

| Skill ID | Phase | Tech | Description |
|----------|-------|------|-------------|
| research/codebase/explorer | Research | codebase | Deep codebase exploration before writing code |
| research/docs/researcher | Research | docs | Research external documentation and APIs |
| research/architecture/planner | Research | architecture | Create implementation plans with phases and file lists |
| research/planning/forja-plan | Research | planning | Planning pipeline with interview, research, and auto-detect |
| code/typescript/feature | Code | typescript | TypeScript specialist with strict types |
| code/python/feature | Code | python | Python specialist for FastAPI/Django |
| code/golang/feature | Code | golang | Go specialist with standard project layout |
| code/rust/feature | Code | rust | Rust specialist with thiserror/anyhow |
| code/nextjs/feature | Code | nextjs | Next.js specialist with App Router |
| code/nestjs/feature | Code | nestjs | NestJS specialist with modules and services |
| code/database/feature | Code | database | Database specialist for schemas and migrations |
| code/general/feature | Code | general | General-purpose coding agent with auto-detect |
| test/tdd/workflow | Test | tdd | TDD Red-Green-Refactor cycle |
| test/generate/suite | Test | generate | Generate unit and integration tests for existing code |
| test/e2e/playwright | Test | e2e | Playwright E2E tests with Page Object Model |
| test/coverage/analyzer | Test | coverage | Analyze coverage gaps and generate targeted tests |
| review/code-quality/reviewer | Review | code-quality | Fresh-context AI code review |
| review/security/auditor | Review | security | Security audit covering OWASP Top 10 |
| review/performance/analyzer | Review | performance | Performance review for bottlenecks |
| review/pr-workflow/reviewer | Review | pr-workflow | Full PR review lifecycle |
| review/code-simplifier/simplifier | Review | code-simplifier | Simplify code for clarity and maintainability |
| deploy/git/commit | Deploy | git | Conventional commits with git |
| deploy/git/pr | Deploy | git | Push branches and create Pull Requests |
| deploy/verify/checker | Deploy | verify | Post-deploy verification and health checks |
| teams/full-product/team | Teams | - | 5-agent product development team |
| teams/solo-sprint/team | Teams | - | 2-agent lightweight team for medium features |
| teams/quick-fix/team | Teams | - | 2-agent hotfix team for rapid deployment |
| teams/refactor/team | Teams | - | 3-agent refactoring team |
| teams/dispatch/team | Teams | - | Parallel task dispatcher |
| teams/technical-council/team | Teams | - | Council of 5 technical perspectives |
| teams/strategic-council/team | Teams | - | Council of 5 business perspectives |

---

## Phase 1: Research

Use these skills when you need to understand the codebase, research external documentation, or plan complex features before writing code.

### research/codebase/explorer

**Deep codebase exploration before writing code. Maps structure, traces patterns, identifies conventions, outputs structured report.**

The researcher agent performs read-only codebase exploration to build a complete mental model before any changes. It reads CLAUDE.md, maps directory structure, detects the stack from config files, traces recurring patterns using Grep, and checks recent git history.

**What you get:**
- Stack detection (framework, language, database, hosting)
- Architecture patterns and directory structure
- Conventions (naming, exports, error handling, testing)
- Key files and their purposes
- Risks and potential issues
- Recommended approach for upcoming work

**Install & Usage:**
```bash
forja install research/codebase/explorer
researcher
```

[View full documentation](../skills/research/codebase/explorer/README.md)

---

### research/docs/researcher

**Research external documentation and APIs using web search and context fetching. Produces structured summaries with code examples.**

The doc-researcher agent finds, reads, and summarizes external documentation and APIs. It searches for official documentation first, then community resources, extracts key APIs, setup instructions, working examples, and flags gotchas like breaking changes or common mistakes.

**What you get:**
- Summary and overview
- Key APIs and concepts
- Setup and installation steps
- Code examples from the docs
- Gotchas, known issues, and breaking changes
- Sources with URLs

**Install & Usage:**
```bash
forja install research/docs/researcher
doc-researcher
```

[View full documentation](../skills/research/docs/researcher/README.md)

---

### research/architecture/planner

**Create implementation plans with phases, file lists, and dependency maps. Analyzes codebase before proposing changes.**

The planner agent analyzes your codebase and creates phased implementation plans. It reads CLAUDE.md, maps directory structure, traces data flow, identifies files to create and modify, and outputs a multi-phase plan with specific file paths, dependencies, risks, and open questions.

**What you get:**
- Context (what exists today and how the feature fits)
- Phases with files to create and modify
- Dependencies between phases
- Risks and mitigation strategies
- Open questions that need answers

**Install & Usage:**
```bash
forja install research/architecture/planner
planner
```

[View full documentation](../skills/research/architecture/planner/README.md)

---

### research/planning/forja-plan

**Planning pipeline: interview, research, auto-detect agents, auto-size team, save executable plan to ~/.forja/plans/**

The forja-plan command runs a 5-step planning pipeline: (1) Interview - asks questions about project type, goals, exclusions, and planning depth. (2) Research - spawns an Explore subagent to map the codebase and detect stack. (3) Agents & Team Sizing - auto-detects required skills and suggests team size. (4) Build Phases - creates implementation phases with dependencies. (5) Save Plan - generates both JSON metadata and Markdown plan files.

**What you get:**
- Interactive interview with 4-8 questions
- Automated codebase research
- Agent and team size recommendations
- Implementation phases with dependencies
- Executable plan saved to `~/.forja/plans/`

**Install & Usage:**
```bash
forja install research/planning/forja-plan
/forja-plan Add user authentication with JWT
```

[View full documentation](../skills/research/planning/forja-plan/README.md)

---

## Phase 2: Code

Use these skills to implement features following language-specific conventions and best practices.

### code/typescript/feature

**TypeScript specialist with strict types, no any, proper patterns. Follows existing conventions and prefers boring solutions.**

The ts-coder agent writes strict, type-safe TypeScript code following project conventions. It reads CLAUDE.md, checks tsconfig.json for compiler options, studies existing files, and reuses existing types and utilities.

**Standards:**
- Strict TypeScript with no `any` or unsafe casts
- Named exports (not default exports)
- Interface-first design
- Zod for runtime validation at boundaries
- Type narrowing with guards instead of assertions

**Install & Usage:**
```bash
forja install code/typescript/feature
ts-coder
```

[View full documentation](../skills/code/typescript/feature/README.md)

---

### code/python/feature

**Python specialist for FastAPI/Django with type hints, pydantic models, and clean architecture.**

The py-coder agent writes typed, well-structured Python following project conventions. It detects the framework (FastAPI, Django, Flask) from project structure, uses type hints on all functions, leverages Pydantic for data models, and follows framework-specific patterns.

**Standards:**
- Type hints on all functions (parameters and return types)
- Pydantic BaseModel for DTOs and validation
- Dataclasses for internal models
- f-strings for formatting
- pathlib.Path instead of os.path

**Install & Usage:**
```bash
forja install code/python/feature
py-coder
```

[View full documentation](../skills/code/python/feature/README.md)

---

### code/golang/feature

**Go specialist with standard project layout, interface-first design, and proper error wrapping.**

The go-coder agent writes idiomatic Go code following standard project layout and conventions. It uses interface-first design, wraps errors with fmt.Errorf for context, accepts interfaces and returns structs, and follows naming conventions like MustX for panic-on-error functions.

**Standards:**
- Standard Go project layout (cmd, internal, pkg)
- Interface-first design (accept interfaces, return structs)
- Error wrapping with fmt.Errorf
- Table-driven tests
- Explicit error handling (no silent failures)

**Install & Usage:**
```bash
forja install code/golang/feature
go-coder
```

[View full documentation](../skills/code/golang/feature/README.md)

---

### code/rust/feature

**Rust specialist with thiserror/anyhow, idiomatic ownership patterns, and derive traits.**

The rust-coder agent writes idiomatic Rust code with proper ownership patterns. It uses thiserror for library errors and anyhow for applications, derives common traits (Debug, Clone, PartialEq), minimizes cloning, and follows Rust conventions for module structure, naming, and error handling.

**Standards:**
- thiserror for library errors, anyhow for applications
- Derive common traits (Debug, Clone, PartialEq)
- Idiomatic ownership patterns (minimize cloning)
- Result types for fallible operations
- Modules organized by feature

**Install & Usage:**
```bash
forja install code/rust/feature
rust-coder
```

[View full documentation](../skills/code/rust/feature/README.md)

---

### code/nextjs/feature

**Next.js specialist with App Router, Server Components, Server Actions, and Tailwind v4.**

The nextjs-coder agent builds Next.js applications using App Router conventions. It creates Server Components by default, uses Client Components only when needed ('use client'), implements Server Actions for mutations, and applies Tailwind v4 for styling.

**Standards:**
- App Router file conventions (page.tsx, layout.tsx)
- Server Components by default
- Client Components ('use client') only when necessary
- Server Actions for data mutations
- Tailwind v4 for styling

**Install & Usage:**
```bash
forja install code/nextjs/feature
nextjs-coder
```

[View full documentation](../skills/code/nextjs/feature/README.md)

---

### code/nestjs/feature

**NestJS specialist with modules, services, controllers, Prisma integration, and class-validator DTOs.**

The nestjs-coder agent builds NestJS backend applications following the framework's modular architecture. It creates modules with controllers and services, uses class-validator DTOs for request validation, integrates Prisma for database access, and implements dependency injection patterns.

**Standards:**
- Modular architecture (feature modules)
- Controllers for routing, Services for business logic
- class-validator DTOs for validation
- Prisma for database operations
- Dependency injection with @Injectable()

**Install & Usage:**
```bash
forja install code/nestjs/feature
nestjs-coder
```

[View full documentation](../skills/code/nestjs/feature/README.md)

---

### code/database/feature

**Database specialist for schemas, migrations, and queries. Supports Prisma, Drizzle, and raw SQL.**

The db-coder agent creates database schemas, migrations, and queries. It detects your ORM or query builder (Prisma, Drizzle, raw SQL), follows existing schema patterns, creates proper indexes, enforces foreign key constraints, and writes migrations that can be safely rolled back.

**Standards:**
- Schema design with proper constraints and indexes
- Reversible migrations
- Query optimization (avoid N+1, use joins)
- ORM-specific patterns (Prisma schema, Drizzle queries)
- Data integrity enforcement

**Install & Usage:**
```bash
forja install code/database/feature
db-coder
```

[View full documentation](../skills/code/database/feature/README.md)

---

### code/general/feature

**General-purpose coding agent. Auto-detects project stack and writes code following existing conventions.**

The coder agent is a general-purpose implementation specialist that auto-detects your project stack and adapts to existing conventions. It reads CLAUDE.md, identifies the tech stack from config files, studies existing code patterns, and reuses utilities before creating new ones.

**Principles:**
- Auto-detect stack from config files
- Follow existing patterns (naming, structure, style)
- Prefer boring solutions over abstractions
- Small, focused changes (one concern per function)
- Reuse existing utilities

**Install & Usage:**
```bash
forja install code/general/feature
coder
```

[View full documentation](../skills/code/general/feature/README.md)

---

## Phase 3: Test

Use these skills to write tests, enforce TDD workflows, and analyze test coverage.

### test/tdd/workflow

**TDD Red-Green-Refactor cycle. Write failing tests first, implement minimum code to pass, refactor.**

The tdd-guide agent enforces the Test-Driven Development cycle: Red (write a failing test), Green (write minimum implementation to pass), Refactor (improve code while keeping tests passing). It ALWAYS writes tests first, targets 80%+ coverage, and ensures each test covers one specific behavior.

**TDD Cycle:**
1. RED - Write a failing test that describes expected behavior
2. GREEN - Write minimum code to make the test pass
3. REFACTOR - Improve code while keeping all tests green

**Rules:**
- NEVER write implementation before the test
- Each test covers one behavior
- Tests are independent (no shared state)

**Install & Usage:**
```bash
forja install test/tdd/workflow
tdd-guide
```

[View full documentation](../skills/test/tdd/workflow/README.md)

---

### test/generate/suite

**Generate unit and integration tests for existing code. Analyzes source to produce comprehensive test suites.**

The test-generator agent writes comprehensive test suites for existing code. It reads source files, identifies the test framework, studies existing test patterns, and generates tests covering happy path, edge cases, and error conditions.

**What you get:**
- Unit tests (isolated, mocked dependencies)
- Integration tests (real dependencies)
- Tests matching existing patterns and naming
- Verification that tests pass

**Install & Usage:**
```bash
forja install test/generate/suite
test-generator
```

[View full documentation](../skills/test/generate/suite/README.md)

---

### test/e2e/playwright

**Playwright E2E tests with Page Object Model, auto-waiting, and trace-based debugging.**

The e2e-tester agent writes reliable, maintainable Playwright browser tests using the Page Object Model. It creates Page Objects for UI components, writes test specs with proper isolation, uses Playwright's auto-waiting features, and enables trace collection for debugging.

**Patterns:**
- Page Object Model for reusable UI abstractions
- Auto-waiting (no manual sleeps or timeouts)
- Test isolation (each test starts fresh)
- User-centric selectors (getByRole, getByLabel)
- Trace collection for debugging failures

**Install & Usage:**
```bash
forja install test/e2e/playwright
e2e-tester
```

[View full documentation](../skills/test/e2e/playwright/README.md)

---

### test/coverage/analyzer

**Analyze test coverage gaps and generate targeted tests to improve coverage metrics.**

The coverage-analyzer agent runs your test suite with coverage enabled, parses the report to identify uncovered lines and branches, prioritizes gaps by risk (error handlers > business logic > utilities), and writes targeted tests for the highest-priority gaps.

**What you get:**
- Coverage report parsing (Jest, Vitest, pytest, Go)
- Risk-prioritized gaps
- Targeted tests for high-priority gaps
- Re-run verification toward 80% target

**Install & Usage:**
```bash
forja install test/coverage/analyzer
coverage-analyzer
```

[View full documentation](../skills/test/coverage/analyzer/README.md)

---

## Phase 4: Review

Use these skills to review code quality, security, performance, and simplify implementations.

### review/code-quality/reviewer

**Fresh-context AI code review. Reviews git diff for quality, correctness, security, and pattern consistency.**

The reviewer agent performs fresh-context code review by analyzing git diffs. It categorizes findings by severity (CRITICAL, WARNING, SUGGESTION), checks for security issues (hardcoded secrets, injection risks), logic errors, performance problems (N+1 queries), and code quality issues.

**What you get:**
- Severity-categorized findings
- Security, logic, performance, and quality checks
- Specific fix examples for each issue
- Verdict: APPROVE, REQUEST CHANGES, or COMMENT

**Install & Usage:**
```bash
forja install review/code-quality/reviewer
reviewer
```

[View full documentation](../skills/review/code-quality/reviewer/README.md)

---

### review/security/auditor

**Security audit covering OWASP Top 10, hardcoded secrets, injection vulnerabilities, and auth issues.**

The security-auditor agent finds vulnerabilities before attackers do. It scans for hardcoded secrets, reviews authentication and authorization logic, checks input validation at system boundaries, analyzes data flow for injection vulnerabilities, and reviews dependency versions for known CVEs.

**What you get:**
- OWASP Top 10 vulnerability checks
- Hardcoded secrets and credentials detection
- Broken access control and IDOR
- SQL injection, XSS, and path traversal
- Missing input validation
- Weak cryptography

**Install & Usage:**
```bash
forja install review/security/auditor
security-auditor
```

[View full documentation](../skills/review/security/auditor/README.md)

---

### review/performance/analyzer

**Performance review covering algorithmic complexity, N+1 queries, unnecessary re-renders, and bundle size.**

The perf-analyzer agent finds performance bottlenecks before they hit production. It analyzes algorithmic complexity, identifies N+1 queries and missing indexes, detects unnecessary React re-renders, checks bundle size impact, and reviews for missing pagination or caching.

**What you get:**
- N+1 database queries detection
- Missing indexes on filtered/sorted columns
- Unbounded queries without LIMIT
- Unnecessary React re-renders
- Missing memoization
- Bundle size bloat

**Install & Usage:**
```bash
forja install review/performance/analyzer
perf-analyzer
```

[View full documentation](../skills/review/performance/analyzer/README.md)

---

### review/pr-workflow/reviewer

**Full PR review lifecycle: create review, iterate on feedback, approve or request changes.**

The pr-reviewer agent manages the complete PR review lifecycle. It fetches the PR diff using gh CLI, reads each changed file for context, reviews against quality/security/performance criteria, writes structured review comments, and sets a verdict.

**Three-pass process:**
1. Understanding - Read PR description and map changes
2. Correctness - Check logic, edge cases, error handling
3. Quality - Review patterns, naming, performance, security

**Install & Usage:**
```bash
forja install review/pr-workflow/reviewer
pr-reviewer
```

[View full documentation](../skills/review/pr-workflow/reviewer/README.md)

---

### review/code-simplifier/simplifier

**Simplifies and refines code for clarity, consistency, and maintainability while preserving all functionality.**

The code-simplifier agent refines recently written code for clarity without changing behavior. It uses git diff to identify changes, renames unclear variables, replaces complex conditionals with early returns, extracts magic numbers, reduces nesting depth, and removes dead code.

**What it does:**
- Focus on recently modified code (via git diff)
- Rename unclear variables for clarity
- Replace complex conditionals with guard clauses
- Extract magic numbers into named constants
- Reduce nesting depth
- Run tests to verify behavior unchanged

**Install & Usage:**
```bash
forja install review/code-simplifier/simplifier
code-simplifier
```

[View full documentation](../skills/review/code-simplifier/simplifier/README.md)

---

## Phase 5: Deploy

Use these skills to commit changes, create pull requests, and verify deployments.

### deploy/git/commit

**Conventional commits with type, scope, and descriptive messages. Analyzes staged changes and creates well-formatted commits.**

The committer agent creates conventional commits by analyzing staged changes. It runs git status and git diff to understand changes, checks recent commit history to match repository style, determines the commit type and scope, and writes concise descriptive messages.

**Conventional Commits format:**
```
type(scope): subject (max 50 chars)

Body explaining HOW and WHY (wrap at 72 chars).

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Commit types:** feat, fix, refactor, test, docs, chore, perf, ci

**Install & Usage:**
```bash
forja install deploy/git/commit
committer
```

[View full documentation](../skills/deploy/git/commit/README.md)

---

### deploy/git/pr

**Push branches and create Pull Requests via gh CLI with structured descriptions.**

The pr-creator agent pushes branches and creates Pull Requests using the gh CLI. It checks branch tracking status, reviews changes with git log and git diff, pushes the branch with -u flag, creates the PR with a structured description (Summary, Changes, Test Plan), and outputs the PR URL.

**What it does:**
- Check current branch and remote status
- Review commits and changes
- Push branch to remote
- Create PR with structured description
- Output the PR URL

**Install & Usage:**
```bash
forja install deploy/git/pr
pr-creator
```

[View full documentation](../skills/deploy/git/pr/README.md)

---

### deploy/verify/checker

**Post-deploy verification: CI status checks, health endpoints, and smoke tests.**

The deploy-checker agent verifies deployments are healthy after merge or deploy. It checks CI/CD pipeline status for the latest commit, verifies health check endpoints respond correctly, runs smoke tests against the deployed environment, and reports overall status with details.

**What it does:**
- Check CI/CD pipeline status (gh pr checks, gh run watch)
- Verify health check endpoints (HTTP 200 responses)
- Run smoke tests against deployed environment
- Report overall deployment health

**Install & Usage:**
```bash
forja install deploy/verify/checker
deploy-checker
```

[View full documentation](../skills/deploy/verify/checker/README.md)

---

## Teams

Use agent teams for complex tasks requiring orchestration across multiple phases.

### teams/full-product/team

**5-agent product development team: researcher, coder, tester, reviewer, deployer. Orchestrates parallel workflows across all forja phases.**

The full-product team creates a task list with dependencies across all phases. The researcher explores the codebase and produces a plan first. After plan approval, the coder implements the feature, then the tester writes and runs tests, the code-simplifier refines for clarity, the reviewer checks quality/security, the chronicler documents decisions, and finally the deployer commits and creates a PR.

**When to use:**
- New features that touch multiple files (10+ files)
- Features that need research + implementation + tests + review
- Multi-phase, cross-cutting changes
- When you want the full development lifecycle automated

**When NOT to use:**
- Simple one-file fixes (use a single session instead)
- Quick bug fixes (use the quick-fix team)
- Tasks where steps are heavily sequential with no parallelism

**Usage:**
```bash
/full-product Add user authentication with JWT
```

**Install:**
```bash
forja install teams/full-product/team
```

[View full documentation](../skills/teams/full-product/team/README.md)

---

### teams/solo-sprint/team

**Lightweight 2-agent team: combined coder-tester and quick reviewer. For medium features that need tests and review.**

The solo-sprint team runs a 3-step workflow. The coder-tester implements the feature and writes tests together in one pass (targeting 80%+ coverage). Then the code-simplifier refines the code for clarity without changing behavior. Finally, the reviewer performs a quick code review checking correctness, error handling, and security basics.

**When to use:**
- Medium features (3-10 files)
- Moderate complexity tasks
- When you need implementation, tests, and review
- Faster than full-product, more thorough than quick-fix

**When NOT to use:**
- Large multi-phase features (use full-product team)
- Simple hotfixes (use quick-fix team)
- Refactoring work (use refactor team)

**Usage:**
```bash
/solo-sprint Add pagination to user list endpoint
```

**Install:**
```bash
forja install teams/solo-sprint/team
```

[View full documentation](../skills/teams/solo-sprint/team/README.md)

---

### teams/quick-fix/team

**Minimal 2-agent team for hotfixes: coder fixes the issue, deployer commits and creates PR.**

The quick-fix team runs a 3-step workflow for fast hotfix delivery. The coder finds the root cause, fixes the bug with minimal changes, and runs existing tests to verify no regressions. Then the deployer creates a conventional commit with type 'fix', pushes to a new branch, and creates a PR.

**When to use:**
- Fast hotfixes (1-3 files, single concern)
- Bug fixes with clear error messages
- When you need minimal testing and rapid deployment
- Production issues that need quick turnaround

**When NOT to use:**
- New features (use solo-sprint or full-product)
- Complex bugs requiring investigation
- Changes that need comprehensive testing

**Usage:**
```bash
/quick-fix Fix TypeError in login handler when email is missing
```

**Install:**
```bash
forja install teams/quick-fix/team
```

[View full documentation](../skills/teams/quick-fix/team/README.md)

---

### teams/refactor/team

**3-agent refactoring team: analyzer maps the code, refactorer makes structural changes, reviewer verifies behavioral equivalence.**

The refactor team specializes in structural changes that preserve behavior. The analyzer maps public APIs, dependencies, test coverage, and risk areas, then produces an ordered refactoring plan. After plan approval, the refactorer executes the plan step-by-step, running tests after each change. The reviewer checks ONLY for behavioral regressions and API breaks.

**When to use:**
- Structural code changes preserving behavior
- Extract, rename, or reorganize modules/classes
- When test coverage is sufficient to catch regressions
- Refactoring with clear objectives

**When NOT to use:**
- Code has low/no test coverage (too risky)
- Changes that intentionally modify behavior
- Quick cleanup (do it manually)

**Usage:**
```bash
/refactor Extract auth logic from UserController into AuthService
```

**Install:**
```bash
forja install teams/refactor/team
```

[View full documentation](../skills/teams/refactor/team/README.md)

---

### teams/dispatch/team

**Parallel task dispatcher — fan-out independent work to background agents while you keep working in the main session.**

The dispatch pattern decomposes your request into discrete independent tasks and fans them out to specialized background agents. There's no TeamCreate, no shared task list, and no dedicated lead agent — just you dispatching tasks in parallel using the Task tool with `run_in_background: true`.

**Workflow:**
1. Decompose the request into independent tasks
2. Map each task to the appropriate subagent_type and model
3. Dispatch all tasks in a single message
4. Monitor progress using TaskOutput
5. Synthesize results when all agents complete

**When to use:**
- Multiple independent tasks that can run in parallel
- Research + implementation + review happening simultaneously
- Exploring multiple approaches to compare results
- When you want to stay in control while delegating work

**When NOT to use:**
- All tasks depend on each other (do sequentially instead)
- Single task (no need for parallelism)
- When you need tight orchestration (use a team instead)

**Usage:**
```bash
/dispatch Research authentication patterns, implement rate limiting, and review security issues
```

**Install:**
```bash
forja install teams/dispatch/team
```

[View full documentation](../skills/teams/dispatch/team/README.md)

---

### teams/technical-council/team

**Council of engineering personas — consult 5 diverse technical perspectives on architecture, infrastructure, security, quality, and performance decisions.**

The technical council facilitates consultation on engineering decisions, not execution. You provide a technical question or decision to evaluate. The facilitator spawns all 5 council members in parallel as background agents. Each member analyzes the decision from their unique perspective and bias. After all members respond, the facilitator synthesizes their feedback into a decision matrix.

**Council Members:**
- Principal Engineer (Simplicity and long-term maintenance)
- Platform Engineer (Operational burden and infrastructure costs)
- Security Engineer (Threat modeling and compliance)
- QA Lead (Testability and edge cases)
- Performance Engineer (Latency, throughput, bottlenecks)

**When to use:**
- Architecture decisions (monolith vs. microservices, database choice)
- Infrastructure changes (deployment strategy, observability)
- Technical debt prioritization
- When you want diverse expert perspectives before deciding

**When NOT to use:**
- Simple implementation questions (ask directly)
- When you've already decided and just need execution
- Product or business decisions (use strategic-council)

**Usage:**
```bash
/technical-council Should we migrate from REST to GraphQL?
```

**Install:**
```bash
forja install teams/technical-council/team
```

[View full documentation](../skills/teams/technical-council/team/README.md)

---

### teams/strategic-council/team

**Council of business personas — consult 5 diverse strategic perspectives on product, design, data, growth, and operations decisions.**

The strategic council facilitates consultation on business and product decisions, not execution. You provide a business or product question to evaluate. The facilitator spawns all 5 council members in parallel as background agents. Each member analyzes the decision from their unique perspective and bias. After all members respond, the facilitator synthesizes their feedback into a decision matrix.

**Council Members:**
- Product Lead (User value and business impact)
- Design Lead (UX and interaction patterns)
- Data/Analytics Lead (Metrics and measurement)
- Growth Lead (Acquisition and retention)
- Operations Lead (Team capacity and process)

**When to use:**
- Product decisions (feature prioritization, roadmap planning)
- UX/design direction (new user flows, interface changes)
- Growth strategies (onboarding optimization, retention)
- When you want diverse business perspectives before deciding

**When NOT to use:**
- Simple implementation questions (ask directly)
- When you've already decided and just need execution
- Technical architecture decisions (use technical-council)

**Usage:**
```bash
/strategic-council Should we add a free tier or keep pricing simple?
```

**Install:**
```bash
forja install teams/strategic-council/team
```

[View full documentation](../skills/teams/strategic-council/team/README.md)

---

## Discovering Skills

### Browse vs. Search

**Browse this catalog** when you want to explore all available skills by phase or tech stack.

**Search dynamically** when you know what you're looking for:

```bash
# Search for skills by keyword
forja search typescript

# List all available skills
forja list --available

# List installed skills
forja list
```

### Install Individual Skills

Install skills one at a time using their skill ID:

```bash
forja install research/codebase/explorer
forja install code/typescript/feature
forja install test/tdd/workflow
```

### Install by Phase

Install all skills for a specific phase:

```bash
forja install research/*    # All research skills
forja install code/*         # All code skills
forja install test/*         # All test skills
forja install review/*       # All review skills
forja install deploy/*       # All deploy skills
forja install teams/*        # All team configurations
```

---

**Need help?** Run `forja help` or visit the [forja documentation](./GETTING-STARTED.md).
