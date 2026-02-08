# Changelog

All notable changes to forja will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-06-XX

### Added

- Auto-install all skills during `forja init` — one command to go from zero to ready
- Project stack detection during init (Next.js, Rust, Python, Go, TypeScript, and more)
- `forja` with no args shows contextual status dashboard (or welcome pitch if not initialized)
- Shell installer script (`install.sh`) for non-npm installs via `curl | sh`
- `install_all_quiet()` function for silent bulk installation

### Changed

- `forja init` output is now minimal (3-4 lines with checkmarks, no file paths)
- README restructured for GitHub visitors: hero, quick start, before/after, summary table
- CLI subcommand field is now `Option<Commands>` to support no-args invocation

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
