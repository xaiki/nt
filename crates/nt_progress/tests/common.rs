// Common test utilities for the integration tests
// This file re-exports and centralizes test utilities to keep imports clean

// Re-export the TestEnv from the terminal module instead of having a duplicate implementation
pub use nt_progress::terminal::TestEnv;

// Also re-export the timeout utility for consistent test patterns
pub use nt_progress::terminal::test_helpers::with_timeout;