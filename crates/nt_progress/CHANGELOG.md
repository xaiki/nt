# NT Progress Changelog

This file contains completed tasks and improvements moved from the TODO.md file.

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