# Testing Strategy

## Overview

This document outlines the testing strategy for the `nt_progress` crate, focusing on terminal output verification and mode-specific behavior testing.

## Current Implementation Status

### Implemented Tests ✅

1. **Basic Output Tests**
   - [x] Text output verification using TestBackend
   - [x] ANSI sequence testing
   - [x] Cursor movement testing
   - [x] Exact character-by-character comparison

2. **Mode-Specific Tests**
   - [x] Capturing mode tests
   - [x] Limited mode tests
   - [x] Window mode tests
   - [ ] WindowWithTitle mode tests (file exists but empty)

3. **Concurrency Tests**
   - [x] Multiple task output
   - [x] Output ordering
   - [x] Resource contention

4. **Performance Tests**
   - [ ] Output speed
   - [ ] Memory usage
   - [ ] CPU usage

## Test Structure

```
src/tests/
    common.rs       # Common test utilities and TestEnv
    terminal.rs     # Terminal-specific tests
    window.rs       # Window mode tests
    window_with_title.rs # WindowWithTitle mode tests (empty)
    capturing.rs    # Capturing mode tests
    display.rs      # Display functionality tests
```

## Test Categories

1. **Basic Output Tests**
   - Text output verification using TestBackend
   - ANSI sequence testing
   - Cursor movement testing
   - Exact character-by-character comparison

2. **Mode-Specific Tests**
   - Capturing mode tests
   - Limited mode tests
   - Window mode tests
   - WindowWithTitle mode tests

3. **Concurrency Tests**
   - Multiple task output
   - Output ordering
   - Resource contention

4. **Performance Tests**
   - Output speed
   - Memory usage
   - CPU usage

## Testing Guidelines

### Output Verification

1. **Text Content**
   - [x] Use TestBackend for exact output capture
   - [x] Verify character-by-character matches
   - [x] Check for proper line endings
   - [x] Validate special characters and unicode

2. **ANSI Sequences**
   - [x] Verify escape sequences
   - [x] Check cursor movements
   - [x] Validate color codes
   - [x] Test style resets

3. **Concurrent Output**
   - [x] Check output ordering
   - [x] Verify no corruption
   - [x] Test resource limits
   - [x] Validate thread safety

### Test Environment

1. **TestEnv Structure**
   ```rust
   struct TestEnv {
       backend: TestBackend,
       expected: Vec<String>,
   }
   ```
   - [x] Wraps crossterm::test::TestBackend
   - [x] Tracks expected output
   - [x] Provides verification methods

2. **Key Methods**
   - [x] `new(width, height)`: Create test environment
   - [x] `expect(line)`: Add expected output
   - [x] `verify()`: Compare actual vs expected
   - [x] `write(text)`: Write and track output
   - [x] `writeln(text)`: Write line and track output
   - [x] `move_to(x, y)`: Move cursor
   - [x] `set_color(color)`: Set text color
   - [x] `reset_styles()`: Reset terminal styles

3. **Verification Process**
   - [x] Track all expected output
   - [x] Capture actual terminal output
   - [x] Compare character by character
   - [x] Provide detailed error messages

### Performance Considerations

1. **Output Speed**
   - [ ] Measure render time
   - [ ] Track update frequency
   - [ ] Monitor buffer usage

2. **Resource Usage**
   - [ ] Track memory allocation
   - [ ] Monitor CPU usage
   - [ ] Check file descriptor usage

## Implementation Phases

### Phase 1: Research and Documentation
- [x] Document testing strategy
- [x] Define test utilities
- [x] Specify testing scenarios

### Phase 2: Proof of Concept
- [x] Set up crossterm test environment
- [x] Implement basic output capture
- [x] Test cursor movements

### Phase 3: Core Testing Implementation
- [x] Implement TestBackend integration
- [x] Add precise output verification
- [x] Create TestEnv utility

### Phase 4: Comprehensive Test Suite
- [x] Implement mode-specific tests
- [x] Add concurrency testing
- [ ] Add performance benchmarking

## Example Test Structure

