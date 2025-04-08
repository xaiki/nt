use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::fmt::Debug;

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
            ThreadMode::WindowWithTitle(max_lines) => Box::new(WindowWithTitle::new(total_jobs, max_lines, String::new())),
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