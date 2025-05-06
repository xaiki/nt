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
   - [x] WindowWithTitle mode tests

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
src/tests/                    # Unit tests
    common.rs                 # Common test utilities and TestEnv
    mod.rs                    # Test module definition
    terminal.rs               # Terminal-specific tests
    window.rs                 # Window mode tests
    window_with_title.rs      # WindowWithTitle mode tests
    capturing.rs              # Capturing mode tests
    limited.rs                # Limited mode tests
    display.rs                # Display functionality tests
    test_builder.rs           # TestBuilder utility for simplified test creation
    test_builder_example.rs   # Example tests using TestBuilder

tests/                        # Integration tests
    common.rs                 # Shared test utilities
    terminal.rs               # Terminal integration tests
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
       parser: Parser,
       expected: Vec<String>,
       width: u16,
       height: u16,
   }
   ```
   - [x] Uses vt100::Parser for terminal emulation
   - [x] Tracks expected output
   - [x] Provides verification methods
   - [x] Manages terminal dimensions

2. **Key Methods**
   - [x] `new(width, height)`: Create test environment
   - [x] `contents()`: Get current terminal content
   - [x] `verify()`: Compare actual vs expected
   - [x] `write(text)`: Write and track output
   - [x] `writeln(text)`: Write line and track output
   - [x] `move_to(x, y)`: Move cursor
   - [x] `set_color(color)`: Set text color
   - [x] `reset_styles()`: Reset terminal styles

3. **Verification Process**
   - [x] Track all expected output
   - [x] Capture actual terminal output
   - [x] Compare line by line
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
- [x] Set up vt100 parser for terminal emulation
- [x] Implement basic output capture
- [x] Test cursor movements

### Phase 3: Core Testing Implementation
- [x] Implement test environment integration
- [x] Add precise output verification
- [x] Create TestEnv utility

### Phase 4: Comprehensive Test Suite
- [x] Implement mode-specific tests
- [x] Add concurrency testing
- [ ] Add performance benchmarking

### Phase 5: Test Utilities Refactoring
- [x] Create a TestBuilder utility to simplify test creation
- [x] Add standard testing utilities for common mode assertions
- [x] Reduce duplication in test setup

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

## TestBuilder Usage

Using the new TestBuilder utility greatly simplifies test creation:

```rust
#[tokio::test]
async fn test_builder_basic_message() -> Result<()> {
    // Create a new TestBuilder with default terminal size (80x24)
    let mut builder = TestBuilder::new();
    
    // Test a simple message with Limited mode (default)
    let display = builder.test_message("Hello, world!").await?;
    display.stop().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_builder_window_mode() -> Result<()> {
    // Create a TestBuilder for Window mode with 5 lines
    let mut builder = TestBuilder::new().window_mode(5);
    
    // Test window features with multiple lines
    let display = builder.test_window_features(&[
        "First line",
        "Second line",
        "Third line",
        "Fourth line",
        "Fifth line",
    ]).await?;
    
    display.stop().await?;
    
    Ok(())
}
```

## Next Steps

1. **Short Term**
   - [x] Create TestBuilder utility for simplified test creation
   - [x] Add standard testing utilities for common assertions
   - [x] Refactor existing tests to use new utilities

2. **Medium Term**
   - [ ] Implement remaining features (WindowWithTitle mode, total jobs support)
   - [ ] Set up performance testing infrastructure
   - [ ] Add memory usage benchmarks
   - [ ] Add CPU usage benchmarks

3. **Long Term**
   - [ ] Implement comprehensive performance testing
   - [ ] Add resource utilization monitoring
   - [ ] Set up continuous benchmarking

## Implementation Plan

### 1. TestBuilder Utility (Highest Priority)
   - [x] Core Builder Structure
     - [x] Create TestBuilder class with fluent interface
     - [x] Support common test setup patterns
     - [x] Simplify mode-specific test creation
   - [x] Assertion Helpers
     - [x] Create standard assertion methods
     - [x] Provide common test verification patterns
     - [x] Improve error messages for test failures
   - [x] Test Fixtures
     - [x] Standard test configurations
     - [x] Common test scenarios
     - [x] Reusable test patterns

### 2. Common Mode Assertions (Medium Priority)
   - [x] Basic Mode Testing
     - [x] Standard message handling tests
     - [x] Standard display formatting tests
     - [x] Standard concurrency tests
   - [x] Mode-Specific Testing
     - [x] Window mode specific assertions
     - [x] Limited mode specific assertions
     - [x] Capturing mode specific assertions
     - [x] WindowWithTitle mode specific assertions

### 3. Performance Testing Utilities (Lower Priority)
   - [ ] Benchmarking Framework
     - [ ] Output speed measurement
     - [ ] Resource usage tracking
     - [ ] Comparative performance analysis
   - [ ] Standard Benchmarks
     - [ ] Single line output benchmarks
     - [ ] Window mode benchmarks
     - [ ] Concurrent task benchmarks

## Conclusion

The testing strategy focuses on precise output verification using TestBackend and the TestEnv utility, now further enhanced with the TestBuilder utility. This ensures reliable testing of terminal output and mode-specific behavior with reduced boilerplate and improved readability. The approach provides:

1. Exact character-by-character verification
2. Support for ANSI sequences and terminal operations
3. Concurrent testing capabilities
4. Detailed error reporting
5. Simplified test creation through TestBuilder
6. Performance monitoring (planned)

This strategy helps maintain high code quality and reliability while making it easier to catch and debug issues with terminal output. The TestBuilder utility makes it significantly easier to create and maintain tests, reducing duplication and improving test coverage. 