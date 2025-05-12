# NT Progress Changelog

This file contains completed tasks and improvements moved from the TODO.md file.

## v0.1.42 (2025-05-12)

### Features
- Added support for multiple progress bars
  - Implemented MultiProgressBar struct for managing groups of progress bars
  - Added methods to ProgressManager for handling multi-progress bars
  - Added convenience methods to ProgressDisplay for working with multi-progress bars
  - Created tests for the new functionality
  - Can display multiple bars with different styles and progress in a single view

## v0.1.41 (2025-05-12)

### Bug Fixes
- Fixed failing tests in formatter module
  - Corrected ratio format implementation to properly handle context variables
  - Fixed padding formats to correctly apply padding with requested width
  - Fixed color formatting to properly retrieve color names from format parts
  - Improved tests to verify formatted output against expected format

## v0.1.40 (2025-05-11)

### Added Job Persistence for Long-Running Operations
- Implemented job persistence capabilities for long-running operations
  - Added PersistentJob trait for saving and loading job state
  - Created JobState serializable structure using serde for JSON serialization
  - Added persistence_id and storage functionality to BaseConfig
  - Implemented methods to save and load job state to/from files
  - Added comprehensive tests for job persistence functionality
  - Maintained backward compatibility with existing job tracking systems
  - Added generic implementation for all types that implement HasBaseConfig

## v0.1.39 (2025-05-10)

### Added Job Statistics and Reporting
- Implemented comprehensive job statistics and reporting
  - Created `JobStatisticsReport` struct for capturing job statistics
  - Added the `JobStatistics` trait for generating reports and summaries
  - Integrated with existing job tracking infrastructure 
  - Fixed test cases to properly verify job status
  - Consolidated time tracking code to remove duplication
  - Implemented the `WithProgress` trait for `BaseConfig`
  - Properly formatted display of job statistics and summaries

## v0.1.38 (2025-05-09)

### Time Tracking Enhancement
- Fixed time tracking in progress display
  - Added `reset_start_time` method to BaseConfig for better time tracking
  - Implemented Window mode's `get_elapsed_time` to use BaseConfig directly
  - Ensured elapsed time is properly tracked from the first progress update
  - Added comprehensive unit tests for time estimation functionality
  - Fixed integration tests to handle elapsed time correctly

## v0.1.37 (2025-05-08)

### Benchmark and Code Cleanup
- Fixed ThreadMode import path in benchmark file
  - Updated benches/terminal.rs to use the correct ThreadMode import from config module
  - Refactored benchmark code to use current Criterion API for async benchmarks
  - Eliminated compilation errors in the benchmark code
  - Removed references to deprecated APIs and methods
- Verified all tests are passing and no dead code remains
- Ensured proper error handling in all benchmarks

## v0.1.36 (2024-06-02)

### Bug Fixes and Code Cleanup
- Fixed retry limit handling in job status tracking
  - Reset retry counter when a job is marked as completed
  - Ensures retry limits are correctly applied and cleared when jobs finish
  - Passes all test cases for retry management
- Removed unused `start_time_secs` field from BaseConfig
  - Eliminated dead code warning
  - Simplified the time tracking implementation

## v0.1.35 (2024-06-02)

### Code Cleanup and Refactoring
- Fixed Clippy warning in BaseConfig implementation
  - Removed unnecessary let binding in `retry()` method
  - Improved code readability with direct return
  - Simplified control flow in retry handling
  - Fixed all Clippy warnings in the codebase

## v0.1.34 (2024-06-01)

### Architecture Improvements
- Implemented HasBaseConfig trait for Config struct
  - Added proper delegation to underlying mode implementations
  - Enabled direct use of JobStatusTracker and FailureHandlingJob capabilities with Config
  - Fixed integration tests for job status monitoring
  - Improved API consistency across the codebase
  - Removed need for excessive type casting

## v0.1.33 (2024-06-01)

### Added Features
- Implemented failure handling and retry logic
  - Added `FailureHandlingJob` trait for tracking failures and retries
  - Added methods to mark jobs as failed, get error messages, and handle retries
  - Added configurable retry limits with `set_max_retries` and `has_reached_retry_limit`
  - Added comprehensive test coverage for failure handling functionality
