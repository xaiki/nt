// Core module exports
// Contains foundational components for progress tracking and job management

pub mod base_config;
pub mod job_traits;
pub mod thread_config;
pub mod job_statistics;

// Re-export key components
pub use base_config::BaseConfig;
pub use job_traits::{
    HasBaseConfig, JobTracker, PausableJob, HierarchicalJobTracker,
    PrioritizedJob, DependentJob
};
pub use thread_config::{ThreadConfig, ThreadConfigExt}; 