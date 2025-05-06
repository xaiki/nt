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
- [x] Add emoji support (`add_emoji` method in ProgressDisplay)
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
- [ ] Fix layering violations in modes/mod.rs:
  - [ ] Replace direct type checking in ThreadConfigExt with capability registration system
  - [ ] Implement a capability provider pattern instead of hard-coded type checks
  - [ ] Create a dynamic dispatch system for capability resolution
  - [ ] Remove explicit dependencies on concrete types in trait extension methods
  - [ ] Use trait objects instead of specific implementation types in downcast operations

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

## Detailed Refactoring Roadmap

### Phase 0: Immediate Warning Fixes
- [ ] Fix the unused `writer` field in TaskHandle (src/lib.rs:487)
- [ ] Address shared reference to mutable static in factory.rs (src/modes/factory.rs:245)
- [ ] Fix unused `width` and `height` fields in TestBuilder (src/tests/test_builder.rs:14-16)
- [ ] Clean up unused imports in terminal.rs tests (tests/terminal.rs:3-4)
- [ ] Address unused mutable variable in terminal.rs (tests/terminal.rs:73)

### Phase 1: Capability System Improvements
- [x] Complete WithEmoji trait implementation for WindowWithTitle mode
  - [x] Add emoji container to WindowWithTitle
  - [x] Implement emoji rendering in display method
  - [x] Add emoji validation and normalization
- [x] Add unit tests for capability traits
  - [x] Test WithTitle functionality
  - [x] Test WithCustomSize functionality
  - [x] Test WithEmoji functionality
- [x] Create composite capabilities
  - [x] WithTitleAndEmoji trait for combined functionality
  - [x] StandardWindow trait for common window operations
- [x] Add capability discovery API
  - [x] Create a capabilities() method returning HashSet of supported capabilities
  - [x] Add runtime capability checking

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
  - [ ] Add keyboard input handling (for interactive modes)
  - [ ] Support terminal capability detection
- [ ] Fix terminal integration tests
  - [ ] Fix coordinate system in cursor positioning (x,y swapped in tests/terminal.rs:34)
  - [ ] Fix text overwriting in basic terminal test (tests/terminal.rs:17)
    - [ ] Investigate cursor movement before writing text
    - [ ] Fix output buffer handling for moved cursor positions
  - [ ] Fix cursor position tracking after write operations (tests/terminal.rs:68)
    - [ ] Ensure cursor advances correctly after writing text
    - [ ] Add proper cursor position calculation with ANSI sequences
  - [ ] Add more robust TestEnv methods for validation
    - [ ] Add screen buffer dumping for debugging
    - [ ] Add string diff utility for test failures
    - [ ] Implement expected vs actual comparison helper

### Phase 3: Mode Factory Improvements
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
- [ ] Fix Config::set_total_jobs implementation
  - [ ] Replace explicit type list with dynamic capability lookup
  - [ ] Use HasBaseConfig trait directly without type checking
  - [ ] Add a generic mechanism for capability-based dispatch
- [ ] Implement proper layering between traits and implementations
  - [ ] Move type-specific code out of shared traits
  - [ ] Create proper abstraction boundaries between layers
  - [ ] Ensure high-level modules don't depend on low-level implementations

### Specific Layering Issues to Address

#### ThreadConfigExt Implementation

Current problematic code in ThreadConfigExt:
```rust
fn supports_title(&self) -> bool {
    self.as_any().type_id() == TypeId::of::<WindowWithTitle>()
}

fn as_title(&self) -> Option<&dyn WithTitle> {
    // WindowWithTitle is currently the only implementation of WithTitle
    self.as_any().downcast_ref::<WindowWithTitle>().map(|w| w as &dyn WithTitle)
}
```

