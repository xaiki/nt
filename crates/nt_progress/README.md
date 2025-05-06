# nt_progress

A flexible and thread-safe progress display library for Rust applications, featuring concurrent progress tracking, terminal-aware display, and customizable output modes.

## Features

- 🔄 Thread-safe progress tracking
- 📊 Multiple display modes
- 🖥️ Terminal-aware output
- 🎨 Customizable spinners
- 📝 Window-based output with titles
- 🚀 Async/await support
- 🎯 Progress bar support
- 🔍 Automatic terminal size detection
- 🛡️ Robust error handling with context and recovery strategies

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nt_progress = "0.1.0"
```

## Usage

### Basic Example

```rust
use nt_progress::{ProgressDisplay, ThreadLogger, ThreadConfig, ThreadMode};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Create channel for progress messages
    let (tx, rx) = mpsc::channel(100);
    let progress = ProgressDisplay::new(rx);
    
    // Start progress display task
    let progress_clone = progress.clone();
    let display_handle = tokio::spawn(async move {
        loop {
            progress_clone.display().await.unwrap();
            progress_clone.process_messages().await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    // Create a logger for your task
    let mut logger = ThreadLogger::new(
        0, // thread ID
        tx.clone(),
        ThreadConfig::new(ThreadMode::Window(3), 10) // show last 3 lines, 10 total jobs
    );

    // Log progress
    for i in 0..10 {
        let progress = (i + 1) * 10;
        let bar = "▉".repeat(progress / 2) + &"▏".repeat(50 - progress / 2);
        logger.log(format!("Progress: {}%|{}| {}/10", progress, bar, i + 1)).await;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Clean up
    display_handle.abort();
}
```

### Display Modes

The library supports three display modes:

1. **Capturing Mode**: Shows only one line at a time, replacing it with new content
```rust
ThreadConfig::new(ThreadMode::Capturing, total_jobs)
```
- Always shows exactly one line
- Replaces the previous line with new content
- No title support
- No emoji support
- Ideal for progress bars or status updates

2. **Limited Mode**: Passes messages to stdout/stderr but keeps the last one for display
```rust
ThreadConfig::new(ThreadMode::Limited, total_jobs)
```
- Messages are passed through to stdout/stderr
- Only the last message is kept for display in the progress block
- No title support
- No emoji support
- Ideal for verbose output that should be visible in the terminal

3. **Window Mode**: Shows the last N lines for each thread
```rust
ThreadConfig::new(ThreadMode::Window(3), total_jobs) // Shows last 3 lines
```
- Displays the last N lines of output
- N is specified by the user (e.g., Window(3) for 3 lines)
- N will be automatically reduced if it doesn't fit the terminal
- No title support
- No emoji support

4. **Window with Title**: Shows a title line plus N lines
```rust
ThreadConfig::new(ThreadMode::WindowWithTitle(2), total_jobs) // Title + 2 lines
```
- Displays a title followed by the last N lines of output
- N is specified by the user (e.g., WindowWithTitle(2) for 2 lines)
- N will be automatically reduced if it doesn't fit the terminal
- Title is always displayed at the top
- Supports title updates via `task_handle.set_title("New Title")` or `progress_display.set_title(thread_id, "New Title")`
- Supports emoji stacking in the title (coming soon)

### Multiple Progress Trackers

You can track progress from multiple sources concurrently:

```rust
let mut loggers = vec![];
for i in 0..3 {
    loggers.push(ThreadLogger::new(
        i,
        tx.clone(),
        ThreadConfig::new(ThreadMode::Window(2), total_jobs)
    ));
}

for logger in loggers.iter_mut() {
    logger.log("Starting task...".to_string()).await;
}
```

### Terminal-Aware Display

The library automatically adapts to terminal size:
- Adjusts output to fit terminal height
- Handles cursor movement
- Cleans up display lines
- Supports spinners and progress bars

## Features in Detail

### ThreadLogger

The `ThreadLogger` struct provides:
- Async logging capabilities
- Message buffering based on mode
- Automatic line management
- Thread-safe message passing

### ProgressDisplay

The `ProgressDisplay` struct handles:
- Terminal output management
- Spinner animations
- Multi-thread message coordination
- Terminal size adaptation

### ThreadConfig

Configure display behavior with:
- Display mode selection
- Line count control
- Job tracking
- Result emoji support (Window with Title mode only)

### Setting Titles in WindowWithTitle Mode

When using WindowWithTitle mode, you can update the title at any time:

```rust
// Using a task handle
let mut task = progress.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "Initial Title").await?;
// ... later ...
task.set_title("Updated Title".to_string()).await?;

// Using the progress display directly
let thread_id = task.thread_id();
progress.set_title(thread_id, "Another Title".to_string()).await?;
```

Note that attempting to set a title for a task that is not in WindowWithTitle mode will result in an error.

## Error Handling

The library provides robust error handling with detailed context and graceful recovery:

### Error Types

- **ProgressError**: Top-level error type with various variants:
  - `ModeCreation`: Errors related to creating and configuring display modes
  - `TaskOperation`: Errors that occur during task operations
  - `DisplayOperation`: Errors during display operations
  - `Io`: Input/output errors
  - `External`: Errors from external sources
  - `WithContext`: Errors with additional context information

- **ModeCreationError**: Specific errors for mode creation:
  - `InvalidWindowSize`: When window size parameters are invalid
  - `MissingParameter`: When a required parameter is missing
  - `Implementation`: General implementation errors

### Context-Aware Errors

Errors include rich context information to aid debugging:

```rust
// Add context to an error
let result = operation().with_context("creating config", "ProgressDisplay");

// Add detailed context
let ctx = ErrorContext::new("operation", "component")
    .with_thread_id(42)
    .with_details("Additional details");
    
let result = operation().context(ctx);
```

### Error Recovery

The library implements graceful recovery strategies:

- Mode creation will attempt to use reasonable fallback values
- If a requested window size is invalid, a sensible default is used
- If a mode can't be created, simpler modes are tried as fallbacks
- Operations never panic due to configuration errors

### Error Logging and Debugging

```rust
// Get detailed error chain information
let debug_info = format_error_debug(&error);

// Log the complete error chain
log_error(&error);
```

Error chains preserve all context, making it easy to trace the source of problems.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Implementation Notes

- Each mode has its own `ThreadConfig` implementation
- The `ProgressDisplay` struct is generic over any implementer of the `ThreadConfig` trait
- Window and Window With Title modes share common code and structs
- Emoji handling is only implemented in Window With Title mode
- Title handling is only implemented in Window With Title mode 