use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::fmt::Debug;
use std::collections::VecDeque;

mod limited;
mod capturing;
mod window;
mod window_with_title;
pub use limited::Limited;
pub use capturing::Capturing;
pub use window::Window;
pub use window_with_title::WindowWithTitle;

/// Trait defining the behavior of different display modes
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

/// Trait for tracking job progress
pub trait JobTracker: Send + Sync + Debug {
    /// Get the total number of jobs
    fn get_total_jobs(&self) -> usize;
    
    /// Increment the completed jobs counter and return the new value
    fn increment_completed_jobs(&self) -> usize;
}

/// Base implementation for window-based display modes
#[derive(Debug, Clone)]
pub struct WindowBase {
    base: BaseConfig,
    lines: VecDeque<String>,
    max_lines: usize,
}

impl WindowBase {
    /// Create a new WindowBase with the specified number of total jobs and maximum lines
    pub fn new(total_jobs: usize, max_lines: usize) -> Result<Self, String> {
        if max_lines == 0 {
            return Err("Window size must be at least 1".to_string());
        }
        Ok(Self {
            base: BaseConfig::new(total_jobs),
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
        })
    }
    
    /// Add a message to the window
    pub fn add_message(&mut self, message: String) {
        // Add new line to the end
        self.lines.push_back(message);
        
        // Remove lines from the front if we exceed max_lines
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
    }
    
    /// Get the current lines
    pub fn get_lines(&self) -> Vec<String> {
        self.lines.iter().cloned().collect()
    }
    
    /// Get the maximum number of lines
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

/// Base implementation for single-line display modes
#[derive(Debug, Clone)]
pub struct SingleLineBase {
    base: BaseConfig,
    current_line: String,
    passthrough: bool,
}

impl SingleLineBase {
    /// Create a new SingleLineBase with the specified number of total jobs
    /// and passthrough mode (whether to send output to stdout/stderr)
    pub fn new(total_jobs: usize, passthrough: bool) -> Self {
        Self {
            base: BaseConfig::new(total_jobs),
            current_line: String::new(),
            passthrough,
        }
    }
    
    /// Update the current line
    pub fn update_line(&mut self, message: String) {
        self.current_line = message;
    }
    
    /// Get the current line
    pub fn get_line(&self) -> String {
        self.current_line.clone()
    }
    
    /// Check if this mode passes output through to stdout/stderr
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

/// Wrapper struct for ThreadConfig implementations
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
    pub fn new(mode: ThreadMode, total_jobs: usize) -> Result<Self, String> {
        let config: Box<dyn ThreadConfig> = match mode {
            ThreadMode::Limited => Box::new(Limited::new(total_jobs)),
            ThreadMode::Capturing => Box::new(Capturing::new(total_jobs)),
            ThreadMode::Window(max_lines) => Box::new(Window::new(total_jobs, max_lines)?),
            ThreadMode::WindowWithTitle(max_lines) => Box::new(WindowWithTitle::new(total_jobs, max_lines)?),
        };

        Ok(Self { config })
    }

    pub fn lines_to_display(&self) -> usize {
        self.config.lines_to_display()
    }

    pub fn handle_message(&mut self, message: String) -> Vec<String> {
        self.config.handle_message(message)
    }

    pub fn get_lines(&self) -> Vec<String> {
        self.config.get_lines()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(ThreadMode::Limited, 1).unwrap()
    }
}

/// Enum representing the different display modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreadMode {
    Limited,
    Capturing,
    Window(usize),
    WindowWithTitle(usize),
}

/// Common configuration shared across all modes
#[derive(Debug, Clone)]
pub struct BaseConfig {
    total_jobs: usize,
    completed_jobs: Arc<AtomicUsize>,
}

impl BaseConfig {
    pub fn new(total_jobs: usize) -> Self {
        Self {
            total_jobs,
            completed_jobs: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_total_jobs(&self) -> usize {
        self.total_jobs
    }

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

/// Factory function to create a ThreadConfig implementation from a ThreadMode
pub fn create_thread_config(mode: ThreadMode, total_jobs: usize) -> Box<dyn ThreadConfig> {
    match mode {
        ThreadMode::Limited => Box::new(Limited::new(total_jobs)),
        ThreadMode::Capturing => Box::new(Capturing::new(total_jobs)),
        ThreadMode::Window(max_lines) => Box::new(Window::new(total_jobs, max_lines).unwrap_or_else(|err| {
            eprintln!("Error creating Window mode: {}", err);
            Window::new(total_jobs, 3).unwrap()
        })),
        ThreadMode::WindowWithTitle(max_lines) => Box::new(WindowWithTitle::new(total_jobs, max_lines).unwrap_or_else(|err| {
            eprintln!("Error creating WindowWithTitle mode: {}", err);
            // Fallback to a reasonable size if there was an error
            WindowWithTitle::new(total_jobs, 3).unwrap()
        })),
    }
} 