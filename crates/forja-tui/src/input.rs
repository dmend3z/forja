use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Focus};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Global keys (work in any focus mode)
    match key.code {
        KeyCode::Esc => {
            app.should_quit = true;
            return;
        }
        KeyCode::BackTab => {
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
        // macOS terminals often send Ctrl+Enter as Ctrl+J (linefeed = ASCII 10)
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.try_launch();
            return;
        }
        _ => {}
    }

    // Focus-specific keys
    match app.focus {
        Focus::Textarea => {
            // Clear error when user starts typing
            app.error_message = None;
            app.textarea.input(key);
        }
        Focus::Team | Focus::Profile => match key.code {
            KeyCode::Left | KeyCode::Char('h') => app.select_prev(),
            KeyCode::Right | KeyCode::Char('l') => app.select_next(),
            KeyCode::Enter => app.try_launch(),
            _ => {}
        },
        Focus::Submit => {
            if key.code == KeyCode::Enter {
                app.try_launch();
            }
        }
    }
}