- Added job status tracking capabilities
  - Created `JobStatus` enum with Pending, Running, Completed, Failed, and Retry states
  - Added `JobStatusTracker` trait for status tracking and reporting
  - Implemented methods to check and update job status
  - Added helpers to query job state (is_running, is_completed, etc.)
  - Added comprehensive test coverage for status tracking

## v0.1.32 (2024-05-31)

### Code Cleanup and Refactoring
- Fixed Clippy warnings throughout the codebase
  - Replaced `or_insert_with(VecDeque::new)` with `or_default()` in window_base.rs
  - Fixed module inception issue by renaming `config` module to `config_impl`
  - Removed unnecessary `mut` variables in tests
  - Improved code quality and maintainability

## v0.1.31 (2024-05-30)

### Code Cleanup and Refactoring
- Removed dead code in Config implementation
  - Removed unused `as_job_tracker` method that was flagged by the linter
  - Kept the mutable version `as_job_tracker_mut` which is used by several methods
  - Improved code quality by eliminating unused code
  - Fixed all linter warnings in the codebase

## v0.1.29 (2024-05-29)

### Added Features
- Added DependentJob trait for job dependency tracking
  - Implemented add_dependency, remove_dependency, and dependency checking methods
  - Added support for checking if dependencies are satisfied
  - Added comprehensive test coverage for dependency management
  - Integrated with the existing capability system

### Code Cleanup and Refactoring
- Removed dead code by deleting ignored test that was replaced by direct test
  - Removed `test_window_mode_line_wrapping` as it was replaced by `test_window_direct_line_wrapping`
  - Fixed related test harness to no longer show ignored tests
- Fixed Clippy warnings throughout the codebase
  - Replaced boolean equality assertions with more idiomatic `assert!`/`assert!(!)`
  - Removed redundant variable redefinitions in test code
  - Improved code quality and maintainability

## v0.1.28 (2024-05-28)

### Added
- Added public API for pause/resume functionality
  - Added `pause()`, `resume()`, and `is_paused()` methods to TaskHandle
  - Added `pause_thread()`, `resume_thread()`, and `is_thread_paused()` methods to ProgressManager
  - Added `pause_all()` and `resume_all()` methods to ProgressManager
  - Added `pause_thread()`, `resume_thread()`, `is_thread_paused()`, `pause_all()`, and `resume_all()` methods to ProgressDisplay
  - Added proper error handling for all pause/resume operations
  - Added comprehensive tests for pause/resume functionality

## v0.1.27 (2024-05-20)

### Added
- Implemented job priority system
  - Added PrioritizedJob trait for job priority management
  - Implemented ability to set and retrieve job priorities
  - Added priority field to BaseConfig for centralized priority tracking
  - Added proper thread-safe access with atomic counters
  - Extended Config with priority management methods
  - Added comprehensive tests for job prioritization functionality

- Implemented job pause/resume functionality
  - Added PausableJob trait for job pause/resume operations
  - Added pause/resume controls in BaseConfig
  - Implemented thread-safe pausing with atomic flags
  - Added proper API across all job tracking interfaces
  - Added comprehensive tests for pause/resume functionality
  - Extended Config with pause/resume methods and state checking

### Enhanced
- Updated all capability checks to include prioritized and pausable job capabilities
- Implemented these capabilities across all mode types
- Added unit tests to verify functionality in all modes
- Fixed and updated capability test cases to account for new features

## v0.1.26 (2024-05-19)

### Added
- Implemented progress bar visualization
  - Added dedicated ProgressBar module with customizable appearance
  - Created ProgressBarConfig for flexible bar configuration
  - Added support for multiple progress bar styles (Standard, Block, Braille, Dots, Gradient)
  - Implemented ETA and speed calculations for progress tracking
  - Added custom template support for progress bars
  - Added comprehensive test suite for progress bar functionality

### Enhanced
- Improved ProgressManager to support the new progress bar functionality
- Added integration between ProgressBar and existing template system
- Refactored progress tracking to use the new ProgressBar component

## v0.1.25 (2024-05-18)

### Added
- Implemented percentage calculation and display for progress tracking
- Added WithProgress capability trait for supporting progress tracking in display modes
- Added methods to TaskHandle for tracking and displaying progress percentages
- Added comprehensive test suite for progress tracking functionality

