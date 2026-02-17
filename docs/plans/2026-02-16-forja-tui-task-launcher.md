# forja-tui Task Launcher Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a ratatui-based TUI that opens on `forja task` (no args) for describing tasks, configuring team/profile, previewing prompts, and launching Claude Code.

**Architecture:** New `crates/forja-tui/` crate with ratatui + crossterm + tui-textarea. The TUI returns a `TaskOutput` struct to `forja-cli`, which handles the actual Claude launch. Pure function separation for testability.

**Tech Stack:** Rust 1.93 (edition 2024), ratatui 0.29, crossterm 0.28, tui-textarea 0.7, forja-core

---

### Task 1: Scaffold the `forja-tui` crate

**Files:**
- Create: `crates/forja-tui/Cargo.toml`
- Create: `crates/forja-tui/src/lib.rs`
- Modify: `Cargo.toml` (workspace members)
- Modify: `crates/forja-cli/Cargo.toml` (add forja-tui dep)

**Step 1: Create `crates/forja-tui/Cargo.toml`**

```toml
[package]
name = "forja-tui"
description = "Interactive TUI task launcher for forja"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true

[dependencies]
forja-core = { path = "../forja-core" }
ratatui = "0.29"
crossterm = "0.28"
tui-textarea = "0.7"

[dev-dependencies]
tempfile = { workspace = true }
```

**Step 2: Create minimal `crates/forja-tui/src/lib.rs`**

```rust
pub mod app;

use forja_core::error::Result;
use forja_core::models::profile::Profile;

pub struct TaskOutput {
    pub description: String,
    pub team: Option<String>,
    pub profile: Profile,
}

pub fn launch() -> Result<Option<TaskOutput>> {
    todo!("TUI not yet implemented")
}
```

**Step 3: Add to workspace `Cargo.toml` members**

Add `"crates/forja-tui"` to the `members` array.

**Step 4: Add `forja-tui` dep to `crates/forja-cli/Cargo.toml`**

```toml
forja-tui = { path = "../forja-tui" }
```

**Step 5: Create empty `crates/forja-tui/src/app.rs`**

```rust
// App state machine — will be implemented in Task 3
```

**Step 6: Verify it compiles**

Run: `cargo check -p forja-tui`
Expected: compiles with no errors (todo! is fine for now)

**Step 7: Commit**

```bash
git add crates/forja-tui/ Cargo.toml crates/forja-cli/Cargo.toml
git commit -m "feat(tui): scaffold forja-tui crate with deps"
```

---

### Task 2: Make `task` argument optional in CLI

**Files:**
- Modify: `crates/forja-cli/src/cli.rs:200-215` (Task variant)
- Modify: `crates/forja-cli/src/main.rs:53-58` (dispatch)
- Modify: `crates/forja-cli/src/commands/task.rs:21` (run signature)

**Step 1: Write the failing test**

In `crates/forja-cli/src/commands/task.rs`, the existing `run` takes `task: &str`. Change the signature to `task: Option<&str>` and verify the build breaks — confirming the call sites need updating.

Actually this is a refactor step, not TDD-able in isolation. Proceed directly.

**Step 2: Change clap `Task` variant — make `task` optional**

In `crates/forja-cli/src/cli.rs`, change the `Task` variant:

```rust
Task {
    /// Task description (omit to open interactive TUI)
    task: Option<String>,

    /// Run in non-interactive mode (output to stdout)
    #[arg(long)]
    print: bool,

    /// Optional team name (configured or preset: full-product, solo-sprint, quick-fix)
    #[arg(long)]
    team: Option<String>,

    /// Model profile override (only with --team)
    #[arg(long)]
    profile: Option<String>,
},
```

**Step 3: Update dispatch in `main.rs`**

```rust
Commands::Task {
    ref task,
    print,
    ref team,
    ref profile,
} => commands::task::run(task.as_deref(), print, team.as_deref(), profile.as_deref()),
```

**Step 4: Update `task.rs` — split into TUI vs direct path**

Rename the existing `run` to `run_with_task` (private), create new `run`:

