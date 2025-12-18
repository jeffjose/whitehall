//! Keyboard input handling for watch mode
//!
//! This module provides non-blocking keyboard input handling for watch mode,
//! allowing users to press keys like 'r' to rebuild or 'q' to quit.

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::io;
use std::time::Duration;

/// Actions that can be triggered by keyboard input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Rebuild/recompile the project
    Rebuild,
    /// Quit watch mode
    Quit,
    /// No action (timeout or unrecognized key)
    None,
}

/// Guard that restores terminal state when dropped
pub struct RawModeGuard {
    enabled: bool,
}

impl RawModeGuard {
    /// Enable raw mode and return a guard that will restore on drop
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self { enabled: true })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.enabled {
            let _ = disable_raw_mode();
        }
    }
}

/// Poll for keyboard input with a timeout
///
/// Returns the action corresponding to the key pressed, or `KeyAction::None`
/// if no key was pressed within the timeout period.
///
/// # Keyboard shortcuts
/// - `r` - Rebuild
/// - `q` - Quit
/// - `Ctrl+C` - Quit
pub fn poll_key(timeout: Duration) -> io::Result<KeyAction> {
    if event::poll(timeout)? {
        if let Event::Key(key_event) = event::read()? {
            return Ok(key_to_action(key_event));
        }
    }
    Ok(KeyAction::None)
}

/// Convert a key event to an action
fn key_to_action(key: KeyEvent) -> KeyAction {
    match key.code {
        KeyCode::Char('r') => KeyAction::Rebuild,
        KeyCode::Char('q') => KeyAction::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Quit,
        KeyCode::Esc => KeyAction::Quit,
        _ => KeyAction::None,
    }
}

/// Print the keyboard shortcuts help message
pub fn print_shortcuts() {
    println!("   Press {} to rebuild, {} to quit", "r", "q");
}

/// Check if we're running in a terminal that supports raw mode
pub fn is_terminal() -> bool {
    terminal::is_raw_mode_enabled().is_ok()
}
