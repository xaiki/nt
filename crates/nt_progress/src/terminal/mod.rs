//! Terminal handling module
//! 
//! This module provides abstractions for terminal operations, including:
//! - Terminal size detection
//! - Cursor positioning and movement
//! - Terminal feature detection
//! - Style management
//! - Event handling

mod size;
mod cursor;
mod style;
mod test_env;

pub use size::Terminal;
pub use cursor::CursorPosition;
pub use test_env::TestEnv;
pub use style::Style;

// Re-export the crossterm colors for convenience
pub use crossterm::style::Color; 