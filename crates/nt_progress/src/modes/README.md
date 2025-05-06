# Thread Configuration System

This document outlines the design of the thread configuration system, which is responsible for managing the display behavior of different output modes in the nt_progress library.

## Overview

The thread configuration system is built around several key components:

1. **ThreadConfig Trait**: The core interface that all display modes must implement
2. **JobTracker Trait**: Interface for tracking job progress
3. **Base Implementations**:
   - **WindowBase**: For modes that display multiple lines (Window, WindowWithTitle)
   - **SingleLineBase**: For modes that display a single line (Limited, Capturing)
4. **ThreadMode Enum**: Represents the available display modes
5. **Config Wrapper**: Provides a unified interface to the various mode implementations

## Key Components

### ThreadConfig Trait

```rust
pub trait ThreadConfig: Send + Sync + Debug {
    /// Returns the number of lines this mode needs to display
    fn lines_to_display(&self) -> usize;

    /// Processes a new message and returns the lines to display
    fn handle_message(&mut self, message: String) -> Vec<String>;

    /// Returns the current lines to display without processing a new message
    fn get_lines(&self) -> Vec<String>;

    /// Returns a boxed clone of this config
    fn clone_box(&self) -> Box<dyn ThreadConfig>;
}
```

This trait defines the core behavior that all display modes must implement:
- Determining how many lines they need for display
- Processing messages and returning the lines to display
- Retrieving the current lines without processing new messages
- Supporting cloning (necessary for thread safety)

### JobTracker Trait

```rust
pub trait JobTracker: Send + Sync + Debug {
    /// Get the total number of jobs
    fn get_total_jobs(&self) -> usize;
    
    /// Increment the completed jobs counter and return the new value
    fn increment_completed_jobs(&self) -> usize;
}
```

This trait is responsible for tracking job progress across the various modes:
- Getting the total number of jobs assigned
- Incrementing and returning the count of completed jobs

### Base Implementations

#### WindowBase

```rust
pub struct WindowBase {
    base: BaseConfig,
    lines: VecDeque<String>,
    max_lines: usize,
}
```

The `WindowBase` implementation provides:
- A fixed-size scrolling window of output lines
- Common functionality for window-based modes
- Support for maximum line limits with automatic scrolling

#### SingleLineBase

```rust
pub struct SingleLineBase {
    base: BaseConfig,
    current_line: String,
    passthrough: bool,
}
```

The `SingleLineBase` implementation provides:
- A single-line display mode
- Support for passthrough to stdout/stderr
- Base for limited and capturing modes

### ThreadMode Enum

```rust
pub enum ThreadMode {
    Limited,
    Capturing,
    Window(usize),
    WindowWithTitle(usize),
}
```

This enum represents the available display modes:
- **Limited**: Shows only the most recent message
- **Capturing**: Captures output without displaying
- **Window**: Displays the last N messages in a scrolling window
- **WindowWithTitle**: Similar to Window but with a title bar

### Config Wrapper

```rust
pub struct Config {
    config: Box<dyn ThreadConfig>,
}
```

The `Config` wrapper provides:
- A unified interface to all mode implementations
- Factory methods for creating thread configurations
- Error handling for invalid configurations

## Standard Pattern for Mode Implementation

When implementing a new mode, follow this standard pattern:

1. **Create a struct** that includes the appropriate base:
   ```rust
   pub struct MyMode {
       window_base: WindowBase,  // or SingleLineBase
       // Additional mode-specific fields
   }
   ```

2. **Implement initialization**:
   ```rust
   impl MyMode {
       pub fn new(total_jobs: usize, max_lines: usize) -> Result<Self, String> {
           // Validate parameters
           if max_lines < MIN_REQUIRED_LINES {
               return Err("MyMode requires at least MIN_REQUIRED_LINES lines".to_string());
           }
           
           // Create the mode
           Ok(Self {
               window_base: WindowBase::new(total_jobs, max_lines)?,
               // Initialize other fields
           })
       }
   }
   ```

3. **Implement JobTracker** (usually delegating to the base):
   ```rust
   impl JobTracker for MyMode {
       fn get_total_jobs(&self) -> usize {
           self.window_base.get_total_jobs()
       }
       
       fn increment_completed_jobs(&self) -> usize {
           self.window_base.increment_completed_jobs()
       }
   }
   ```

4. **Implement ThreadConfig**:
   ```rust
   impl ThreadConfig for MyMode {
       fn lines_to_display(&self) -> usize {
           // Return the number of lines needed by this mode
           self.window_base.max_lines()
       }

       fn handle_message(&mut self, message: String) -> Vec<String> {
           // Process the message and update state
           self.window_base.add_message(message);
           
           // Return current lines to display
           self.get_lines()
       }

       fn get_lines(&self) -> Vec<String> {
           // Return the lines to display
           self.window_base.get_lines()
       }

       fn clone_box(&self) -> Box<dyn ThreadConfig> {
           Box::new(self.clone())
       }
   }
   ```

