# forja-tui: Task Launcher TUI

**Date:** 2026-02-16
**Status:** Approved

## Problem

`forja task` requires the task description as an inline CLI argument (`forja task "fix bug"`). For complex tasks, typing a multi-line description in a shell string is awkward. There's no way to preview the generated prompt or configure team/profile visually before launching.

## Solution

New crate `crates/forja-tui/` — a ratatui-based TUI that opens when `forja task` is invoked without arguments. The user describes their task in a textarea, configures team and profile with inline selectors, previews the generated prompt, and launches with Enter. The TUI closes and Claude CLI runs normally in the terminal.

## Architecture

### Crate structure

```
crates/
├── forja-core/     # Business logic (existing)
├── forja-cli/      # CLI dispatcher (existing, gains forja-tui dep)
├── forja-tui/      # NEW: TUI for task launcher
└── forja-spark/    # Desktop process manager (existing)
```

### Dependencies

- `forja-core` — ForjaPaths, ForjaState, Profile, TeamEntry
- `ratatui` + `crossterm` — TUI rendering and terminal control
- `tui-textarea` — multi-line editable textarea widget

### Entry point

```
forja task              → opens TUI
forja task "fix bug"    → current behavior (direct launch)
forja task --print "x"  → current behavior (print mode)
```

The `task` argument in clap becomes `Option<String>`.

## TUI Layout

```
┌─ forja task ──────────────────────────────────────┐
│                                                   │
│  Task Description                                 │
│  ┌───────────────────────────────────────────────┐│
│  │ (editable multi-line textarea)                ││
│  │                                               ││
│  └───────────────────────────────────────────────┘│
│                                                   │
│  Configuration                                    │
│  Team:    < Solo >   quick-fix   solo-sprint      │
│  Profile: < balanced >                            │
│                                                   │
│  Prompt Preview                                   │
│  ┌───────────────────────────────────────────────┐│
│  │ (read-only, updates live based on inputs)     ││
│  └───────────────────────────────────────────────┘│
│                                                   │
│  Tab: next  Shift+Tab: prev  Enter: launch        │
│  Esc: quit                                        │
└───────────────────────────────────────────────────┘
```

Three focusable areas (Tab/Shift+Tab to cycle):
1. **Textarea** — task description (tui-textarea widget)
2. **Team selector** — Left/Right to pick: Solo, quick-fix, solo-sprint, full-product, custom teams
3. **Profile selector** — Left/Right to pick: fast, balanced, max

Prompt Preview is read-only and updates automatically.

## Data Flow

```
TaskInput {
    description: String,
    team: Option<String>,
    profile: Profile,
}
    │
    ▼
build_prompt() → preview panel (live update)
    │
    ▼ (on Enter)
TaskOutput {
    description: String,
    team: Option<String>,
    profile: Profile,
}
    │
    ▼
forja-cli::commands::task::run() → claude CLI
```

The TUI only produces a `TaskOutput` and returns it to the CLI. The CLI handles launch as it does today. No logic duplication.

## Module layout

```
crates/forja-tui/src/
├── lib.rs           # pub fn launch() -> Result<Option<TaskOutput>>
├── app.rs           # App state machine (focus, inputs, team/profile options)
├── ui.rs            # render function (layout + widgets)
└── input.rs         # key event handling
```

Estimated ~400-600 lines total.

## CLI Integration

In `forja-cli/src/commands/task.rs`:

```rust
pub fn run(task: Option<&str>, print: bool, team: Option<&str>, profile: Option<&str>) -> Result<()> {
    match task {
        Some(t) => run_with_task(t, print, team, profile),
        None => {
            let output = forja_tui::launch()?;
            match output {
                Some(o) => run_with_task(&o.description, false, o.team.as_deref(), Some(o.profile.as_str())),
                None => Ok(()),  // user pressed Esc
            }
        }
    }
}
```

## Error Handling

- Terminal doesn't support raw mode (piped stdin, CI): fallback to requiring the `task` argument, show error if missing
- Esc in TUI: returns `Ok(None)`, CLI exits cleanly
- Enter with empty textarea: does not launch, shows inline feedback
- Panic handler restores terminal via crossterm to prevent broken state

## Testing

- Unit tests for `app.rs`: state transitions (focus cycling, team/profile selection)
- Unit tests for prompt generation: given a TaskInput, verify prompt contains correct fields
- No rendering tests — ratatui widgets are declarative, visual testing not worth the cost
- `launch()` itself is not unit-testable (requires terminal), but all logic lives in pure functions

## Out of Scope

- TUI does NOT show execution progress (TUI closes before claude runs)
- TUI does NOT persist task history
- TUI does NOT replace `forja plan` (separate flow)
- No theme/color customization
