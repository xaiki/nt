# nt_progress

A flexible and thread-safe progress display library for Rust applications, featuring concurrent progress tracking, terminal-aware display, customizable output modes, and a powerful templating system for formatted progress reporting.

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
- 📋 Powerful templating system with variable interpolation and formatting
- 🎭 Conditional rendering for dynamic display

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

### Setting Total Jobs

You can update the total number of jobs for a task at any time:

```rust
// Using a task handle to update a specific task
let task = progress.spawn_with_mode(ThreadMode::Window(3), || "Task").await?;
task.set_total_jobs(100).await?; // Set to 100 total jobs

// Using the progress display to update a specific task
let thread_id = task.thread_id();
progress.set_total_jobs(Some(thread_id), 100).await?;

// Using the progress display to update all tasks
progress.set_total_jobs(None, 100).await?; // Set 100 jobs for all tasks
```

This is useful when you don't know the total count of jobs initially, or when the job count changes dynamically during execution.

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

## Template System

The library includes a powerful templating system for formatting progress messages, status updates, and progress bars. This allows you to create customized output formats for your progress displays.

### Basic Template Syntax

Templates use a simple syntax for variable interpolation and conditional sections:

- `{var}` - Interpolate the variable `var` directly
- `{var:format}` - Apply a format to the variable `var`
- `{var:format:param1:param2}` - Apply a format with parameters
- `{?condition}content{/}` - Include `content` only if `condition` is truthy
- `{!condition}content{/}` - Include `content` only if `condition` is falsy

### Creating and Using Templates

```rust
use nt_progress::formatter::{ProgressTemplate, TemplateContext};

// Create a template with progress bar, percentage and counts
let template = ProgressTemplate::new("Progress: {progress:bar} {progress:percent} ({completed}/{total})");

// Set up context with variables
let mut ctx = TemplateContext::new();
ctx.set("progress", 0.5)  // Progress as a value between 0.0 and 1.0
   .set("completed", 5)   // Number of completed items
   .set("total", 10);     // Total number of items

// Render the template
let output = template.render(&ctx).unwrap();
// Output: "Progress: [=====     ] 50% (5/10)"
```

### Built-in Template Presets

The library provides several built-in template presets for common use cases:

```rust
use nt_progress::formatter::{TemplatePreset, TemplateContext};

// Use the SimpleProgress preset
let template = TemplatePreset::SimpleProgress.create_template();

// Other available presets include:
// - TaskStatus: "Running task: <message>"
// - JobProgress: "Completed 5/10 jobs (50%)"
// - DownloadProgress: "Downloading file.txt [====    ] 10.5 MB / 20 MB (50%)"
```

### Available Formatting Options

#### Progress Bars

The template system supports multiple progress bar types:

1. **Standard Bar**: `{progress:bar}`
   ```
   [=====     ]
   ```

2. **Block Bar**: `{progress:bar:block}`
   ```
   [█████     ]
   ```

3. **Numeric**: `{progress:bar:numeric}`
   ```
   50%
   ```

4. **Spinner**: `{progress:bar:spinner}`
   ```
   / (animates through frames: -, \, |, /)
   ```

5. **Interactive**: `{progress:bar:interactive}`
   ```
   [=====>    ]
   ```

6. **Custom Indicators**:
   - Dots: `{progress:bar:custom:dots}`
   - Braille: `{progress:bar:custom:braille}`
   - Gradient: `{progress:bar:custom:gradient}`

#### Progress Bar Customization

Each progress bar type supports various customization options:

```rust
// Width (number of characters)
"{progress:bar:10}"  // 10 character width

// Custom characters for bar
"{progress:bar:bar:=:_:[:]}"  // fill_char, empty_char, left_bracket, right_bracket

// Custom block bar
"{progress:bar:block:#: }"  // Use # for filled blocks, space for empty

// Custom spinner frames
"{progress:bar:spinner:⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏}"  // Use custom animation frames

// Numeric with/without percent sign
"{progress:bar:numeric}"     // "50%"
"{progress:bar:numeric:false}"  // "50"

// Color support (available for all bar types)
"{progress:bar:bar:=:_:red:blue}"  // Red filled parts, blue empty parts
```

#### Text Formatting

1. **Percentage**: `{progress:percent}`
   ```
   50%
   ```

2. **Ratio**: `{completed:ratio:total}`
   ```
   5/10
   ```

3. **Padding**:
   - Left padding: `{text:lpad:10}` → `"      text"`
   - Right padding: `{text:rpad:10}` → `"text      "`
   - Center padding: `{text:pad:10}` → `"   text   "`

4. **Color**: `{status:color:green}`
   ```
   Success (in green)
   ```
   
   Supported colors: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `reset`

### Conditional Formatting

You can conditionally include content based on variable values:

```rust
// Include if completed is truthy (non-zero, non-empty)
"Task status: {?completed}Done{/}{!completed}In progress{/}"

// Show different status based on success flag
"{?success}✅ Completed{/}{!success}❌ Failed{/}"
```

### Advanced Template Examples

#### Download Progress with File Size