```rust
pub fn run(task: Option<&str>, print: bool, team: Option<&str>, profile: Option<&str>) -> Result<()> {
    if Command::new("claude").arg("--version").output().is_err() {
        return Err(ForjaError::ClaudeCliNotFound);
    }

    match task {
        Some(t) => run_with_task(t, print, team, profile),
        None => {
            if print {
                return Err(ForjaError::Dialoguer(
                    "--print requires a task description".to_string(),
                ));
            }
            let output = forja_tui::launch()?;
            match output {
                Some(o) => run_with_task(
                    &o.description,
                    false,
                    o.team.as_deref(),
                    Some(o.profile.as_str()),
                ),
                None => Ok(()), // user pressed Esc
            }
        }
    }
}

fn run_with_task(task: &str, print: bool, team: Option<&str>, profile: Option<&str>) -> Result<()> {
    // ... existing logic from current `run`, starting from `match team {`
}
```

**Step 5: Verify it compiles**

Run: `cargo check -p forja-cli`
Expected: compiles (the `todo!()` in forja-tui launch is fine at compile time)

**Step 6: Run existing tests**

Run: `cargo test -p forja-cli`
Expected: all existing tests pass (they test `build_args`, `build_team_prompt`, etc. — unchanged)

**Step 7: Commit**

```bash
git add crates/forja-cli/src/cli.rs crates/forja-cli/src/main.rs crates/forja-cli/src/commands/task.rs
git commit -m "refactor(cli): make task argument optional for TUI launch"
```

---

### Task 3: Implement `App` state machine

**Files:**
- Create: `crates/forja-tui/src/app.rs`
- Test: `crates/forja-tui/src/app.rs` (inline `#[cfg(test)]`)

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_focus_is_textarea() {
        let app = App::new(vec!["Solo".into()], vec!["balanced".into()]);
        assert_eq!(app.focus, Focus::Textarea);
    }

    #[test]
    fn tab_cycles_focus_forward() {
        let mut app = App::new(vec!["Solo".into()], vec!["balanced".into()]);
        app.next_focus();
        assert_eq!(app.focus, Focus::Team);
        app.next_focus();
        assert_eq!(app.focus, Focus::Profile);
        app.next_focus();
        assert_eq!(app.focus, Focus::Textarea);
    }

    #[test]
    fn shift_tab_cycles_focus_backward() {
        let mut app = App::new(vec!["Solo".into()], vec!["balanced".into()]);
        app.prev_focus();
        assert_eq!(app.focus, Focus::Profile);
    }

    #[test]
    fn team_selector_wraps() {
        let mut app = App::new(
            vec!["Solo".into(), "quick-fix".into(), "solo-sprint".into()],
            vec!["balanced".into()],
        );
        app.focus = Focus::Team;
        assert_eq!(app.team_index, 0);
        app.select_next();
        assert_eq!(app.team_index, 1);
        app.select_next();
        assert_eq!(app.team_index, 2);
        app.select_next();
        assert_eq!(app.team_index, 0); // wraps
    }

    #[test]
    fn profile_selector_wraps() {
        let mut app = App::new(
            vec!["Solo".into()],
            vec!["fast".into(), "balanced".into(), "max".into()],
        );
        app.focus = Focus::Profile;
        assert_eq!(app.profile_index, 0);
        app.select_next();
        assert_eq!(app.profile_index, 1);
    }

    #[test]
    fn selected_team_none_for_solo() {
        let app = App::new(vec!["Solo".into(), "quick-fix".into()], vec!["balanced".into()]);
        assert_eq!(app.selected_team(), None);
    }

    #[test]
    fn selected_team_some_for_non_solo() {
        let mut app = App::new(vec!["Solo".into(), "quick-fix".into()], vec!["balanced".into()]);
        app.team_index = 1;
        assert_eq!(app.selected_team(), Some("quick-fix"));
    }

    #[test]
    fn selected_profile_returns_label() {
        let app = App::new(vec!["Solo".into()], vec!["fast".into(), "balanced".into(), "max".into()]);
        assert_eq!(app.selected_profile(), "fast");
    }

    #[test]
    fn description_empty_initially() {
        let app = App::new(vec!["Solo".into()], vec!["balanced".into()]);
        assert!(app.description().is_empty());
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p forja-tui`
Expected: FAIL — `App`, `Focus`, methods not defined

**Step 3: Implement `App` struct**

```rust
use tui_textarea::TextArea;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Textarea,
    Team,
    Profile,
}

pub struct App<'a> {
    pub focus: Focus,
    pub team_options: Vec<String>,
    pub team_index: usize,
    pub profile_options: Vec<String>,
    pub profile_index: usize,
    pub textarea: TextArea<'a>,
    pub should_quit: bool,
    pub should_launch: bool,
    pub error_message: Option<String>,
}

