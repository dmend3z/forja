use crossterm::event::{KeyCode, KeyEvent};

use crate::scan_app::ScanApp;

pub fn handle_key(app: &mut ScanApp, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Enter => {
            app.confirm();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
        }
        KeyCode::Char(' ') => {
            app.toggle();
        }
        KeyCode::Char('a') => {
            app.select_all();
        }
        KeyCode::Char('n') => {
            app.deselect_all();
        }
        KeyCode::Tab => {
            app.jump_next_group();
        }
        _ => {}
    }
}