```rust
let template = ProgressTemplate::new(
    "Downloading {filename} {progress:bar:15} {bytes_done:lpad:8} / {bytes_total:lpad:8} ({progress:percent})"
);

let mut ctx = TemplateContext::new();
ctx.set("filename", "example.zip")
   .set("progress", 0.35)
   .set("bytes_done", "3.5 MB")
   .set("bytes_total", "10.0 MB");

// Output: "Downloading example.zip [=====          ]    3.5 MB /   10.0 MB (35%)"
```

#### Colored Status with Conditional Elements

```rust
let template = ProgressTemplate::new(
    "Task: {name} - {?running}[{status:color:yellow}]{/}{?completed}[{status:color:green}]{/}{?failed}[{status:color:red}]{/}"
);

let mut ctx = TemplateContext::new();
ctx.set("name", "File Processing")
   .set("running", true)
   .set("completed", false)
   .set("failed", false)
   .set("status", "In Progress");

// Output: "Task: File Processing - [In Progress]" (with "In Progress" in yellow)
```

#### Multipart Progress Bar with Different Indicators

```rust
let template = ProgressTemplate::new(
    "Stage 1: {stage1:bar:block:5} | Stage 2: {stage2:bar:10} | Overall: {total:bar:custom:gradient:15}"
);

let mut ctx = TemplateContext::new();
ctx.set("stage1", 1.0)      // Complete
   .set("stage2", 0.5)      // Half complete
   .set("total", 0.75);     // 75% complete

// Output with different progress bar styles for each stage
```

### Integrating Templates with Tasks

The template system integrates seamlessly with the `ThreadLogger` and task handling system. Here's how to use templates with your logging tasks:

#### Using Templates with ThreadLogger

```rust
use nt_progress::{ProgressDisplay, ThreadLogger, ThreadConfig, ThreadMode};
use nt_progress::formatter::{ProgressTemplate, TemplateContext};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Create channel for progress messages
    let (tx, rx) = mpsc::channel(100);
    let progress = ProgressDisplay::new(rx);
    
    // Start progress display in the background
    let progress_handle = tokio::spawn(async move {
        loop {
            progress.display().await.unwrap();
            progress.process_messages().await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    // Create a logger for the task
    let mut logger = ThreadLogger::new(
        0, // thread ID
        tx.clone(),
        ThreadConfig::new(ThreadMode::Window(3), 10) // Window mode with 3 lines, 10 total jobs
    );
    
    // Process tasks and log with templates
    for i in 0..10 {
        // Create template with progress data
        let template = ProgressTemplate::new(
            "Job {i}: {progress:bar:15} {progress:percent} - {state}"
        );
        
        // Set up template context
        let mut ctx = TemplateContext::new();
        ctx.set("i", i + 1)
           .set("progress", (i + 1) as f64 / 10.0)
           .set("state", if i < 9 { "Processing" } else { "Completed" });
        
        // Render and log the template
        let message = template.render(&ctx).unwrap();
        logger.log(message).await;
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
```

#### Customizing Task Display with Templates

You can create specialized progress displays for different types of tasks:

```rust
// Create templates for different task types
let file_template = ProgressTemplate::new(
    "File {filename}: {progress:bar:block:15} {bytes:lpad:8}/{total_bytes:lpad:8} ({progress:percent})"
);

let network_template = ProgressTemplate::new(
    "Network {host}: {progress:bar:spinner} {status} - {speed:rpad:10}"
);

let processing_template = ProgressTemplate::new(
    "{?completed}✅{/}{!completed}⏳{/} Processing: {progress:bar:custom:gradient:20} {items_done}/{items_total}"
);

// Use the appropriate template for each task type
async fn log_task_progress(logger: &mut ThreadLogger, task_type: &str, data: &HashMap<String, TemplateVar>) {
    let template = match task_type {
        "file" => &file_template,
        "network" => &network_template,
        "processing" => &processing_template,
        _ => panic!("Unknown task type"),
    };
    
    // Create context from data
    let mut ctx = TemplateContext::new();
    for (k, v) in data {
        ctx.set(k.clone(), v.clone());
    }
    
    let message = template.render(&ctx).unwrap();
    logger.log(message).await;
}
```

#### Using Template Presets with ThreadLogger

For common progress formats, you can use the predefined template presets:

```rust
use nt_progress::formatter::TemplatePreset;

// Create context for a download task
let mut ctx = TemplateContext::new();
ctx.set("filename", "large-file.zip")
   .set("progress", 0.45)
   .set("bytes_done", "450 MB")
   .set("bytes_total", "1 GB");

// Use the download progress preset
let template = TemplatePreset::DownloadProgress.create_template();
let message = template.render(&ctx).unwrap();

// Log the message
logger.log(message).await;
```

#### Template-based Progress UI in Window with Title Mode

When using `WindowWithTitle` mode, you can create rich progress UIs:

