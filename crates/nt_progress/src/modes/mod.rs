use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::fmt::Debug;
use std::collections::VecDeque;
use std::any::Any;
use crate::errors::ModeCreationError;
use std::any::TypeId;

mod limited;
mod capturing;
mod window;
mod window_with_title;
mod factory;

pub use limited::Limited;
pub use capturing::Capturing;
pub use window::Window;
pub use window_with_title::WindowWithTitle;
pub use factory::{ModeRegistry, ModeCreator, get_registry, create_thread_config, set_error_propagation};

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
    
    /// Returns this config as an Any reference for downcasting.
    ///
    /// This method is used to downcast the config to a specific implementation
    /// type when you need to access implementation-specific methods.
    ///
    /// # Returns
    /// A reference to self as an Any
    fn as_any(&self) -> &dyn Any;
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
    
    /// Set the total number of jobs for this tracker.
    ///
    /// This method is used to update the total number of jobs
    /// when it was not known at creation time or has changed.
    ///
    /// # Parameters
    /// * `total` - The new total number of jobs
    fn set_total_jobs(&mut self, total: usize);
}

/// Trait for types that contain a BaseConfig, either directly or through composition.
///
/// This trait is used to provide a uniform way to access the BaseConfig
/// of different types, which enables generic implementations for traits
/// like JobTracker.
pub trait HasBaseConfig {
    /// Get a reference to the BaseConfig.
    ///
    /// # Returns
    /// A reference to the BaseConfig
    fn base_config(&self) -> &BaseConfig;
    
    /// Get a mutable reference to the BaseConfig.
    ///
    /// # Returns
    /// A mutable reference to the BaseConfig
    fn base_config_mut(&mut self) -> &mut BaseConfig;
}

/// Trait for types that can have a title.
///
/// This capability is implemented by modes that support setting and retrieving a title,
/// such as WindowWithTitle mode.
pub trait WithTitle: Send + Sync {
    /// Set the title for this config.
    fn set_title(&mut self, title: String);
    
    /// Get the current title.
    fn get_title(&self) -> &str;
}

/// Trait for types that can have a custom size.
///
/// This capability is implemented by modes that support custom sizes,
/// such as Window and WindowWithTitle.
pub trait WithCustomSize: Send + Sync {
    /// Set the maximum number of lines to display.
    fn set_max_lines(&mut self, max_lines: usize) -> Result<(), ModeCreationError>;
    
    /// Get the maximum number of lines that can be displayed.
    fn get_max_lines(&self) -> usize;
}

/// Trait for types that support emoji display.
///
/// This capability is implemented by modes that can display emoji characters.
pub trait WithEmoji: Send + Sync {
    /// Add an emoji to the display.
    fn add_emoji(&mut self, emoji: &str);
    
    /// Get the current emojis.
    fn get_emojis(&self) -> Vec<String>;
}

/// Extension methods for ThreadConfig to access capabilities.
pub trait ThreadConfigExt: ThreadConfig {
    /// Check if this config supports the WithTitle capability.
    fn supports_title(&self) -> bool {
        self.as_any().type_id() == TypeId::of::<WindowWithTitle>()
    }
    
    /// Try to get this config as a WithTitle.
    fn as_title(&self) -> Option<&dyn WithTitle> {
        // WindowWithTitle is currently the only implementation of WithTitle
        self.as_any().downcast_ref::<WindowWithTitle>().map(|w| w as &dyn WithTitle)
    }
    
    /// Try to get this config as a mutable WithTitle.
    fn as_title_mut(&mut self) -> Option<&mut dyn WithTitle> {
        // WindowWithTitle is currently the only implementation of WithTitle
        self.as_any_mut().downcast_mut::<WindowWithTitle>().map(|w| w as &mut dyn WithTitle)
    }
    