Proposed solution:
```rust
fn supports_title(&self) -> bool {
    // Check if this type is registered as supporting WithTitle capability
    CAPABILITY_REGISTRY.has_capability::<dyn WithTitle>(self.type_id())
}

fn as_title(&self) -> Option<&dyn WithTitle> {
    // Get as trait object without knowing concrete type
    self.as_any().as_capability::<dyn WithTitle>()
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

Proposed solution:
```rust
// Direct dynamic dispatch without knowing concrete types
if let Some(has_base) = self.config.as_any_mut().as_capability::<dyn HasBaseConfig>() {
    has_base.set_total_jobs(total);
}
```

#### Capability Checking in Factory Module

Current problematic pattern:
```rust
// Each method must know about all concrete types
match mode {
    ThreadMode::Limited => self.create("limited", total_jobs, &[]),
    ThreadMode::Capturing => self.create("capturing", total_jobs, &[]),
    ThreadMode::Window(max_lines) => self.create("window", total_jobs, &[max_lines]),
    ThreadMode::WindowWithTitle(max_lines) => self.create("window_with_title", total_jobs, &[max_lines]),
}
```

Proposed solution:
```rust
// Mode registration that's extensible
fn register_mode<T: ThreadConfig + 'static>(&mut self, name: &str, creator: Box<dyn ModeCreator<T>>) {
    self.creators.insert(name.to_string(), creator);
}

// Mode lookup without explicit dependencies on specific types
fn create_from_mode_type(&self, mode_type: &str, params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
    self.creators.get(mode_type)
        .ok_or_else(|| ModeCreationError::UnknownMode(mode_type.to_string()))
        .and_then(|creator| creator.create(params))
}
```

#### Static Registry Issue

Current problematic code:
```rust
// Unsafe static mutable state with singleton pattern
static mut REGISTRY: Option<Arc<Mutex<ModeRegistry>>> = None;
static REGISTRY_INIT: Once = Once::new();

pub fn get_registry() -> Arc<Mutex<ModeRegistry>> {
    unsafe {
        REGISTRY_INIT.call_once(|| {
            // Initialize registry...
            REGISTRY = Some(Arc::new(Mutex::new(registry)));
        });
        
        REGISTRY.clone().unwrap()
    }
}
```

Proposed solution:
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
    
    fn create_config(&self, mode: ThreadMode, total_jobs: usize) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        // Create config without static state
    }
}

// Pass factory explicitly to ProgressDisplay
impl ProgressDisplay {
    pub fn new_with_factory(factory: Arc<ModeFactory>) -> Self {
        // Use provided factory
    }
}
```

### Comprehensive Architectural Solution

To properly address the layering violations, a more holistic architectural refactoring is needed:

1. **Create a Proper Capability System**:
   - Define a CapabilityId type that uniquely identifies capabilities (could use TypeId)
   - Implement a CapabilityProvider trait that any type can implement to provide capabilities
   - Create a CapabilityRegistry that tracks which types implement which capabilities
   - Implement registration methods for types to declare their capabilities

2. **Implement Type-Erased Capability Resolution**:
   ```rust
   trait AnyExt {
       // Get a capability from this object without knowing its concrete type
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
       
       // Similar implementation for as_capability_mut
   }
   ```

3. **Replace Static Registry with Dependency Injection**:
   - Create a proper ModeFactory that is instantiated and passed to components
   - Implement builder pattern for ProgressDisplay to configure with different factories
   - Remove all static state from the factory implementation
   - Allow for factory customization and extension

4. **Refactor ThreadConfig and ThreadConfigExt**:
   - Remove all direct references to concrete types
   - Replace type checking with capability checking
   - Make trait extensions use the type-erased capability system
   - Ensure no trait method needs to know about concrete implementations

5. **Implement Proper Plugin System for Modes**:
   - Allow registering new modes without modifying existing code
   - Modes register their capabilities during registration
   - Factory creates modes based on capabilities, not types
   - Enable composition of capabilities without hard dependencies

This approach would provide:
- Proper separation of concerns between capability definition and implementation
- Elimination of layering violations where high-level modules depend on low-level ones
- A more extensible and maintainable architecture
- Better testability by removing static state and dependencies on concrete types

## Development Guidelines

- Avoid introducing new linter errors
- Minimize warnings
- Avoid code duplication
- Maintain test coverage
- Keep documentation updated
- Use descriptive commit messages 