```rust
// Create a task with title and formatted content
let mut task = progress.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "Task Progress").await?;

// Use templates for both the title and the progress content
async fn update_task(task: &mut Task, progress_value: f64, current: usize, total: usize) -> Result<(), ProgressError> {
    // Title template with emoji
    let title_template = ProgressTemplate::new("{icon} Task {id}: {status}");
    let mut title_ctx = TemplateContext::new();
    title_ctx.set("icon", if progress_value >= 1.0 { "✅" } else { "⏳" })
            .set("id", task.thread_id())
            .set("status", if progress_value >= 1.0 { "Complete" } else { "In Progress" });
    
    // Content template with progress bar
    let content_template = ProgressTemplate::new(
        "Progress: {progress:bar:block:20} {progress:percent}\n" +
        "Items: {current}/{total}\n" +
        "Estimated time remaining: {eta}"
    );
    
    let mut content_ctx = TemplateContext::new();
    content_ctx.set("progress", progress_value)
              .set("current", current)
              .set("total", total)
              .set("eta", format!("{:.1} seconds", (total - current) as f64 * 0.5));
    
    // Update the task title and content
    task.set_title(title_template.render(&title_ctx)?).await?;
    task.log(content_template.render(&content_ctx)?).await?;
    
    Ok(())
}
```

### Template System Best Practices

To get the most out of the templating system while maintaining good performance, consider these best practices:

#### Performance Considerations

1. **Cache Templates:** Create templates once and reuse them rather than creating new template instances for each message:

   ```rust
   // GOOD: Create once and reuse
   let template = ProgressTemplate::new("Progress: {progress:bar} {progress:percent}");
   
   for i in 0..100 {
       let progress = i as f64 / 100.0;
       let mut ctx = TemplateContext::new();
       ctx.set("progress", progress);
       logger.log(template.render(&ctx).unwrap()).await;
   }
   
   // AVOID: Creating new template for each message
   for i in 0..100 {
       let progress = i as f64 / 100.0;
       let template = ProgressTemplate::new("Progress: {progress:bar} {progress:percent}");
       // ...
   }
   ```

2. **Reuse Contexts:** For efficiency, consider resetting and reusing context objects:

   ```rust
   let mut ctx = TemplateContext::new();
   
   for i in 0..100 {
       // Clear previous values (if needed)
       ctx = TemplateContext::new();
       
       // Set new values
       ctx.set("progress", i as f64 / 100.0)
          .set("completed", i)
          .set("total", 100);
          
       // Render and log
       logger.log(template.render(&ctx).unwrap()).await;
   }
   ```

3. **Avoid Complex Conditional Logic:** While the template system supports conditionals, complex logic should be handled in your Rust code:

   ```rust
   // GOOD: Handle complex logic in code
   let status = if success {
       "✅ Completed successfully"
   } else if partial_success {
       "⚠️ Completed with warnings"
   } else {
       "❌ Failed"
   };
   
   ctx.set("status", status);
   
   // AVOID: Complex nested conditionals in templates
   // "{?success}✅ Completed successfully{/}{!success}{?partial_success}⚠️ Completed with warnings{/}{!partial_success}❌ Failed{/}{/}"
   ```

#### Readability and Maintenance

1. **Use Named Presets for Common Patterns:**

   ```rust
   // Define common templates in one place
   struct AppTemplates {
       progress: ProgressTemplate,
       error: ProgressTemplate,
       summary: ProgressTemplate,
   }
   
   impl AppTemplates {
       fn new() -> Self {
           Self {
               progress: ProgressTemplate::new("Progress: {progress:bar:15} {progress:percent}"),
               error: ProgressTemplate::new("❌ Error: {message}"),
               summary: ProgressTemplate::new("✅ Processed {success} items, failed: {failed}"),
           }
       }
   }
   
   // Use throughout application
   let templates = AppTemplates::new();
   logger.log(templates.progress.render(&ctx).unwrap()).await;
   ```

2. **Document Variable Requirements:**

   ```rust
   /// Creates a progress template for file operations
   /// 
   /// Required context variables:
   /// - filename: String - Name of the file
   /// - progress: f64 (0.0-1.0) - Progress of the operation
   /// - bytes_done: String - Amount of processed data
   /// - bytes_total: String - Total amount of data
   fn create_file_template() -> ProgressTemplate {
       ProgressTemplate::new(
           "File {filename} {progress:bar:15} {bytes_done}/{bytes_total} ({progress:percent})"
       )
   }
   ```

#### Error Handling

Always handle template rendering errors properly:

```rust
match template.render(&ctx) {
    Ok(message) => {
        logger.log(message).await;
    },
    Err(err) => {
        // Fall back to a simple message on template error
        logger.log(format!("Progress: {}%", (progress * 100.0) as u8)).await;
        eprintln!("Template error: {}", err);
    }
}
```

#### Terminal Compatibility

Be mindful of terminal capabilities when using advanced formatting:

```rust
// Check if the terminal supports unicode and color
if terminal_supports_unicode() && terminal_supports_color() {
    // Use fancy template with blocks and colors
    template = ProgressTemplate::new(
        "Progress: {progress:bar:block:15:red:blue} {progress:percent}"
    );
} else {
    // Use simple ASCII template
    template = ProgressTemplate::new(
        "Progress: {progress:bar:bar:=:_:15} {progress:percent}"
    );
}
```

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