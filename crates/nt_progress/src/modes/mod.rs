use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::fmt::Debug;
use std::collections::VecDeque;
use std::any::Any;
use crate::errors::ModeCreationError;

mod limited;
mod capturing;
mod window;
mod window_with_title;
pub use limited::Limited;
pub use capturing::Capturing;
pub use window::Window;
pub use window_with_title::WindowWithTitle;

/// Trait defining the behavior of different display modes.
///
/// This trait is the core interface that all display modes must implement.
/// It defines how messages are processed and displayed in the terminal.
///
/// # Implementing a New Mode
///
/// When implementing a new mode, you should:
/// 1. Create a struct that extends WindowBase or SingleLineBase
/// 2. Implement this trait for your struct
/// 3. Add your mode to the ThreadMode enum
/// 4. Update Config::new to handle your mode
///
/// See the README.md file in this directory for a complete example.
pub trait ThreadConfig: Send + Sync + Debug {
    /// Returns the number of lines this mode needs to display.
    ///
    /// This method is used to determine the height of the display area
    /// needed by this mode. It should return a consistent value that
    /// doesn't change during the lifetime of the config.
    fn lines_to_display(&self) -> usize;

    /// Processes a new message and returns the lines to display.
    ///
    /// This method is called whenever a new message is received. It should:
    /// 1. Update the internal state based on the message
    /// 2. Return the lines that should be displayed
    ///
    /// # Parameters
    /// * `message` - The message to process
    ///
    /// # Returns
    /// A vector of strings representing the lines to display
    fn handle_message(&mut self, message: String) -> Vec<String>;

    /// Returns the current lines to display without processing a new message.
    ///
    /// This method should return the current state of the display without
    /// modifying any internal state.
    ///
    /// # Returns
    /// A vector of strings representing the lines to display
    fn get_lines(&self) -> Vec<String>;

    /// Returns a boxed clone of this config.
    ///
    /// This method is used to create a clone of the config for use in
    /// multiple threads. It should return a boxed clone of the implementing
    /// struct.
    ///
    /// # Returns
    /// A boxed clone of this config as a ThreadConfig trait object
    fn clone_box(&self) -> Box<dyn ThreadConfig>;
    
    /// Returns this config as a mutable Any reference for downcasting.
    ///
    /// This method is used to downcast the config to a specific implementation
    /// type when you need to access implementation-specific methods.
    ///
    /// # Returns
    /// A mutable reference to self as a mutable Any
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Trait for tracking job progress across different display modes.
///
/// This trait is implemented by display modes to track the progress
/// of jobs being processed. It provides methods for getting the total
/// number of jobs and incrementing the completed jobs counter.
pub trait JobTracker: Send + Sync + Debug {
    /// Get the total number of jobs assigned to this tracker.
    ///
    /// # Returns
    /// The total number of jobs
    fn get_total_jobs(&self) -> usize;
    
    /// Increment the completed jobs counter and return the new value.
    ///
    /// This method is used to mark a job as completed and get the
    /// new count of completed jobs.
    ///
    /// # Returns
    /// The new count of completed jobs
    fn increment_completed_jobs(&self) -> usize;
}

/// Base implementation for window-based display modes.
///
/// WindowBase provides a fixed-size scrolling window of output lines
/// with automatic management of line limits. It's the foundation for
/// modes that display multiple lines simultaneously.
#[derive(Debug, Clone)]
pub struct WindowBase {
    base: BaseConfig,
    lines: VecDeque<String>,
    max_lines: usize,
}

impl WindowBase {
    /// Create a new WindowBase with the specified number of total jobs and maximum lines.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    /// * `max_lines` - The maximum number of lines to display
    ///
    /// # Returns
    /// A Result containing either the new WindowBase or a ModeCreationError
    ///
    /// # Errors
    /// Returns an InvalidWindowSize error if max_lines is 0
    pub fn new(total_jobs: usize, max_lines: usize) -> Result<Self, ModeCreationError> {
        if max_lines == 0 {
            return Err(ModeCreationError::InvalidWindowSize {
                size: max_lines,
                min_size: 1,
                mode_name: "WindowBase".to_string(),
            });
        }
        Ok(Self {
            base: BaseConfig::new(total_jobs),
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
        })
    }
    
    /// Add a message to the window.
    ///
    /// Adds the message to the end of the window and removes lines from
    /// the front if the number of lines exceeds max_lines.
    ///
    /// # Parameters
    /// * `message` - The message to add to the window
    pub fn add_message(&mut self, message: String) {
        // Add new line to the end
        self.lines.push_back(message);
        
        // Remove lines from the front if we exceed max_lines
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
    }
    