impl<'a> App<'a> {
    pub fn new(team_options: Vec<String>, profile_options: Vec<String>) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_placeholder_text("Describe your task here...");
        Self {
            focus: Focus::Textarea,
            team_options,
            team_index: 0,
            profile_options,
            profile_index: 0,
            textarea,
            should_quit: false,
            should_launch: false,
            error_message: None,
        }
    }

    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Textarea => Focus::Team,
            Focus::Team => Focus::Profile,
            Focus::Profile => Focus::Textarea,
        };
    }

    pub fn prev_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Textarea => Focus::Profile,
            Focus::Team => Focus::Textarea,
            Focus::Profile => Focus::Team,
        };
    }

    pub fn select_next(&mut self) {
        match self.focus {
            Focus::Team => {
                self.team_index = (self.team_index + 1) % self.team_options.len();
            }
            Focus::Profile => {
                self.profile_index = (self.profile_index + 1) % self.profile_options.len();
            }
            Focus::Textarea => {}
        }
    }

    pub fn select_prev(&mut self) {
        match self.focus {
            Focus::Team => {
                self.team_index = if self.team_index == 0 {
                    self.team_options.len() - 1
                } else {
                    self.team_index - 1
                };
            }
            Focus::Profile => {
                self.profile_index = if self.profile_index == 0 {
                    self.profile_options.len() - 1
                } else {
                    self.profile_index - 1
                };
            }
            Focus::Textarea => {}
        }
    }

    pub fn selected_team(&self) -> Option<&str> {
        if self.team_index == 0 {
            None // "Solo" is always index 0
        } else {
            Some(&self.team_options[self.team_index])
        }
    }

    pub fn selected_profile(&self) -> &str {
        &self.profile_options[self.profile_index]
    }

    pub fn description(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn try_launch(&mut self) {
        let desc = self.description();
        if desc.trim().is_empty() {
            self.error_message = Some("Task description required".to_string());
        } else {
            self.error_message = None;
            self.should_launch = true;
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p forja-tui`
Expected: all 9 tests PASS

**Step 5: Commit**

```bash
git add crates/forja-tui/src/app.rs
git commit -m "feat(tui): implement App state machine with tests"
```

---

### Task 4: Implement key event handling

**Files:**
- Create: `crates/forja-tui/src/input.rs`
- Modify: `crates/forja-tui/src/lib.rs` (add module)

**Step 1: Create `input.rs`**

Handle crossterm `KeyEvent` and delegate to `App` methods. When focus is on Textarea, forward key events to tui-textarea. When focus is on Team/Profile, handle Left/Right for selection. Tab/Shift+Tab always cycle focus. Esc quits. Ctrl+Enter launches (Enter on its own goes to textarea in textarea mode).

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Focus};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Global keys
    match key.code {
        KeyCode::Esc => {
            app.should_quit = true;
            return;
        }
        KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.prev_focus();
            return;
        }
        KeyCode::Tab => {
            app.next_focus();
            return;
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.try_launch();
            return;
        }
        _ => {}
    }

    // Focus-specific keys
    match app.focus {
        Focus::Textarea => {
            app.textarea.input(key);
        }
        Focus::Team | Focus::Profile => match key.code {
            KeyCode::Left | KeyCode::Char('h') => app.select_prev(),
            KeyCode::Right | KeyCode::Char('l') => app.select_next(),
            KeyCode::Enter => app.try_launch(),
            _ => {}
        },
    }
}
```

**Step 2: Add module to `lib.rs`**

Add `pub mod input;` to `lib.rs`.

**Step 3: Verify it compiles**

Run: `cargo check -p forja-tui`
Expected: compiles

**Step 4: Commit**

```bash
git add crates/forja-tui/src/input.rs crates/forja-tui/src/lib.rs
git commit -m "feat(tui): add key event handling for all focus modes"
```

---

### Task 5: Implement UI rendering

**Files:**
- Create: `crates/forja-tui/src/ui.rs`
- Modify: `crates/forja-tui/src/lib.rs` (add module)

**Step 1: Create `ui.rs` with the render function**

Layout: vertical split into 4 chunks — title, textarea (40%), config (2 lines), preview (remaining), help bar.

```rust
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, Focus};

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // title
            Constraint::Min(6),    // textarea
            Constraint::Length(4), // config (team + profile)
            Constraint::Min(4),    // preview
            Constraint::Length(1), // help bar
        ])
        .split(area);

    render_title(frame, chunks[0]);
    render_textarea(frame, app, chunks[1]);
    render_config(frame, app, chunks[2]);
    render_preview(frame, app, chunks[3]);
    render_help(frame, app, chunks[4]);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" forja task ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]));
    frame.render_widget(title, area);
}

fn render_textarea(frame: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if app.focus == Focus::Textarea {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Task Description ");
    app.textarea.set_block(block);
    frame.render_widget(&app.textarea, area);
}

fn render_config(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(2)])
        .split(area);

    // Team row
    let team_spans: Vec<Span> = std::iter::once(Span::styled("  Team:    ", Style::default().fg(Color::White)))
        .chain(app.team_options.iter().enumerate().map(|(i, name)| {
            if i == app.team_index {
                let style = if app.focus == Focus::Team {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                };
                Span::styled(format!(" {name} "), style)
            } else {
                Span::styled(format!(" {name} "), Style::default().fg(Color::DarkGray))
            }
        }))
        .collect();
    frame.render_widget(Paragraph::new(Line::from(team_spans)), rows[0]);

    // Profile row
    let profile_spans: Vec<Span> = std::iter::once(Span::styled("  Profile: ", Style::default().fg(Color::White)))
        .chain(app.profile_options.iter().enumerate().map(|(i, name)| {
            if i == app.profile_index {
                let style = if app.focus == Focus::Profile {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                };
                Span::styled(format!(" {name} "), style)
            } else {
                Span::styled(format!(" {name} "), Style::default().fg(Color::DarkGray))
            }
        }))
        .collect();
    frame.render_widget(Paragraph::new(Line::from(profile_spans)), rows[1]);
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let preview_text = build_preview(app);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Prompt Preview ");
    let para = Paragraph::new(preview_text)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::styled(" Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": next  "),
        Span::styled("Shift+Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": prev  "),
        Span::styled("Ctrl+Enter", Style::default().fg(Color::Green)),
        Span::raw(": launch  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(": quit"),
    ];

    if let Some(ref msg) = app.error_message {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(msg, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn build_preview(app: &App) -> String {
    let desc = app.description();
    if desc.trim().is_empty() {
        return "(type a task description above)".to_string();
    }

    let mut preview = String::new();
    match app.selected_team() {
        Some(team) => {
            preview.push_str(&format!("Mode: team ({})\n", team));
            preview.push_str(&format!("Profile: {}\n", app.selected_profile()));
        }
        None => {
            preview.push_str("Mode: solo\n");
        }
    }
    preview.push_str(&format!("\nTask:\n{}", desc));
    preview
}
```

**Step 2: Add module to `lib.rs`**

Add `pub mod ui;` to `lib.rs`.

**Step 3: Verify it compiles**

Run: `cargo check -p forja-tui`
Expected: compiles

**Step 4: Commit**

```bash
git add crates/forja-tui/src/ui.rs crates/forja-tui/src/lib.rs
git commit -m "feat(tui): add UI rendering with layout and widgets"
```

---

### Task 6: Implement `launch()` — the main event loop

**Files:**
- Modify: `crates/forja-tui/src/lib.rs`

**Step 1: Implement `launch()` with terminal setup/teardown**

Replace the `todo!()` in `lib.rs`:

```rust
pub mod app;
pub mod input;
pub mod ui;

use std::io;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use forja_core::error::{ForjaError, Result};
use forja_core::models::profile::Profile;
use forja_core::models::state::load_state;
use forja_core::paths::ForjaPaths;

use app::App;

pub struct TaskOutput {
    pub description: String,
    pub team: Option<String>,
    pub profile: Profile,
}

pub fn launch() -> Result<Option<TaskOutput>> {
    // Build team options from state
    let (team_options, _team_names) = load_team_options();
    let profile_options = vec!["fast".to_string(), "balanced".to_string(), "max".to_string()];

    // Default profile index to "balanced" (index 1)
    let mut app = App::new(team_options, profile_options);
    app.profile_index = 1;

    // Terminal setup
    enable_raw_mode().map_err(|e| ForjaError::Dialoguer(format!("raw mode: {e}")))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| ForjaError::Dialoguer(format!("alternate screen: {e}")))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| ForjaError::Dialoguer(format!("terminal: {e}")))?;

    // Main loop
    let result = run_loop(&mut terminal, &mut app);

    // Terminal teardown (always runs)
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result?;

    if app.should_launch {
        let profile: Profile = app
            .selected_profile()
            .parse()
            .map_err(|e: String| ForjaError::Dialoguer(e))?;
        Ok(Some(TaskOutput {
            description: app.description(),
            team: app.selected_team().map(|s| s.to_string()),
            profile,
        }))
    } else {
        Ok(None)
    }
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal
            .draw(|frame| ui::render(frame, app))
            .map_err(|e| ForjaError::Dialoguer(format!("draw: {e}")))?;

        if let Event::Key(key) = event::read().map_err(ForjaError::Io)? {
            input::handle_key(app, key);
        }

        if app.should_quit || app.should_launch {
            break;
        }
    }
    Ok(())
}

fn load_team_options() -> (Vec<String>, Vec<String>) {
    let mut options = vec!["Solo".to_string()];
    let mut names = vec!["solo".to_string()];

    // Preset teams
    let presets = [
        ("quick-fix", "quick-fix (coder + deployer)"),
        ("solo-sprint", "solo-sprint (coder-tester + reviewer)"),
        ("full-product", "full-product (5 agents)"),
    ];

    if let Ok(paths) = ForjaPaths::new() {
        if paths.forja_root.exists() {
            let state = load_state(&paths.state);
            for (name, label) in &presets {
                if !state.teams.contains_key(*name) {
                    options.push(label.to_string());
                    names.push(name.to_string());
                }
            }
            let mut configured: Vec<_> = state.teams.iter().collect();
            configured.sort_by_key(|(n, _)| n.to_string());
            for (name, entry) in configured {
                options.push(format!("{} ({} agents)", name, entry.members.len()));
                names.push(name.clone());
            }
            return (options, names);
        }
    }

    // Fallback: just presets
    for (name, label) in &presets {
        options.push(label.to_string());
        names.push(name.to_string());
    }
    (options, names)
}
```

**Important:** The `selected_team()` method on App returns the label string. We need the actual team name. Store a parallel `team_names` vec in App, or store tuples. The simplest fix: store `team_names: Vec<String>` alongside `team_options` in `App`, and add `pub fn selected_team_name(&self) -> Option<&str>` that indexes into `team_names`.

Update `App::new` to accept `team_names: Vec<String>` too, and update the `selected_team()` to use it:

```rust
pub fn selected_team(&self) -> Option<&str> {
    if self.team_index == 0 {
        None
    } else {
        Some(&self.team_names[self.team_index])
    }
}
```

Update the tests in Task 3 accordingly (pass a matching `team_names` vec).

**Step 2: Verify it compiles**

Run: `cargo check -p forja-tui`
Expected: compiles

**Step 3: Manual test**

Run: `cargo run -p forja-cli -- task`
Expected: TUI opens, shows textarea, team/profile selectors, Esc quits cleanly

**Step 4: Commit**

```bash
git add crates/forja-tui/src/lib.rs crates/forja-tui/src/app.rs
git commit -m "feat(tui): implement launch() event loop with terminal setup/teardown"
```

---

### Task 7: Wire up CLI dispatch and end-to-end test

**Files:**
- Modify: `crates/forja-cli/src/commands/task.rs`

**Step 1: Verify the full flow works**

Run: `cargo run -p forja-cli -- task`
Expected: TUI opens. Type a description, select team/profile, Ctrl+Enter launches Claude.

Run: `cargo run -p forja-cli -- task "inline still works"`
Expected: existing behavior — launches Claude directly.

Run: `cargo run -p forja-cli -- task --print`
Expected: error "print requires a task description"

**Step 2: Run full test suite**

Run: `cargo test --workspace`
Expected: all tests pass

**Step 3: Commit**

```bash
git add crates/forja-cli/src/commands/task.rs
git commit -m "feat(tui): wire up TUI launch in forja task command"
```

---

### Task 8: Polish and edge cases

**Files:**
- Modify: `crates/forja-tui/src/app.rs` (add validation test)
- Modify: `crates/forja-tui/src/lib.rs` (stdin check)

**Step 1: Write test for empty description validation**

```rust
#[test]
fn try_launch_rejects_empty_description() {
    let mut app = App::new(vec!["Solo".into()], vec!["Solo".into()], vec!["balanced".into()]);
    app.try_launch();
    assert!(!app.should_launch);
    assert!(app.error_message.is_some());
}

#[test]
fn try_launch_accepts_non_empty_description() {
    let mut app = App::new(vec!["Solo".into()], vec!["Solo".into()], vec!["balanced".into()]);
    app.textarea.insert_str("fix the login bug");
    app.try_launch();
    assert!(app.should_launch);
    assert!(app.error_message.is_none());
}
```

**Step 2: Run tests**

Run: `cargo test -p forja-tui`
Expected: PASS

**Step 3: Add stdin TTY check in `launch()`**

At the top of `launch()`, before entering raw mode:

```rust
use std::io::IsTerminal;

if !io::stdin().is_terminal() {
    return Err(ForjaError::Dialoguer(
        "TUI requires an interactive terminal. Use: forja task \"description\"".to_string(),
    ));
}
```

**Step 4: Final full test**

Run: `cargo test --workspace`
Expected: all tests pass

**Step 5: Commit**

```bash
git add crates/forja-tui/
git commit -m "feat(tui): add validation, stdin check, and polish"
```
