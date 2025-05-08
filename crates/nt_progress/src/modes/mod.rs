// Re-export public items from the modes module

pub mod base_config;
pub mod capabilities;
pub mod config;
pub mod job_traits;
pub mod mode_parameters;
pub mod thread_config;
pub mod window_base;

// Keep the existing modules
pub mod limited;
pub mod capturing;
pub mod window;
pub mod window_with_title;
pub mod factory;

// Re-export types for public API
pub use base_config::BaseConfig;
pub use capabilities::{
    Capability, 
    WithTitle, WithCustomSize, WithEmoji, WithTitleAndEmoji,
    StandardWindow, WithWrappedText, WithProgress
};
pub use config::Config;
pub use job_traits::{
    HasBaseConfig, JobTracker, PausableJob, HierarchicalJobTracker,
    PrioritizedJob, DependentJob
};
pub use mode_parameters::{ThreadMode, ModeParameters};
pub use thread_config::{ThreadConfig, ThreadConfigExt};
pub use window_base::{WindowBase, SingleLineBase, WithPassthrough};

// Re-export concrete mode implementations
pub use limited::Limited;
pub use capturing::Capturing;
pub use window::Window;
pub use window_with_title::WindowWithTitle;
pub use factory::{ModeRegistry, ModeCreator, ModeFactory, set_error_propagation};