5. **Implement Clone** (if needed beyond derive):
   ```rust
   impl Clone for MyMode {
       fn clone(&self) -> Self {
           Self {
               window_base: self.window_base.clone(),
               // Clone other fields
           }
       }
   }
   ```

6. **Update the ThreadMode enum** to include your new mode:
   ```rust
   pub enum ThreadMode {
       Limited,
       Capturing,
       Window(usize),
       WindowWithTitle(usize),
       MyMode(usize),  // Add your mode with any parameters it needs
   }
   ```

7. **Update Config::new** to handle your mode:
   ```rust
   impl Config {
       pub fn new(mode: ThreadMode, total_jobs: usize) -> Result<Self, String> {
           let config: Box<dyn ThreadConfig> = match mode {
               // Existing modes...
               ThreadMode::MyMode(max_lines) => Box::new(MyMode::new(total_jobs, max_lines)?),
           };

           Ok(Self { config })
       }
   }
   ```

## Error Handling

Error handling in mode implementations should follow these principles:

1. **Validation**: Validate all parameters passed to constructors
2. **Specific Error Messages**: Provide clear, specific error messages for invalid parameters
3. **Result Return Type**: Use `Result<Self, String>` for constructor functions
4. **Error Propagation**: Use the `?` operator to propagate errors from dependent components

## Testing

Each mode should include comprehensive tests covering:

1. **Basic Functionality**: Basic message handling and display
2. **Error Handling**: Proper handling of invalid parameters and edge cases
3. **Concurrency**: Behavior with multiple concurrent tasks
4. **Special Cases**: Handling of special characters, long lines, etc.

For examples, refer to the test modules in each mode implementation.

## Example: Implementing a Custom Mode

Here's an example of implementing a "Rotating" mode that cycles through a fixed set of messages:

```rust
use super::{ThreadConfig, WindowBase, JobTracker};

/// Configuration for Rotating mode
/// 
/// In Rotating mode, messages are shown in rotation,
/// cycling through a fixed window of the last N messages.
#[derive(Debug, Clone)]
pub struct Rotating {
    window_base: WindowBase,
    current_index: usize,
    rotate_interval: usize,  // How many messages before rotating
    message_count: usize,    // Number of messages received
}

impl Rotating {
    pub fn new(total_jobs: usize, max_lines: usize, rotate_interval: usize) -> Result<Self, String> {
        if max_lines < 1 {
            return Err("Rotating mode requires at least 1 line".to_string());
        }
        
        if rotate_interval < 1 {
            return Err("Rotate interval must be at least 1".to_string());
        }
        
        Ok(Self {
            window_base: WindowBase::new(total_jobs, max_lines)?,
            current_index: 0,
            rotate_interval,
            message_count: 0,
        })
    }
}

impl JobTracker for Rotating {
    fn get_total_jobs(&self) -> usize {
        self.window_base.get_total_jobs()
    }
    
    fn increment_completed_jobs(&self) -> usize {
        self.window_base.increment_completed_jobs()
    }
}

impl ThreadConfig for Rotating {
    fn lines_to_display(&self) -> usize {
        1  // One line at a time in rotation
    }

    fn handle_message(&mut self, message: String) -> Vec<String> {
        // Add the message to our window
        self.window_base.add_message(message);
        self.message_count += 1;
        
        // Every rotate_interval messages, rotate the display
        if self.message_count % self.rotate_interval == 0 {
            let window_size = self.window_base.get_lines().len();
            if window_size > 0 {
                self.current_index = (self.current_index + 1) % window_size;
            }
        }
        
        self.get_lines()
    }

    fn get_lines(&self) -> Vec<String> {
        let lines = self.window_base.get_lines();
        if lines.is_empty() {
            return vec![];
        }
        
        // Return only the current message in rotation
        vec![lines[self.current_index].clone()]
    }

    fn clone_box(&self) -> Box<dyn ThreadConfig> {
        Box::new(self.clone())
    }
}

// Then update ThreadMode and Config::new as described above
```

## Best Practices

1. **Delegate to Base Classes**: Reuse functionality from WindowBase or SingleLineBase
2. **Clear Error Messages**: Provide specific error messages for validation failures
3. **Thorough Documentation**: Document the purpose and behavior of your mode
4. **Comprehensive Testing**: Test all aspects of your mode's behavior
5. **Performance Considerations**: Be mindful of memory usage and string allocations
6. **Thread Safety**: Ensure your implementation is thread-safe 