### Enhanced
- Extended BaseConfig with progress tracking methods
- Added progress formatting customization
- Improved progress bar display with percentage indicators
- Added support for incremental progress updates

### Fixed
- Fixed an issue with progress calculation for edge cases

## v0.1.24 (2024-05-17)

### Feature Enhancements
- Added ANSI escape sequence support
  - Implemented ANSI escape sequence stripping for text width calculations
  - Added support for CSI and OSC sequence detection and handling
  - Enhanced TextWrapper to account for ANSI escape sequences in line wrapping
  - Fixed visual width calculations to properly handle styled text
  - Added comprehensive tests for ANSI sequence handling

## v0.1.23 (2024-05-16)

### Code Cleanup and Refactoring
- Fixed Clippy warning in CustomIndicatorType implementation
  - Replaced custom from_str method with standard FromStr trait implementation
  - Added proper error type for string parsing failures
  - Improved error handling in format_custom_indicator method
  - Enhanced code consistency with Rust conventions
  - Updated documentation for the FromStr implementation

## v0.1.22 (2024-05-15)

### Feature Enhancements
- Added extensible custom progress indicators
  - Implemented CustomIndicatorType enum for built-in indicator types
  - Added three built-in custom indicators: dots, braille, and gradient
  - Replaced hardcoded indicator names with proper enum-based approach
  - Added comprehensive tests for custom indicators
  - Improved error handling for invalid custom indicators
  - Used a more extensible design for future indicator additions

## v0.1.21 (2024-05-15)

### Code Cleanup and Refactoring
- Fixed Clippy warning for ProgressIndicator by implementing FromStr trait
  - Replaced custom from_str method with standard FromStr trait implementation
  - Added proper error handling for invalid indicator names
  - Added comprehensive tests for FromStr implementation
  - Fixed compiler warnings related to unused variables in tests
  - Improved code quality and type safety

## v0.1.20 (2024-05-15)

### Code Cleanup and Refactoring
- Fixed needless lifetime warning in TextWrapper implementation
  - Removed explicit lifetime annotation in get_next_word method
  - Improved code readability and maintainability
  - Fixed all Clippy warnings in the codebase

## v0.1.19 (2024-05-14)

### Feature Enhancements
- Added line wrapping support for window display modes
  - Implemented TextWrapper utility for wrapping long lines
  - Added WithWrappedText capability trait for window modes
  - Added line wrapping configuration methods to Window and WindowWithTitle modes
  - Added comprehensive tests for line wrapping functionality
  - Properly handles unicode and wide characters in wrapped text

## v0.1.18 (2024-05-14)

### Code Cleanup and Refactoring
- Fixed Clippy warning for ColorName::to_color method
  - Updated method to take self by value since ColorName is Copy
  - Improved method consistency with Rust conventions
  - Fixed all clippy warnings in the codebase
  - Enhanced code quality with better method conventions

## v0.1.17 (2024-05-13)

### Display Enhancements
- Added color support for text highlighting 
  - Implemented color formatting in ProgressTemplate
  - Added {var:color:name} syntax to templates
  - Supported standard terminal colors (black, red, green, yellow, blue, magenta, cyan, white)
  - Added documentation and examples
  - Created ColorName enum for color representation and conversion
  - Added tests for color formatting functionality

## v0.1.16 (2024-05-13)

### Code Cleanup and Refactoring
- Fixed various Clippy warnings throughout the codebase
  - Improved code structure and readability
  - Removed redundant code constructs
  - Used more idiomatic Rust patterns
  - Added proper Default implementations
  - Used appropriate type aliases for complex types
- Enhanced error handling with io::Error::other
- Improved comparison handling with match statements
- Optimized iterator usage with better patterns

## v0.1.15 (2024-05-12)

### Feature Enhancements
- Implemented terminal size customization in TestBuilder
  - Added `resize()` method to change terminal dimensions at runtime
  - Added `terminal_size()` method to retrieve current terminal dimensions
  - Added `test_resize_handling()` method to test resize adaptability
  - Added `test_terminal_size_detection()` method for size detection testing
  - Added comprehensive test for terminal size customization
  - Improved TestBuilder's ability to simulate various terminal environments

## v0.1.14 (2024-05-12)

