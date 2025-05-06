# NT Progress Enhancements and Roadmap

## Overview

The `nt_progress` library provides a flexible and thread-safe progress display for Rust applications, featuring concurrent progress tracking, terminal-aware display, and customizable output modes.

## Development Roadmap

### Phase 0: Immediate Fixes (Completed) ✓
- [x] Fix failing terminal integration tests
  - [x] Fix coordinate system in cursor positioning (x,y swapped in tests/terminal.rs:34)
  - [x] Fix text overwriting in basic terminal test (tests/terminal.rs:17)
  - [x] Fix cursor position tracking after write operations (tests/terminal.rs:68)

### Phase 1: Capability System Improvements (Completed) ✓
- [x] Complete WithEmoji trait implementation for WindowWithTitle mode
- [x] Add unit tests for capability traits
- [x] Create composite capabilities
  - [x] WithTitleAndEmoji trait for combined functionality
  - [x] StandardWindow trait for common window operations
- [x] Add capability discovery API with runtime capability checking

### Phase 2: Terminal Module Refactoring
- [ ] Create a dedicated terminal module
  - [ ] Move terminal size detection to Terminal struct
  - [ ] Implement terminal feature detection
  - [ ] Add terminal style management
- [ ] Implement cursor handling abstractions
  - [ ] Create CursorPosition type
  - [ ] Add movement operations
  - [ ] Support relative and absolute positioning
- [ ] Add terminal event system
  - [ ] Implement resize event handling
  - [ ] Add keyboard input handling for interactive modes
  - [ ] Support terminal capability detection
- [ ] Improve TestEnv for terminal testing
  - [ ] Add screen buffer dumping for debugging
  - [ ] Add string diff utility for test failures
  - [ ] Implement expected vs actual comparison helper

### Phase 3: Mode Factory and Dependency Injection
- [ ] Replace static registry with dependency injection
  - [ ] Create ModeFactory struct to replace static REGISTRY
  - [ ] Add factory creation method to ProgressDisplay
  - [ ] Implement factory cloning without static references
  - [ ] Replace direct type checking with capability-based registration
- [ ] Improve mode creation error handling
  - [ ] Add more detailed failure reasons
  - [ ] Implement validation before creation attempts
  - [ ] Add logging for creation failures
- [ ] Standardize mode parameters
  - [ ] Create ModeParameters type for consistent creation
  - [ ] Add validation methods for parameters
  - [ ] Support default parameter values

### Phase 4: Thread Management Refactoring
- [ ] Separate thread management from mode implementation
  - [ ] Create ThreadManager struct for thread tracking
  - [ ] Move thread ID generation to ThreadManager
  - [ ] Implement thread resource cleanup
- [ ] Implement thread context
  - [ ] Add ThreadContext for storing thread-specific data
  - [ ] Support context propagation between components
  - [ ] Add context serialization for debugging
- [ ] Add thread lifecycle management
  - [ ] Support thread pausing/resuming
  - [ ] Add thread completion notification
  - [ ] Implement graceful thread termination

### Phase 5: I/O Abstraction Layer
- [ ] Create I/O abstraction for TaskHandle
  - [ ] Implement trait-based writer interface
  - [ ] Add buffer management
  - [ ] Support async I/O operations
- [ ] Add passthrough functionality to SingleLineBase
  - [ ] Implement output tee functionality
  - [ ] Support filtering of passed-through content
  - [ ] Add optional formatting for passed-through text
- [ ] Implement custom writer support
  - [ ] Add pluggable writer system
  - [ ] Support custom formatters
  - [ ] Implement output redirection

### Phase 6: Layering Violation Fixes
- [ ] Implement capability registration system
  - [ ] Create a CapabilityRegistry to track which types implement which capabilities
  - [ ] Add runtime capability registration during type creation
  - [ ] Replace direct type checking with registry lookups
- [ ] Refactor ThreadConfigExt trait
  - [ ] Remove direct type references (Window, WindowWithTitle)
  - [ ] Implement capability resolution through type-erased registry
  - [ ] Add dynamic capability discovery without hardcoded types
- [ ] Fix Config implementation
  - [ ] Replace explicit type list with dynamic capability lookup
  - [ ] Use HasBaseConfig trait directly without type checking
  - [ ] Add a generic mechanism for capability-based dispatch
- [ ] Implement proper layering between traits and implementations
  - [ ] Move type-specific code out of shared traits
  - [ ] Create proper abstraction boundaries between layers
  - [ ] Ensure high-level modules don't depend on low-level implementations

### Phase 7: Feature Enhancements
- [ ] Implement remaining core features
  - [ ] Direct writer functionality for TaskHandle
  - [ ] Output passthrough functionality
  - [ ] Terminal size customization in TestBuilder
- [ ] Add display enhancements
  - [ ] Color support for highlighting
  - [ ] Line wrapping for long messages
  - [ ] Custom progress indicators
  - [ ] Interactive progress bars
  - [ ] ANSI escape sequence support
- [ ] Expand job tracking capabilities
  - [ ] Percentage calculation and display
  - [ ] Progress bar visualization
  - [ ] Nested/hierarchical job tracking
  - [ ] Pause/resume functionality
  - [ ] Job priority and sorting
  - [ ] Job dependencies system
  - [ ] Failure handling and retry logic
  - [ ] Estimated time remaining calculations
  - [ ] Job cancellation
  - [ ] Job statistics and reporting
  - [ ] Job persistence for long-running operations

### Phase 8: Optimization and Polish
- [ ] Performance optimizations
  - [ ] Reduce string duplication
  - [ ] Optimize memory usage in window handling
  - [ ] Implement message batching
  - [ ] Optimize buffer operations
  - [ ] Consider message compression for high-volume scenarios
