// Config module exports
// Contains configuration components for mode capabilities and parameters

pub mod capabilities;
pub mod mode_parameters;
pub mod config;

// Re-export key components
pub use capabilities::{
    WithTitle, WithCustomSize, WithEmoji, WithTitleAndEmoji,
    StandardWindow, WithWrappedText, WithProgress
};
pub use mode_parameters::{ThreadMode, ModeParameters};
pub use config::Config; 