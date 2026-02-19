//! Terminal input event handling using crossterm.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Actions that the user can trigger from the TUI.
#[derive(Debug, Clone, PartialEq)]
pub enum TuiAction {
    Quit,
    ToggleConnect,         // 'c' — connect/disconnect serial
    ToggleMidi,            // 'm' — connect/disconnect MIDI
    ScrollUp,              // Up arrow
    ScrollDown,            // Down arrow
    ShowHelp,              // '?'
    DismissPopup,          // Escape
    Confirm,               // Enter
    None,
}

/// Poll for a terminal input event with a timeout.
/// Returns None if no event within the timeout.
pub fn poll(timeout: Duration) -> Option<TuiAction> {
    if event::poll(timeout).ok()? {
        if let Event::Key(key) = event::read().ok()? {
            return Some(map_key(key));
        }
    }
    None
}

fn map_key(key: KeyEvent) -> TuiAction {
    // Ctrl+C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return TuiAction::Quit;
    }

    match key.code {
        KeyCode::Char('q') => TuiAction::Quit,
        KeyCode::Char('c') => TuiAction::ToggleConnect,
        KeyCode::Char('m') => TuiAction::ToggleMidi,
        KeyCode::Char('?') => TuiAction::ShowHelp,
        KeyCode::Up => TuiAction::ScrollUp,
        KeyCode::Down => TuiAction::ScrollDown,
        KeyCode::Esc => TuiAction::DismissPopup,
        KeyCode::Enter => TuiAction::Confirm,
        _ => TuiAction::None,
    }
}