### Feature Enhancements
- Implemented output passthrough functionality
  - Added `set_passthrough()` method to enable/disable passthrough output
  - Added `has_passthrough()` method to check if passthrough is supported and enabled
  - Added `set_passthrough_writer()` to customize where output is passed through to
  - Added `set_passthrough_filter()` to conditionally pass through messages
  - Added comprehensive test for passthrough functionality in TaskHandle
  - Simplified interaction with passthrough functionality

## v0.1.13 (2024-05-12)

### Feature Enhancements
- Implemented direct writer functionality for TaskHandle
  - Added `writer()` method to get direct access to the underlying writer
  - Added `set_writer()` method to replace the writer with a custom implementation
  - Added `add_tee_writer()` method to create a writer that outputs to multiple destinations
  - Added `with_writer()` method for convenient access to the writer with proper locking
  - Added helper function `new_tee_writer()` for working with boxed writers
  - Added comprehensive test for the new writer functionality

## v0.1.12 (2024-05-12)

### Code Cleanup and Refactoring
- Fixed unused variable warnings in test files
  - Fixed unused variable in `tests/display.rs`
  - Fixed variable naming inconsistencies in `tests/capturing.rs` 
  - Removed unused mutability in `tests/custom_writer.rs`
  - Removed unused mutability in `modes/mod.rs`
- Improved code quality and readability
- Reduced compiler warning noise

## v0.1.11 (2024-05-07)

### I/O Abstraction Layer
- Added custom writer support
  - Created `CustomWriter` trait for pluggable writers
  - Added `WriterCapabilities` for writer feature detection  
  - Implemented `WriterRegistry` for writer management
  - Added comprehensive tests for custom writer functionality
  - Improved writer integration with existing components

### Changed
- Refactored IO module structure for better organization
- Enhanced modularity with pluggable writer system
- Improved error handling in custom writers
- Updated documentation for custom writer functionality
- Added tests for filtering and specialized writer capabilities

## v0.1.10 (2024-03-29)

### I/O Abstraction Layer
- Created I/O abstraction for TaskHandle
  - Added `ProgressWriter` trait for handling both sync and async writes
  - Implemented `OutputBuffer` for line-based buffering
  - Added `TeeWriter` for writing to multiple destinations
  - Updated TaskHandle to use new I/O abstractions
  - Added comprehensive tests for I/O functionality

### Changed
- Improved TaskHandle I/O handling with buffered output
- Enhanced error handling in I/O operations
- Updated documentation for I/O abstractions

## v0.1.9 (2024-03-29)

### Thread Management Refactoring
- Completed thread management separation from mode implementation
  - Created ThreadManager with comprehensive thread lifecycle management
  - Implemented thread pool support with configurable limits
  - Added thread state tracking (Running, Paused, Completed, Failed)
  - Added thread context management for thread-specific data
  - Improved thread resource cleanup and error handling
  - Added comprehensive tests for thread management functionality

### Added
- New ThreadContext struct for managing thread-specific data
- Thread state management with support for multiple states
- Thread pool management with configurable thread limits
- Comprehensive thread lifecycle management
- Thread cleanup and resource management
- Improved error handling for thread operations

### Changed
- Separated thread management from mode implementation
- Enhanced thread safety in job tracking
- Improved thread state transitions and notifications
- Updated documentation for thread management features

## v0.1.7 (2024-03-28)

### Mode Factory and Dependency Injection
- Improved mode creation error handling
  - Added ValidationError variant for pre-creation validation failures
  - Added validation for total_jobs and window sizes before mode creation
  - Enhanced error messages with more detailed failure reasons
  - Added comprehensive tests for validation scenarios

## v0.1.6 (2024-03-28)

### Mode Factory and Dependency Injection
- Replaced static registry with dependency injection
  - Added `ModeFactory::with_registry` for custom registry injection
  - Added `ModeFactory::with_modes` for custom mode registration
  - Updated ProgressDisplay to use factory for mode creation
  - Added From implementation for Config to support factory output

## v0.1.5 (2025-05-07)

### Testing Improvements
- Added `

## [0.1.43] - 2024-03-21

### Code Quality
- Removed duplicate `ThreadConfig` trait definition
- Consolidated thread configuration code in `core` module
- Improved code organization and reduced duplication
- Fixed unused variable warnings in formatter module