    /// Check if this config supports the WithCustomSize capability.
    fn supports_custom_size(&self) -> bool {
        let type_id = self.as_any().type_id();
        type_id == TypeId::of::<Window>() || type_id == TypeId::of::<WindowWithTitle>()
    }
    
    /// Try to get this config as a WithCustomSize.
    fn as_custom_size(&self) -> Option<&dyn WithCustomSize> {
        if let Some(w) = self.as_any().downcast_ref::<Window>() {
            Some(w as &dyn WithCustomSize)
        } else if let Some(w) = self.as_any().downcast_ref::<WindowWithTitle>() {
            Some(w as &dyn WithCustomSize)
        } else {
            None
        }
    }
    
    /// Try to get this config as a mutable WithCustomSize.
    fn as_custom_size_mut(&mut self) -> Option<&mut dyn WithCustomSize> {
        let type_id = self.as_any().type_id();
        
        if type_id == TypeId::of::<Window>() {
            // It's a Window, do the downcast
            self.as_any_mut().downcast_mut::<Window>()
                .map(|w| w as &mut dyn WithCustomSize)
        } else if type_id == TypeId::of::<WindowWithTitle>() {
            // It's a WindowWithTitle, do the downcast
            self.as_any_mut().downcast_mut::<WindowWithTitle>()
                .map(|w| w as &mut dyn WithCustomSize)
        } else {
            // Neither type matched
            None
        }
    }
    
    /// Check if this config supports the WithEmoji capability.
    fn supports_emoji(&self) -> bool {
        // No implementations yet
        false
    }
    
    /// Try to get this config as a WithEmoji.
    fn as_emoji(&self) -> Option<&dyn WithEmoji> {
        // No implementations yet
        None
    }
    
    /// Try to get this config as a mutable WithEmoji.
    fn as_emoji_mut(&mut self) -> Option<&mut dyn WithEmoji> {
        // No implementations yet
        None
    }
}

