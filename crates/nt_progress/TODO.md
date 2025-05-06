# NT Progress Enhancements and TODO

## Overview and Current State

The `nt_progress` library provides a flexible and thread-safe progress display for Rust applications, featuring concurrent progress tracking, terminal-aware display, and customizable output modes.

### Completed Tasks

#### Base Structure Refactoring
- [x] Create a `WindowBase` struct that serves as a base class for Window and WindowWithTitle
- [x] Refactor `Window` to use WindowBase for shared functionality
- [x] Refactor `WindowWithTitle` to use WindowBase for shared functionality
- [x] Create a `SingleLineBase` struct that serves as a base for Limited and Capturing modes
- [x] Refactor Limited mode to use SingleLineBase
- [x] Refactor Capturing mode to use SingleLineBase

#### Core Architecture
- [x] Create a `JobTracker` trait to handle job counting consistency across implementations
- [x] Enhance BaseConfig for better reuse across different modes
- [x] Standardize method signatures
- [x] Create a TestBuilder utility to simplify test creation
- [x] Add standard testing utilities for common mode assertions
- [x] Standardize Thread Configuration implementation patterns
- [x] Create consistent method documentation

#### Thread Configuration Interface
- [x] Define minimal trait interface for ThreadConfig
- [x] Create wrapper struct `Config` for mode implementations
- [x] Implement Clone via `clone_box`
- [x] Add delegation methods to underlying implementation

#### Mode Implementations
- [x] Implement basic LimitedConfig with ThreadConfig trait
- [x] Implement basic CapturingConfig with ThreadConfig trait
- [x] Implement basic WindowConfig with ThreadConfig trait
- [x] Implement basic WindowWithTitleConfig with ThreadConfig trait

#### Documentation
- [x] Add documentation for existing modes
- [x] Document common patterns for implementing new modes
- [x] Create a README.md in the modes directory explaining the system design
- [x] Document standard patterns for implementing new modes
- [x] Provide example code for custom mode implementation
- [x] Document feature sets for each mode
- [x] Provide usage examples for each mode

## Remaining Tasks

### Error Handling and Robustness
- [x] Implement a better error handling mechanism for mode creation
- [x] Add detailed error types
- [x] Implement error recovery strategies
- [x] Add context-aware logging for debugging

### Unimplemented Features
- [x] Implement WindowWithTitle mode functionality (`set_title` method in ProgressDisplay)
- [x] Implement total jobs support (`set_total_jobs` method in ProgressDisplay)
- [ ] Add emoji support (`add_emoji` method in ProgressDisplay)
- [ ] Implement direct writer functionality for TaskHandle (currently unused `writer` field)
- [ ] Implement output passthrough functionality (currently unused `passthrough` field and `has_passthrough` method in SingleLineBase)
- [ ] Standardize thread config creation (replace Config::new with more robust `create_thread_config` function)
- [ ] Add terminal size customization in TestBuilder (currently unused `width` and `height` fields)

### Optimization Opportunities
- [ ] Reduce duplication in string handling
- [ ] Consider memory usage optimizations in window handling
- [x] Fix unused mutable variables in tests
- [ ] Implement message batching
- [ ] Optimize buffer operations
- [ ] Consider message compression for high-volume scenarios

### Display Features
- [ ] Add color support for highlighting
- [ ] Implement line wrapping for long messages
- [ ] Add custom progress indicators
- [ ] Implement interactive progress bars
- [ ] Support ANSI escape sequences for advanced terminal operations

### Terminal Handling
- [ ] Move terminal size adjustment to separate module
- [ ] Add terminal size change detection
- [ ] Implement automatic terminal resize handling
- [ ] Fix failing terminal tests

### Testing Improvements
- [ ] Add more extensive unit tests for each mode
- [ ] Add integration tests for mode switching
- [ ] Improve tests for terminal size changes
- [ ] Add stress tests for concurrent usage
- [x] Add tests for error handling and recovery

### Documentation Improvements
- [ ] Add examples for typical use cases
- [ ] Create comprehensive API documentation
- [ ] Document thread safety guarantees
- [ ] Add examples for mode switching
- [ ] Update documentation to reflect latest features

### Code Duplication Reduction
- [x] Create a `HasBaseConfig` trait with blanket implementations for `JobTracker`
- [x] Implement generic downcast methods for Config instead of type-specific ones
- [x] Refactor error context addition to reduce boilerplate
- [x] Standardize access patterns for mode-specific features through capability traits
- [x] Implement a factory pattern with registry for mode creation
- [x] Fix factory-mode layering violation by moving fallback logic to mode creators
- [ ] Extract common terminal size adjustment logic to a shared module
- [x] Create composable components for message formatting and rendering
- [x] Implement templating pattern for task progress reporting
- [ ] Separate mode-specific functionality from the Config wrapper to reduce coupling
- [ ] Implement a proper separation between thread management and mode implementation

### Architectural Improvements
- [ ] Create a dedicated Terminal module to encapsulate all terminal-related functionality
- [ ] Implement a proper I/O abstraction to decouple TaskHandle from direct writer references
- [ ] Establish a clear separation between UI rendering logic and progress tracking logic
- [ ] Apply the Interface Segregation Principle to split large interfaces into smaller, more focused ones
- [ ] Implement a proper event system to decouple error propagation from direct function calls
- [ ] Remove terminal size detection from mode implementations and move it to a dedicated service
- [ ] Extract testing utilities into a separate module that doesn't pollute production code
- [ ] Implement a proper dependency injection system for mode creation instead of static registry
- [ ] Separate configuration from implementations to follow the Dependency Inversion Principle
- [ ] Replace static, mutable state (like REGISTRY) with a more testable and maintainable design

### Job Tracking Enhancements
- [x] Update `Config::set_total_jobs` method to use the trait system instead of manual downcasting
- [ ] Add percentage calculation and display for job progress
- [ ] Implement progress bar visualization for job completion
- [ ] Add support for nested/hierarchical job tracking
- [ ] Implement pause/resume functionality for jobs
- [ ] Add job priority and sorting capabilities
- [ ] Create job dependencies system
- [ ] Implement job failure handling and retry logic
- [ ] Add estimated time remaining calculations
- [ ] Support job cancellation
- [ ] Implement job statistics and reporting
- [ ] Add job persistence for long-running operations

## Implementation Priority

1. Fix warnings and linter issues
   - [ ] Fix the unused `writer` field in TaskHandle (src/lib.rs:487)
   - [ ] Address shared reference to mutable static in factory.rs (src/modes/factory.rs:245)
   - [ ] Fix unused `width` and `height` fields in TestBuilder (src/tests/test_builder.rs:14-16)
   - [ ] Clean up unused imports in terminal.rs tests (tests/terminal.rs:3-4)
   - [ ] Address unused mutable variable in terminal.rs (tests/terminal.rs:73)
2. Fix failing terminal integration tests
   - [ ] Fix test_basic_terminal_output (tests/terminal.rs:17)
   - [ ] Fix test_terminal_operations (tests/terminal.rs:68)
   - [ ] Fix test_terminal_state (tests/terminal.rs:34)
3. Implement remaining features
4. Reduce code duplication through refactoring patterns
5. Add job tracking enhancements
6. Improve terminal handling
7. Add optimization techniques
8. Enhance display features
9. Update documentation

## Development Guidelines

- Avoid introducing new linter errors
- Minimize warnings
- Avoid code duplication
- Maintain test coverage
- Keep documentation updated
- Use descriptive commit messages 