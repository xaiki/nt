# nt_progress TODO

## Phase 1: Core Trait and Default Implementation

### ThreadConfig Trait
- [x] Define minimal trait interface:
  ```rust
  pub trait ThreadConfig: Send + Sync + Debug {
      fn lines_to_display(&self) -> usize;
      fn handle_message(&mut self, message: String) -> Vec<String>;
      fn get_lines(&self) -> Vec<String>;
      fn clone_box(&self) -> Box<dyn ThreadConfig>;
  }
  ```

### Config Wrapper
- [x] Create wrapper struct `Config` for mode implementations
- [x] Implement Clone via `clone_box`
- [x] Add delegation methods to underlying implementation

### LimitedConfig (Default)
- [x] Implement basic LimitedConfig:
  ```rust
  pub struct Limited {
      base: BaseConfig,
      last_line: String,
  }
  ```
- [x] Implement ThreadConfig trait for LimitedConfig
- [x] Add stdout/stderr redirection
- [x] Add tests for basic functionality

## Phase 2: Other Mode Implementations

### CapturingConfig
- [x] Implement basic CapturingConfig:
  ```rust
  pub struct Capturing {
      current_line: String,
  }
  ```
- [x] Implement ThreadConfig trait
- [x] Add tests for single line replacement

### WindowConfig
- [ ] Implement basic WindowConfig:
  ```rust
  pub struct Window {
      lines: VecDeque<String>,
      max_lines: usize,
  }
  ```
- [ ] Implement ThreadConfig trait
- [ ] Add tests for N line retention

### WindowWithTitleConfig
- [ ] Implement basic WindowWithTitleConfig:
  ```rust
  pub struct WindowWithTitle {
      title: String,
      emoji_stack: String,
      lines: VecDeque<String>,
      max_lines: usize,
  }
  ```
- [ ] Implement ThreadConfig trait
- [ ] Add title and emoji support
- [ ] Add tests for title and emoji handling

## Phase 3: ProgressDisplay Integration

### Basic Integration
- [x] Make ProgressDisplay use Config from modes module
- [x] Update constructor to use LimitedConfig by default
- [x] Make `start_display_thread` use the `mode` parameter
- [x] Extract common display functionality to `render_display`

### Terminal Handling
- [ ] Move terminal size adjustment to separate module
- [ ] Add terminal size change detection
- [ ] Add tests for terminal size handling

## Phase 4: Testing and Documentation

### Testing
- [ ] Add unit tests for each config
- [ ] Add integration tests for mode switching
- [ ] Add tests for terminal size changes
- [ ] Add tests for concurrent usage

### Documentation
- [ ] Update README with new API
- [ ] Add examples for each mode
- [ ] Add examples for mode switching
- [ ] Document thread safety guarantees

## Future Enhancements

### Display Features
- [ ] Color support
- [ ] Line wrapping
- [ ] Custom progress indicators
- [ ] Progress bars

### Performance
- [ ] Message batching
- [ ] Buffer optimization
- [ ] Message compression

### Error Handling
- [ ] Detailed error types
- [ ] Error recovery
- [ ] Context logging 