```rust
#[test]
fn test_mode_basic() {
    let mut env = TestEnv::new(80, 24);
    
    // Set up test
    let mut mode = Mode::new();
    
    // Test basic output
    env.writeln("test message");
    mode.handle_message("test message".to_string());
    env.verify();
    
    // Test multiple messages
    env.writeln("new message");
    mode.handle_message("new message".to_string());
    env.verify();
}

#[tokio::test]
async fn test_mode_concurrent() {
    let mut env = TestEnv::new(80, 24);
    let mut display = ProgressDisplay::new().await;
    
    // Spawn concurrent tasks
    let handles = spawn_tasks(&mut display, &mut env);
    
    // Wait for completion
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify final state
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}
```

## Next Steps

1. **Short Term**
   - [ ] Implement WindowWithTitle mode tests
   - [ ] Add missing edge case tests
   - [ ] Add error handling tests

2. **Medium Term**
   - [ ] Set up performance testing infrastructure
   - [ ] Add memory usage benchmarks
   - [ ] Add CPU usage benchmarks

3. **Long Term**
   - [ ] Implement comprehensive performance testing
   - [ ] Add resource utilization monitoring
   - [ ] Set up continuous benchmarking

## Implementation Plan

### 1. WindowWithTitle Mode Tests (Highest Priority)
   - [ ] Basic Setup
     - [ ] Create test module structure
     - [ ] Set up TestEnv for window title testing
     - [ ] Define test fixtures
   - [ ] Core Functionality Tests
     - [ ] Test title initialization
     - [ ] Test title updates
     - [ ] Test title persistence
   - [ ] Edge Cases
     - [ ] Test empty title
     - [ ] Test long titles
     - [ ] Test special characters in titles
   - [ ] Integration Tests
     - [ ] Test title with window resizing
     - [ ] Test title with content updates
     - [ ] Test title with multiple windows

### 2. Edge Case Tests (Medium Priority)
   - [ ] Terminal Edge Cases
     - [ ] Test with minimum terminal size
     - [ ] Test with maximum terminal size
     - [ ] Test with invalid terminal sizes
   - [ ] Content Edge Cases
     - [ ] Test empty content
     - [ ] Test very long content
     - [ ] Test content with special characters
   - [ ] State Edge Cases
     - [ ] Test state transitions
     - [ ] Test error recovery
     - [ ] Test resource exhaustion

### 3. Error Handling Tests (Medium Priority)
   - [ ] Basic Error Cases
     - [ ] Test invalid configurations
     - [ ] Test resource allocation failures
     - [ ] Test invalid state transitions
   - [ ] Recovery Tests
     - [ ] Test error recovery procedures
     - [ ] Test state restoration
     - [ ] Test resource cleanup
   - [ ] Error Propagation
     - [ ] Test error reporting
     - [ ] Test error logging
     - [ ] Test error handling across boundaries

### 4. Performance Testing Infrastructure (Lower Priority)
   - [ ] Setup Phase
     - [ ] Create benches directory
     - [ ] Set up Criterion
     - [ ] Define baseline metrics
   - [ ] Basic Benchmarks
     - [ ] Measure render time
     - [ ] Track update frequency
     - [ ] Monitor buffer usage
   - [ ] Resource Benchmarks
     - [ ] Track memory allocation
     - [ ] Monitor CPU usage
     - [ ] Check file descriptor usage

### 5. Continuous Benchmarking (Lowest Priority)
   - [ ] Infrastructure Setup
     - [ ] Set up benchmark storage
     - [ ] Configure benchmark reporting
     - [ ] Set up performance tracking
   - [ ] Monitoring Setup
     - [ ] Configure resource monitoring
     - [ ] Set up alerting
     - [ ] Define performance thresholds

## Conclusion

The testing strategy focuses on precise output verification using TestBackend and the TestEnv utility. This ensures reliable testing of terminal output and mode-specific behavior. The approach provides:

1. Exact character-by-character verification
2. Support for ANSI sequences and terminal operations
3. Concurrent testing capabilities
4. Detailed error reporting
5. Performance monitoring (planned)

This strategy helps maintain high code quality and reliability while making it easier to catch and debug issues with terminal output. 