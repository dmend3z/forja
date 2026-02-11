# Changelog

All notable changes to forja will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - Unreleased

### Added

- `forja monitor` — real-time web dashboard for monitoring Claude Code agent teams
  - 4-panel UI: team sidebar, kanban task board, message feed, activity timeline
  - Live SSE streaming from filesystem watcher on `~/.claude/teams/` and `~/.claude/tasks/`
  - Embedded web assets (HTML/CSS/JS) — no external dependencies, dark terminal theme
  - Auto-opens browser on `localhost:3030`, configurable with `--port` and `--no-open`
  - Graceful shutdown on Ctrl+C
- `dispatch` team preset — parallel task dispatcher that fans out independent work to background agents
- Registry validation for skill catalog integrity checks
- Model enforcement guidelines for agent spawning in CLAUDE.md
- Auto-install all skills during `forja init` — one command to go from zero to ready
- Project stack detection during init (Next.js, Rust, Python, Go, TypeScript)
- `forja` with no args shows status dashboard or welcome screen
- Shell installer script (`install.sh`) for `curl | sh` installs
- Silent bulk installation for scripted workflows
- Show installation tips in `status` and `update` output
- `--yes` flag for non-interactive `uninstall` and `delete` commands
- Detailed `--help` text for all CLI commands
- Interactive init wizard with guided setup
- Community roadmap with contributor guide
- Auto-release workflow for automated versioning
- Code-Simplifier as 6th agent in team configurations

### Changed

- `forja init` output is minimal — 3-4 lines with checkmarks, no file paths
- Restructure README for GitHub visitors — hero, quick start, before/after, summary table
- Error messages include contextual hints and structured exit codes
- Clarify agent team roles and task dependencies in orchestration
- 5 team presets now available: full-product, solo-sprint, quick-fix, refactor, dispatch

### Fixed

- Replace deprecated `macos-13` CI runner with `macos-latest`

### Removed

- Unused `walkdir` and `same-file` dependencies

## [0.1.1] - 2025-06-03

### Added

- Pre-built binary for `aarch64-apple-darwin` (Apple Silicon)
- Analytics integration in website

## [0.1.0] - 2025-06-01

### Added

- CLI with install, uninstall, search, list, info, phases, doctor commands
- 28 curated skills across 5 workflow phases (Research, Code, Test, Review, Deploy)
- 4 agent team configurations (full-product, solo-sprint, quick-fix, refactor)
- Symlink-based skill installation into `~/.claude/agents/`
- Local development mode with automatic symlink detection
- Plan and execute workflow with model profiles (fast, balanced, max)
- Custom team creation wizard
- Registry management with git-based updates

[0.2.0]: https://github.com/dmend3z/forja/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/dmend3z/forja/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/dmend3z/forja/releases/tag/v0.1.0