    /// Get the current lines in the window.
    ///
    /// # Returns
    /// A vector of strings representing the current lines
    pub fn get_lines(&self) -> Vec<String> {
        self.lines.iter().cloned().collect()
    }
    
    /// Get the maximum number of lines this window can display.
    ///
    /// # Returns
    /// The maximum number of lines
    pub fn max_lines(&self) -> usize {
        self.max_lines
    }
}

// Implement JobTracker for WindowBase
impl JobTracker for WindowBase {
    fn get_total_jobs(&self) -> usize {
        self.base.get_total_jobs()
    }
    
    fn increment_completed_jobs(&self) -> usize {
        self.base.increment_completed_jobs()
    }
}

/// Base implementation for single-line display modes.
///
/// SingleLineBase provides a foundation for modes that display a single
/// line of output, with optional passthrough to stdout/stderr.
#[derive(Debug, Clone)]
pub struct SingleLineBase {
    base: BaseConfig,
    current_line: String,
    passthrough: bool,
}

impl SingleLineBase {
    /// Create a new SingleLineBase with the specified number of total jobs
    /// and passthrough mode (whether to send output to stdout/stderr).
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    /// * `passthrough` - Whether to pass output through to stdout/stderr
    ///
    /// # Returns
    /// A new SingleLineBase instance
    pub fn new(total_jobs: usize, passthrough: bool) -> Self {
        Self {
            base: BaseConfig::new(total_jobs),
            current_line: String::new(),
            passthrough,
        }
    }
    
    /// Update the current line.
    ///
    /// # Parameters
    /// * `message` - The new line to display
    pub fn update_line(&mut self, message: String) {
        self.current_line = message;
    }
    
    /// Get the current line.
    ///
    /// # Returns
    /// The current line as a String
    pub fn get_line(&self) -> String {
        self.current_line.clone()
    }
    
    /// Check if this mode passes output through to stdout/stderr.
    ///
    /// # Returns
    /// true if passthrough is enabled, false otherwise
    pub fn has_passthrough(&self) -> bool {
        self.passthrough
    }
}

// Implement JobTracker for SingleLineBase
impl JobTracker for SingleLineBase {
    fn get_total_jobs(&self) -> usize {
        self.base.get_total_jobs()
    }
    
    fn increment_completed_jobs(&self) -> usize {
        self.base.increment_completed_jobs()
    }
}

/// Wrapper struct for ThreadConfig implementations.
///
/// Config provides a unified interface to different thread configuration
/// implementations, along with factory methods for creating configurations
/// based on the desired ThreadMode.
#[derive(Debug)]
pub struct Config {
    config: Box<dyn ThreadConfig>,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone_box(),
        }
    }
}

impl Config {
    /// Create a new Config with the specified mode and total jobs.
    ///
    /// # Parameters
    /// * `mode` - The display mode to use
    /// * `total_jobs` - The total number of jobs to track
    ///
    /// # Returns
    /// A Result containing either the new Config or a ModeCreationError
    ///
    /// # Errors
    /// Returns an error if the mode's constructor returns an error
    pub fn new(mode: ThreadMode, total_jobs: usize) -> Result<Self, ModeCreationError> {
        let config: Box<dyn ThreadConfig> = match mode {
            ThreadMode::Limited => Box::new(Limited::new(total_jobs)),
            ThreadMode::Capturing => Box::new(Capturing::new(total_jobs)),
            ThreadMode::Window(max_lines) => Box::new(Window::new(total_jobs, max_lines)?),
            ThreadMode::WindowWithTitle(max_lines) => Box::new(WindowWithTitle::new(total_jobs, max_lines)?),
        };

        Ok(Self { config })
    }

    /// Get the number of lines this config needs to display.
    ///
    /// # Returns
    /// The number of lines needed by this configuration
    pub fn lines_to_display(&self) -> usize {
        self.config.lines_to_display()
    }

    /// Process a message and return the lines to display.
    ///
    /// # Parameters
    /// * `message` - The message to process
    ///
    /// # Returns
    /// A vector of strings representing the lines to display
    pub fn handle_message(&mut self, message: String) -> Vec<String> {
        self.config.handle_message(message)
    }

    /// Get the current lines to display.
    ///
    /// # Returns
    /// A vector of strings representing the current lines
    pub fn get_lines(&self) -> Vec<String> {
        self.config.get_lines()
    }