- [ ] Testing improvements
  - [ ] Add more extensive unit tests for each mode
  - [ ] Add integration tests for mode switching
  - [ ] Improve tests for terminal size changes
  - [ ] Add stress tests for concurrent usage
- [ ] Documentation improvements
  - [ ] Add examples for typical use cases
  - [ ] Create comprehensive API documentation
  - [ ] Document thread safety guarantees
  - [ ] Add examples for mode switching
  - [ ] Update documentation to reflect latest features

## Architectural Design Notes

### Capability System Design

The capability system should be refactored to use a proper registration system rather than hard-coded type checks:

```rust
// Type-erased capability resolution
trait AnyExt {
    fn as_capability<C: ?Sized + 'static>(&self) -> Option<&C>;
    fn as_capability_mut<C: ?Sized + 'static>(&mut self) -> Option<&mut C>;
}

impl<T: 'static> AnyExt for T {
    fn as_capability<C: ?Sized + 'static>(&self) -> Option<&C> {
        // First check if T directly implements C
        if let Some(cap) = (self as &dyn Any).downcast_ref::<C>() {
            return Some(cap);
        }
        
        // Then check if T provides C through CapabilityProvider
        if let Some(provider) = (self as &dyn Any).downcast_ref::<dyn CapabilityProvider>() {
            return provider.get_capability::<C>();
        }
        
        None
    }
}
```

### Mode Factory Design

Replace static registry with dependency injection:

```rust
// Factory that's passed explicitly to components that need it
struct ModeFactory {
    registry: Arc<RwLock<ModeRegistry>>,
}

impl ModeFactory {
    fn new() -> Self {
        let mut registry = ModeRegistry::new();
        // Register standard modes...
        Self { registry: Arc::new(RwLock::new(registry)) }
    }
}

// Pass factory explicitly to ProgressDisplay
impl ProgressDisplay {
    pub fn new_with_factory(factory: Arc<ModeFactory>) -> Self {
        // Use provided factory
    }
}
```

### Current Layering Issues

#### ThreadConfigExt Implementation

Current problematic code:
```rust
fn supports_title(&self) -> bool {
    self.as_any().type_id() == TypeId::of::<WindowWithTitle>()
}

fn as_title(&self) -> Option<&dyn WithTitle> {
    self.as_any().downcast_ref::<WindowWithTitle>().map(|w| w as &dyn WithTitle)
}
```

#### Config::set_total_jobs Implementation

Current problematic code:
```rust
// Try each known implementation
try_as_has_base_config!(Window);
try_as_has_base_config!(WindowWithTitle);
try_as_has_base_config!(Limited);
try_as_has_base_config!(Capturing);
```

## Completed Tasks Archive

### Base Structure Refactoring
- [x] Create a `WindowBase` struct that serves as a base class for Window and WindowWithTitle
- [x] Refactor `Window` to use WindowBase for shared functionality
- [x] Refactor `WindowWithTitle` to use WindowBase for shared functionality
- [x] Create a `SingleLineBase` struct that serves as a base for Limited and Capturing modes
- [x] Refactor Limited mode to use SingleLineBase
- [x] Refactor Capturing mode to use SingleLineBase

### Core Architecture
- [x] Create a `JobTracker` trait to handle job counting consistency across implementations
- [x] Enhance BaseConfig for better reuse across different modes
- [x] Standardize method signatures
- [x] Create a TestBuilder utility to simplify test creation
- [x] Add standard testing utilities for common mode assertions
- [x] Standardize Thread Configuration implementation patterns
- [x] Create consistent method documentation

### Thread Configuration Interface
- [x] Define minimal trait interface for ThreadConfig
- [x] Create wrapper struct `Config` for mode implementations
- [x] Implement Clone via `clone_box`
- [x] Add delegation methods to underlying implementation

### Mode Implementations
- [x] Implement basic LimitedConfig with ThreadConfig trait
- [x] Implement basic CapturingConfig with ThreadConfig trait
- [x] Implement basic WindowConfig with ThreadConfig trait
- [x] Implement basic WindowWithTitleConfig with ThreadConfig trait

### Documentation
- [x] Add documentation for existing modes
- [x] Document common patterns for implementing new modes
- [x] Create a README.md in the modes directory explaining the system design
- [x] Document standard patterns for implementing new modes
- [x] Provide example code for custom mode implementation
- [x] Document feature sets for each mode
- [x] Provide usage examples for each mode

### Error Handling and Robustness
- [x] Implement a better error handling mechanism for mode creation
- [x] Add detailed error types
- [x] Implement error recovery strategies
- [x] Add context-aware logging for debugging

### Features
- [x] Implement WindowWithTitle mode functionality (`set_title` method in ProgressDisplay)
- [x] Implement total jobs support (`set_total_jobs` method in ProgressDisplay)
- [x] Add emoji support (`add_emoji` method in ProgressDisplay)

### Code Improvements
- [x] Create a `HasBaseConfig` trait with blanket implementations for `JobTracker`
- [x] Implement generic downcast methods for Config instead of type-specific ones
- [x] Refactor error context addition to reduce boilerplate
- [x] Standardize access patterns for mode-specific features through capability traits
- [x] Implement a factory pattern with registry for mode creation
- [x] Fix factory-mode layering violation by moving fallback logic to mode creators
- [x] Create composable components for message formatting and rendering
- [x] Implement templating pattern for task progress reporting
- [x] Fix unused mutable variables in tests
- [x] Add tests for error handling and recovery
- [x] Update `Config::set_total_jobs` method to use the trait system instead of manual downcasting

## Development Guidelines

- Avoid introducing new linter errors
- Minimize warnings
- Avoid code duplication
- Maintain test coverage
- Keep documentation updated
- Use descriptive commit messages 