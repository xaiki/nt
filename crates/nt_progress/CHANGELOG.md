# NT Progress Changelog

This file contains completed tasks and improvements moved from the TODO.md file.

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
- Added `TestEnv::new_random` to generate random terminal sizes (30–120 width, 10–80 height) to catch hard-coded assumptions

## v0.1.4 (2023-03-28)

### Code Cleanup and Refactoring
- Removed duplicated code in WindowWithTitle implementation
- Moved get_lines and handle_message implementations directly into ThreadConfig trait
- Removed redundant delegation between trait and struct methods
- Improved code organization by removing duplicate functionality

## v0.1.3 (2023-03-27)

### Code Cleanup and Refactoring
- Removed duplicated code in WindowWithTitle implementation
- Simplified trait implementations by delegating to struct methods
- Fixed layering in WithTitle and WithEmoji trait implementations
- Improved code organization in WindowWithTitle struct
- Updated tests to use proper trait imports

## v0.1.2 (2023-03-26)

### Code Cleanup and Refactoring
- Removed duplicate TestEnv implementation from tests/common.rs
- Standardized TestEnv usage across all tests to use the terminal module version
- Fixed test compatibility with the new TestEnv implementation
- Removed old test_utils.rs module as its functionality is now in terminal/test_env.rs
- Updated all tests to use the contents() method instead of directly accessing the expected field
- Created a clean re-export of terminal TestEnv in tests/common.rs

## v0.1.1 (2023-03-25)

### Phase 2: Terminal Module Refactoring (Partial)
- Created a dedicated terminal module with:
  - Terminal struct for size detection and terminal capabilities
  - CursorPosition type for cursor handling abstraction
  - Style struct for terminal style management
  - Improved TestEnv with better diagnostics and comparison tools
- Migrated TestEnv from test_utils to the terminal module
- Added screen buffer dumping for debugging test failures
- Added string diff utility for test failures
- Implemented expected vs actual comparison helper

## Completed Phases

### Phase 0: Immediate Fixes
- Fixed failing terminal integration tests
  - Fixed coordinate system in cursor positioning (x,y swapped in tests/terminal.rs:34)
  - Fixed text overwriting in basic terminal test (tests/terminal.rs:17)
  - Fixed cursor position tracking after write operations (tests/terminal.rs:68)

### Phase 1: Capability System Improvements
- Completed WithEmoji trait implementation for WindowWithTitle mode
- Added unit tests for capability traits
- Created composite capabilities
  - WithTitleAndEmoji trait for combined functionality
  - StandardWindow trait for common window operations
- Added capability discovery API with runtime capability checking

## Other Completed Tasks

### Base Structure Refactoring
- Created a `WindowBase` struct that serves as a base class for Window and WindowWithTitle
- Refactored `Window` to use WindowBase for shared functionality
- Refactored `WindowWithTitle` to use WindowBase for shared functionality
- Created a `SingleLineBase` struct that serves as a base for Limited and Capturing modes
- Refactored Limited mode to use SingleLineBase
- Refactored Capturing mode to use SingleLineBase

### Core Architecture
- Created a `JobTracker` trait to handle job counting consistency across implementations
- Enhanced BaseConfig for better reuse across different modes
- Standardized method signatures
- Created a TestBuilder utility to simplify test creation
- Added standard testing utilities for common mode assertions
- Standardized Thread Configuration implementation patterns
- Created consistent method documentation

### Thread Configuration Interface
- Defined minimal trait interface for ThreadConfig
- Created wrapper struct `Config` for mode implementations
- Implemented Clone via `clone_box`
- Added delegation methods to underlying implementation

### Mode Implementations
- Implemented basic LimitedConfig with ThreadConfig trait
- Implemented basic CapturingConfig with ThreadConfig trait
- Implemented basic WindowConfig with ThreadConfig trait
- Implemented basic WindowWithTitleConfig with ThreadConfig trait

### Documentation
- Added documentation for existing modes
- Documented common patterns for implementing new modes
- Created a README.md in the modes directory explaining the system design
- Documented standard patterns for implementing new modes
- Provided example code for custom mode implementation
- Documented feature sets for each mode
- Provided usage examples for each mode

### Error Handling and Robustness
- Implemented a better error handling mechanism for mode creation
- Added detailed error types
- Implemented error recovery strategies
- Added context-aware logging for debugging

### Features
- Implemented WindowWithTitle mode functionality (`set_title` method in ProgressDisplay)
- Implemented total jobs support (`set_total_jobs` method in ProgressDisplay)
- Added emoji support (`add_emoji` method in ProgressDisplay)

### Code Improvements
- Created a `HasBaseConfig` trait with blanket implementations for `JobTracker`
- Implemented generic downcast methods for Config instead of type-specific ones
- Refactored error context addition to reduce boilerplate
- Standardized access patterns for mode-specific features through capability traits
- Implemented a factory pattern with registry for mode creation
- Fixed factory-mode layering violation by moving fallback logic to mode creators
- Created composable components for message formatting and rendering
- Implemented templating pattern for task progress reporting
- Fixed unused mutable variables in tests
- Added tests for error handling and recovery
- Updated `Config::set_total_jobs` method to use the trait system instead of manual downcasting

## v0.1.8 (2024-03-28)

### Capability System Improvements
- Simplified capability system using direct trait downcasting
  - Removed complex registry approach in favor of direct trait downcasting
  - Updated ThreadConfigExt trait with improved capability checks
  - Added comprehensive tests for capability system
  - Cleaned up unused code and improved error handling
  - All tests passing with improved coverage

## [v0.1.8] - 2024-03-21
### Added
- Enhanced `ModeParameters` with builder pattern for easier parameter construction
- Added comprehensive parameter validation methods
- Added support for default parameter values
- Added new validation methods for required parameters and parameter values

### Changed
- Improved parameter standardization across all modes
- Enhanced error handling for parameter validation
- Updated mode factory to use standardized parameters

### Fixed
- Fixed parameter validation in window modes
- Fixed missing parameter checks in mode creation
- Fixed validation error messages for better clarity

## [v0.1.7] - 2024-03-20
// ... existing code ... 