// Implement ThreadConfigExt for all types that implement ThreadConfig
impl<T: ThreadConfig + ?Sized> ThreadConfigExt for T {}

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
    /// Create a new Config from a ThreadMode and total jobs count
    ///
    /// # Parameters
    /// * `mode` - The display mode to use
    /// * `total_jobs` - The total number of jobs to track
    ///
    /// # Returns
    /// A Result containing either the new Config or a ModeCreationError
    ///
    /// # Examples
    /// ```
    /// use nt_progress::modes::{Config, ThreadMode};
    ///
    /// let config = Config::new(ThreadMode::Limited, 10).unwrap();
    /// let config = Config::new(ThreadMode::Window(3), 10).unwrap();
    /// ```
    pub fn new(mode: ThreadMode, total_jobs: usize) -> Result<Self, ModeCreationError> {
        // Use the factory to create the thread config
        let config = factory::create_thread_config(mode, total_jobs)?;
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

    /// Returns a reference to a specific implementation type.
    ///
    /// This is a generic method that can be used to access any implementation
    /// type that is stored in this Config.
    ///
    /// # Type Parameters
    /// * `T` - The implementation type to downcast to
    ///
    /// # Returns
    /// `Some(&T)` if the config is of type T, `None` otherwise
    pub fn as_type<T: 'static>(&self) -> Option<&T> {
        self.config.as_any().downcast_ref::<T>()
    }

    /// Returns a mutable reference to a specific implementation type.
    ///
    /// This is a generic method that can be used to access any implementation
    /// type that is stored in this Config. It replaces the type-specific
    /// methods like `as_window_mut` and `as_window_with_title_mut`.
    ///
    /// # Type Parameters
    /// * `T` - The implementation type to downcast to
    ///
    /// # Returns
    /// `Some(&mut T)` if the config is of type T, `None` otherwise
    pub fn as_type_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.config.as_any_mut().downcast_mut::<T>()
    }

    /// Set the total number of jobs for this configuration.
    ///
    /// This method updates the total number of jobs in the underlying
    /// ThreadConfig implementation, if it implements the JobTracker trait.
    ///
    /// # Parameters
    /// * `total` - The new total number of jobs
    pub fn set_total_jobs(&mut self, total: usize) {
        // Use HasBaseConfig trait which all our mode types implement
        let config_any_mut = self.config.as_any_mut();
        
        // Try to use any type that implements HasBaseConfig
        macro_rules! try_as_has_base_config {
            ($type:ty) => {
                if let Some(config) = config_any_mut.downcast_mut::<$type>() {
                    config.set_total_jobs(total);
                    return;
                }
            };
        }
        
        // Try each known implementation
        try_as_has_base_config!(Window);
        try_as_has_base_config!(WindowWithTitle);
        try_as_has_base_config!(Limited);
        try_as_has_base_config!(Capturing);
        
        // If we get here, we couldn't update the total_jobs
        // Future implementations should add their types to the list above
        eprintln!("Warning: Could not update total_jobs. Unknown mode type that doesn't implement JobTracker.");
    }

    /// Check if this config supports setting a title
    pub fn supports_title(&self) -> bool {
        self.config.supports_title()
    }
    
    /// Set the title for this config if it supports titles
    pub fn set_title(&mut self, title: String) -> Result<(), ModeCreationError> {
        if let Some(with_title) = self.config.as_title_mut() {
            with_title.set_title(title);
            Ok(())
        } else {
            Err(ModeCreationError::Implementation(
                format!("Config does not support titles")
            ))
        }
    }
    
    /// Get the title for this config if it supports titles
    pub fn get_title(&self) -> Option<&str> {
        self.config.as_title().map(|t| t.get_title())
    }
    
    /// Check if this config supports custom size
    pub fn supports_custom_size(&self) -> bool {
        self.config.supports_custom_size()
    }
    
    /// Set the maximum number of lines for this config if it supports custom size
    pub fn set_max_lines(&mut self, max_lines: usize) -> Result<(), ModeCreationError> {
        if let Some(with_size) = self.config.as_custom_size_mut() {
            with_size.set_max_lines(max_lines)
        } else {
            Err(ModeCreationError::Implementation(
                format!("Config does not support custom size")
            ))
        }
    }
    
    /// Get the maximum number of lines for this config if it supports custom size
    pub fn get_max_lines(&self) -> Option<usize> {
        self.config.as_custom_size().map(|s| s.get_max_lines())
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
    
    /// Set the total number of jobs for this configuration.
    ///
    /// # Parameters
    /// * `total` - The new total number of jobs
    pub fn set_total_jobs(&mut self, total: usize) {
        self.total_jobs = total;
    }
}

// Implement HasBaseConfig for BaseConfig itself (trivial case)
impl HasBaseConfig for BaseConfig {
    fn base_config(&self) -> &BaseConfig {
        self
    }
    
    fn base_config_mut(&mut self) -> &mut BaseConfig {
        self
    }
}

// Implement HasBaseConfig for WindowBase
impl HasBaseConfig for WindowBase {
    fn base_config(&self) -> &BaseConfig {
        &self.base
    }
    
    fn base_config_mut(&mut self) -> &mut BaseConfig {
        &mut self.base
    }
}

// Implement HasBaseConfig for SingleLineBase
impl HasBaseConfig for SingleLineBase {
    fn base_config(&self) -> &BaseConfig {
        &self.base
    }
    
    fn base_config_mut(&mut self) -> &mut BaseConfig {
        &mut self.base
    }
}

// Blanket implementation of JobTracker for any type that implements HasBaseConfig
impl<T: HasBaseConfig + Send + Sync + Debug> JobTracker for T {
    fn get_total_jobs(&self) -> usize {
        self.base_config().get_total_jobs()
    }
    
    fn increment_completed_jobs(&self) -> usize {
        self.base_config().increment_completed_jobs()
    }
    
    fn set_total_jobs(&mut self, total: usize) {
        self.base_config_mut().set_total_jobs(total);
    }
} 