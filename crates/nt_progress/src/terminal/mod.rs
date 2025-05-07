//! Terminal handling module
//! 
//! This module provides abstractions for terminal operations, including:
//! - Terminal size detection
//! - Cursor positioning and movement
//! - Terminal feature detection
//! - Style management
//! - Event handling
//! - Text manipulation and wrapping

mod size;
mod cursor;
mod style;
mod text;
mod test_env;
mod event;
pub mod test_helpers;

pub use size::Terminal;
pub use cursor::CursorPosition;
pub use test_env::TestEnv;
pub use style::Style;
pub use text::TextWrapper;
pub use event::{EventManager, TerminalEvent, KeyData};
#[cfg(test)]
pub use test_helpers::with_timeout;

// Re-export the crossterm colors for convenience
pub use crossterm::style::Color; 