    /// Returns a mutable reference to the WindowWithTitle implementation if available
    ///
    /// # Returns
    /// `Some(&mut WindowWithTitle)` if the config is in WindowWithTitle mode, 
    /// `None` otherwise
    pub fn as_window_with_title_mut(&mut self) -> Option<&mut WindowWithTitle> {
        self.config.as_any_mut().downcast_mut::<WindowWithTitle>()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(ThreadMode::Limited, 1).unwrap()
    }
}

/// Enum representing the different display modes.
///
/// Each variant represents a different way of displaying output:
/// - Limited: Shows only the most recent message
/// - Capturing: Captures output without displaying
/// - Window: Shows the last N messages in a scrolling window
/// - WindowWithTitle: Shows a window with a title bar
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreadMode {
    /// Limited mode shows only the most recent message.
    Limited,
    
    /// Capturing mode captures output without displaying.
    Capturing,
    
    /// Window mode shows the last N messages in a scrolling window.
    /// The parameter specifies the maximum number of lines.
    Window(usize),
    
    /// WindowWithTitle mode shows a window with a title bar.
    /// The parameter specifies the maximum number of lines including the title.
    WindowWithTitle(usize),
}

/// Common configuration shared across all modes.
///
/// BaseConfig provides basic job tracking functionality that
/// can be reused by different mode implementations.
#[derive(Debug, Clone)]
pub struct BaseConfig {
    total_jobs: usize,
    completed_jobs: Arc<AtomicUsize>,
}

impl BaseConfig {
    /// Create a new BaseConfig with the specified number of total jobs.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    ///
    /// # Returns
    /// A new BaseConfig instance
    pub fn new(total_jobs: usize) -> Self {
        Self {
            total_jobs,
            completed_jobs: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get the total number of jobs.
    ///
    /// # Returns
    /// The total number of jobs
    pub fn get_total_jobs(&self) -> usize {
        self.total_jobs
    }

    /// Increment the completed jobs counter and return the new value.
    ///
    /// # Returns
    /// The new count of completed jobs
    pub fn increment_completed_jobs(&self) -> usize {
        self.completed_jobs.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    }
}

// Implement JobTracker for BaseConfig
impl JobTracker for BaseConfig {
    fn get_total_jobs(&self) -> usize {
        self.total_jobs
    }
    
    fn increment_completed_jobs(&self) -> usize {
        self.completed_jobs.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    }
}

/// Factory function to create a ThreadConfig implementation from a ThreadMode.
/// 
/// This function provides a consistent way to create ThreadConfig instances
/// with built-in error handling and graceful fallback logic.
///
/// # Parameters
/// * `mode` - The display mode to use
/// * `total_jobs` - The total number of jobs to track
///
/// # Returns
/// A boxed ThreadConfig implementation
///
/// # Error Recovery
/// This function will never panic. If the requested mode cannot be created:
/// 1. First tries with a reasonable default size for the requested mode
/// 2. If that fails, falls back to a simpler mode (Window -> Limited)
/// 3. Always guarantees a working mode will be returned
pub fn create_thread_config(mode: ThreadMode, total_jobs: usize) -> Box<dyn ThreadConfig> {
    match mode {
        ThreadMode::Limited => Box::new(Limited::new(total_jobs)),
        ThreadMode::Capturing => Box::new(Capturing::new(total_jobs)),
        ThreadMode::Window(max_lines) => {
            // Try with the requested size
            if let Ok(window) = Window::new(total_jobs, max_lines) {
                return Box::new(window);
            }
            
            // Try with a reasonable fallback size
            if let Ok(window) = Window::new(total_jobs, 3) {
                eprintln!("Warning: Requested window size {} was invalid, using size 3 instead", max_lines);
                return Box::new(window);
            }
            
            // Last resort: fall back to Limited mode
            eprintln!("Warning: Could not create Window mode, falling back to Limited mode");
            Box::new(Limited::new(total_jobs))
        },
        ThreadMode::WindowWithTitle(max_lines) => {
            // Try with the requested size
            if let Ok(window) = WindowWithTitle::new(total_jobs, max_lines) {
                return Box::new(window);
            }
            
            // Try with a reasonable fallback size
            if let Ok(window) = WindowWithTitle::new(total_jobs, 3) {
                eprintln!("Warning: Requested window size {} was invalid, using size 3 instead", max_lines);
                return Box::new(window);
            }
            
            // Try with Window mode as a fallback
            if let Ok(window) = Window::new(total_jobs, 3) {
                eprintln!("Warning: Could not create WindowWithTitle mode, falling back to Window mode");
                return Box::new(window);
            }
            
            // Last resort: fall back to Limited mode
            eprintln!("Warning: Could not create any window mode, falling back to Limited mode");
            Box::new(Limited::new(total_jobs))
        },
    }
} 