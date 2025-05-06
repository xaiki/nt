# Implementation Plan for NT Progress Enhancements

## Task Overview

### Window Mode Refactoring
- [x] Create a `WindowBase` struct that serves as a base class for Window and WindowWithTitle.
- [x] Refactor `Window` to use WindowBase for shared functionality.
- [x] Refactor `WindowWithTitle` to use WindowBase for shared functionality.

### Single Line Modes Refactoring
- [x] Create a `SingleLineBase` struct that serves as a base for Limited and Capturing modes.
- [x] Refactor Limited mode to use SingleLineBase.
- [x] Refactor Capturing mode to use SingleLineBase.

### BaseConfig Improvements
- [x] Create a `JobTracker` trait to handle job counting consistency across implementations
- [x] Enhance BaseConfig for better reuse across different modes
- [x] Standardize method signatures

### Test Utilities Refactoring
- [x] Create a TestBuilder utility to simplify test creation
- [x] Add standard testing utilities for common mode assertions

### Thread Configuration Interface
- [ ] Standardize Thread Configuration implementation patterns
- [ ] Create consistent method documentation

### Documentation Improvements
- [ ] Add documentation for existing modes
- [ ] Document common patterns for implementing new modes
- [ ] Add examples for typical use cases

### Optimization Opportunities
- [ ] Reduce duplication in string handling
- [ ] Consider memory usage optimizations in window handling

### Unimplemented Features (Future Work)
- [ ] Implement WindowWithTitle mode functionality (`set_title` method in ProgressDisplay)
- [ ] Implement total jobs support (`set_total_jobs` method in ProgressDisplay)
- [ ] Add emoji support (`add_emoji` method in ProgressDisplay)
- [ ] Implement direct writer functionality for TaskHandle (currently unused `writer` field)
- [ ] Implement output passthrough functionality (currently unused `passthrough` field and `has_passthrough` method in SingleLineBase)
- [ ] Standardize thread config creation (replace Config::new with more robust `create_thread_config` function)
- [ ] Add terminal size customization in TestBuilder (currently unused `width` and `height` fields)

## Implementation Strategy

1. [x] First, refactor Window modes by creating WindowBase
2. [x] Next, refactor SingleLine modes by creating SingleLineBase
3. [x] Introduce JobTracker trait to standardize job tracking functions
4. [x] Enhance BaseConfig with standardized methods
5. [x] Refactor test utilities
6. [ ] Standardize thread configuration interfaces
7. [ ] Improve documentation
8. [ ] Optimize for performance 
9. [ ] Implement remaining features (WindowWithTitle, total jobs, emoji support, direct writer, passthrough mode) 