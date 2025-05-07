# NT Progress Enhancements and Roadmap

## Overview

The `nt_progress` library provides a flexible and thread-safe progress display for Rust applications, featuring concurrent progress tracking, terminal-aware display, and customizable output modes.

## Development Roadmap

### Phase 2: Terminal Module Refactoring
- [x] Create a dedicated terminal module
  - [x] Move terminal size detection to Terminal struct
  - [x] Implement terminal feature detection
  - [x] Add terminal style management
- [x] Implement cursor handling abstractions
  - [x] Create CursorPosition type
  - [x] Add movement operations
  - [x] Support relative and absolute positioning
- [x] Add terminal event system
  - [x] Implement resize event handling
  - [x] Add keyboard input handling for interactive modes
  - [x] Support terminal capability detection
- [x] Improve TestEnv for terminal testing
  - [x] Add screen buffer dumping for debugging
  - [x] Add string diff utility for test failures
  - [x] Implement expected vs actual comparison helper

### Phase 3: Mode Factory and Dependency Injection
- [x] Replace static registry with dependency injection
  - [x] Create ModeFactory struct to replace static REGISTRY
  - [x] Add factory creation method to ProgressDisplay
  - [x] Implement factory cloning without static references
  - [x] Replace direct type checking with capability-based registration
- [x] Improve mode creation error handling
  - [x] Add more detailed failure reasons
  - [x] Implement validation before creation attempts
  - [x] Add logging for creation failures
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

## Development Guidelines

- Avoid introducing new linter errors
- Minimize warnings
- Avoid code duplication
- Maintain test coverage
- Keep documentation updated
- Use descriptive commit messages 