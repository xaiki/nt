use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::fmt::Debug;
use std::collections::{VecDeque, HashSet, HashMap};
use std::any::{Any, TypeId};
use crate::errors::ModeCreationError;
use crate::io::ProgressWriter;

pub mod limited;
pub mod capturing;
pub mod window;
pub mod window_with_title;
pub mod factory;

pub use limited::Limited;
pub use capturing::Capturing;
pub use window::Window;
pub use window_with_title::WindowWithTitle;
pub use factory::{ModeRegistry, ModeCreator, ModeFactory, set_error_propagation};

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

/// Trait for modes that support output passthrough
pub trait WithPassthrough {
    /// Enable or disable passthrough mode
    fn set_passthrough(&mut self, enabled: bool);
    
    /// Check if passthrough is enabled
    fn has_passthrough(&self) -> bool;
    
    /// Get a mutable reference to the current passthrough writer
    fn get_passthrough_writer_mut(&mut self) -> Option<&mut dyn ProgressWriter>;
    
    /// Set a custom passthrough writer
    fn set_passthrough_writer(&mut self, writer: Box<dyn ProgressWriter + Send + 'static>) -> Result<(), ModeCreationError>;
}

/// Core capabilities for display modes.
/// 
/// This module groups the core capabilities that display modes can implement.
/// Each capability is represented by a trait that defines a specific set of
/// functionality a mode can provide.
pub mod capabilities {
    use super::*;

    /// Capability for modes that can have a title.
    ///
    /// # Examples
    /// ```rust
    /// # use nt_progress::modes::{WindowWithTitle, WithTitle};
    /// let mut mode = WindowWithTitle::new(10, 5, "Initial Title".to_string())?;
    /// mode.set_title("New Title".to_string())?;
    /// assert_eq!(mode.get_title(), "New Title");
    /// # Ok::<(), ModeCreationError>(())
    /// ```
    pub trait WithTitle: Send + Sync {
        /// Set the title for this config.
        ///
        /// # Errors
        /// Returns `ModeCreationError` if the title is invalid or cannot be set.
        fn set_title(&mut self, title: String) -> Result<(), ModeCreationError>;
        
        /// Get the current title.
        ///
        /// # Returns
        /// A reference to the current title string.
        fn get_title(&self) -> &str;
    }

    /// Capability for modes that can have a custom display size.
    ///
    /// # Examples
    /// ```rust
    /// # use nt_progress::modes::{Window, WithCustomSize};
    /// let mut mode = Window::new(10, 5)?;
    /// mode.set_max_lines(8)?;
    /// assert_eq!(mode.get_max_lines(), 8);
    /// # Ok::<(), ModeCreationError>(())
    /// ```
    pub trait WithCustomSize: Send + Sync {
        /// Set the maximum number of lines to display.
        ///
        /// # Errors
        /// Returns `ModeCreationError` if the size is invalid.
        fn set_max_lines(&mut self, max_lines: usize) -> Result<(), ModeCreationError>;
        
        /// Get the maximum number of lines that can be displayed.
        fn get_max_lines(&self) -> usize;
    }

    /// Capability for modes that support emoji display.
    ///
    /// # Examples
    /// ```rust
    /// # use nt_progress::modes::{WindowWithTitle, WithEmoji};
    /// let mut mode = WindowWithTitle::new(10, 5, "Title".to_string())?;
    /// mode.add_emoji("✨")?;
    /// assert_eq!(mode.get_emojis(), vec!["✨".to_string()]);
    /// # Ok::<(), ModeCreationError>(())
    /// ```
    pub trait WithEmoji: Send + Sync {
        /// Add an emoji to the display.
        ///
        /// # Errors
        /// Returns `ModeCreationError` if the emoji is invalid or cannot be added.
        fn add_emoji(&mut self, emoji: &str) -> Result<(), ModeCreationError>;
        
        /// Get the current emojis.
        fn get_emojis(&self) -> Vec<String>;
    }

    /// Composite capability for modes that support both title and emoji.
    ///
    /// This trait provides convenience methods for modes that implement both
    /// `WithTitle` and `WithEmoji` capabilities.
    ///
    /// # Examples
    /// ```rust
    /// # use nt_progress::modes::{WindowWithTitle, WithTitleAndEmoji};
    /// let mut mode = WindowWithTitle::new(10, 5, "Title".to_string())?;
    /// mode.set_title_with_emoji("New Title".to_string(), "✨")?;
    /// assert_eq!(mode.get_formatted_title(), "✨ New Title");
    /// # Ok::<(), ModeCreationError>(())
    /// ```
    pub trait WithTitleAndEmoji: WithTitle + WithEmoji {
        /// Set the title and add an emoji in a single operation.
        ///
        /// # Errors
        /// Returns `ModeCreationError` if either operation fails.
        fn set_title_with_emoji(&mut self, title: String, emoji: &str) -> Result<(), ModeCreationError> {
            self.set_title(title)?;
            self.add_emoji(emoji)
        }
        
        /// Clear all emojis and set a new title.
        ///
        /// # Errors
        /// Returns `ModeCreationError` if the title cannot be set.
        fn reset_with_title(&mut self, title: String) -> Result<(), ModeCreationError>;
        
        /// Get the fully formatted title with emojis.
        fn get_formatted_title(&self) -> String;
    }

    /// Capability for standard window operations.
    ///
    /// This trait defines the core operations that all window-based modes
    /// should support.
    ///
    /// # Examples
    /// ```rust
    /// # use nt_progress::modes::{Window, StandardWindow};
    /// let mut mode = Window::new(10, 5)?;
    /// mode.add_line("Line 1".to_string());
    /// assert_eq!(mode.line_count(), 1);
    /// mode.clear();
    /// assert!(mode.is_empty());
    /// # Ok::<(), ModeCreationError>(())
    /// ```
    pub trait StandardWindow: WithCustomSize {
        /// Clear all content from the window.
        fn clear(&mut self);
        
        /// Get the current content as a vector of strings.
        fn get_content(&self) -> Vec<String>;
        
        /// Add a single line to the window.
        fn add_line(&mut self, line: String);
        
        /// Check if the window is empty.
        fn is_empty(&self) -> bool;
        
        /// Get the number of lines currently displayed.
        fn line_count(&self) -> usize;
    }

    /// Trait for modes that support line wrapping.
    pub trait WithWrappedText: Send + Sync {
        /// Enable or disable line wrapping.
        ///
        /// # Parameters
        /// * `enabled` - Whether to enable or disable line wrapping
        fn set_line_wrapping(&mut self, enabled: bool);
        
        /// Check if line wrapping is enabled.
        ///
        /// # Returns
        /// true if line wrapping is enabled, false otherwise
        fn has_line_wrapping(&self) -> bool;
    }
}

pub use capabilities::*;

/// Enum representing the capabilities supported by different modes.
///
/// This enum is used to identify the capabilities that a mode supports
/// and enables runtime discovery of capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    /// The mode supports setting and getting a title.
    Title,
    
    /// The mode supports customizing the display size.
    CustomSize,
    
    /// The mode supports adding emoji characters.
    Emoji,
    
    /// The mode supports both title and emoji capabilities.
    TitleAndEmoji,
    
    /// The mode supports standard window operations.
    StandardWindow,

    /// The mode supports line wrapping for long text.
    WrappedText,
}

/// Extension trait providing capability checks and conversions.
///
/// This trait is automatically implemented for all types that implement
/// `ThreadConfig`. It provides methods to check for supported capabilities
/// and safely convert to capability trait objects.
pub trait ThreadConfigExt: ThreadConfig {
    /// Check if this config supports the WithTitle capability.
    ///
    /// # Returns
    /// `true` if the config supports setting and getting a title.
    fn supports_title(&self) -> bool {
        self.as_any().is::<WindowWithTitle>()
    }
    
    /// Try to get this config as a WithTitle.
    ///
    /// # Returns
    /// Some(&dyn WithTitle) if the config supports titles, None otherwise.
    fn as_title(&self) -> Option<&dyn WithTitle> {
        self.as_any().downcast_ref::<WindowWithTitle>().map(|w| w as &dyn WithTitle)
    }
    
    /// Try to get this config as a mutable WithTitle.
    ///
    /// # Returns
    /// Some(&mut dyn WithTitle) if the config supports titles, None otherwise.
    fn as_title_mut(&mut self) -> Option<&mut dyn WithTitle> {
        self.as_any_mut().downcast_mut::<WindowWithTitle>().map(|w| w as &mut dyn WithTitle)
    }
    
    /// Check if this config supports the WithCustomSize capability.
    ///
    /// # Returns
    /// `true` if the config supports custom window sizes.
    fn supports_custom_size(&self) -> bool {
        let type_id = self.as_any().type_id();
        matches!(type_id, t if t == TypeId::of::<Window>() || t == TypeId::of::<WindowWithTitle>())
    }
    
    /// Try to get this config as a WithCustomSize.
    ///
    /// # Returns
    /// Some(&dyn WithCustomSize) if the config supports custom sizes, None otherwise.
    fn as_custom_size(&self) -> Option<&dyn WithCustomSize> {
        let any = self.as_any();
        if let Some(w) = any.downcast_ref::<Window>() {
            Some(w as &dyn WithCustomSize)
        } else {
            any.downcast_ref::<WindowWithTitle>().map(|w| w as &dyn WithCustomSize)
        }
    }
    
    /// Try to get this config as a mutable WithCustomSize.
    ///
    /// # Returns
    /// Some(&mut dyn WithCustomSize) if the config supports custom sizes, None otherwise.
    fn as_custom_size_mut(&mut self) -> Option<&mut dyn WithCustomSize> {
        let type_id = self.as_any().type_id();
        let any = self.as_any_mut();
        if type_id == TypeId::of::<Window>() {
            any.downcast_mut::<Window>().map(|w| w as &mut dyn WithCustomSize)
        } else if type_id == TypeId::of::<WindowWithTitle>() {
            any.downcast_mut::<WindowWithTitle>().map(|w| w as &mut dyn WithCustomSize)
        } else {
            None
        }
    }
    
    /// Check if this config supports the WithEmoji capability.
    ///
    /// # Returns
    /// `true` if the config supports emoji display.
    fn supports_emoji(&self) -> bool {
        self.as_any().is::<WindowWithTitle>()
    }
    
    /// Try to get this config as a WithEmoji.
    ///
    /// # Returns
    /// Some(&dyn WithEmoji) if the config supports emojis, None otherwise.
    fn as_emoji(&self) -> Option<&dyn WithEmoji> {
        self.as_any().downcast_ref::<WindowWithTitle>().map(|w| w as &dyn WithEmoji)
    }
    
    /// Try to get this config as a mutable WithEmoji.
    ///
    /// # Returns
    /// Some(&mut dyn WithEmoji) if the config supports emojis, None otherwise.
    fn as_emoji_mut(&mut self) -> Option<&mut dyn WithEmoji> {
        self.as_any_mut().downcast_mut::<WindowWithTitle>().map(|w| w as &mut dyn WithEmoji)
    }
    
    /// Check if this config supports the WithTitleAndEmoji capability.
    ///
    /// # Returns
    /// `true` if the config supports both titles and emojis.
    fn supports_title_and_emoji(&self) -> bool {
        self.as_any().is::<WindowWithTitle>()
    }
    
    /// Try to get this config as a WithTitleAndEmoji.
    ///
    /// # Returns
    /// Some(&dyn WithTitleAndEmoji) if the config supports both titles and emojis, None otherwise.
    fn as_title_and_emoji(&self) -> Option<&dyn WithTitleAndEmoji> {
        self.as_any().downcast_ref::<WindowWithTitle>().map(|w| w as &dyn WithTitleAndEmoji)
    }
    
    /// Try to get this config as a mutable WithTitleAndEmoji.
    ///
    /// # Returns
    /// Some(&mut dyn WithTitleAndEmoji) if the config supports both titles and emojis, None otherwise.
    fn as_title_and_emoji_mut(&mut self) -> Option<&mut dyn WithTitleAndEmoji> {
        self.as_any_mut().downcast_mut::<WindowWithTitle>().map(|w| w as &mut dyn WithTitleAndEmoji)
    }
    
    /// Check if this config supports the StandardWindow capability.
    ///
    /// # Returns
    /// `true` if the config supports standard window operations.
    fn supports_standard_window(&self) -> bool {
        let type_id = self.as_any().type_id();
        matches!(type_id, t if t == TypeId::of::<Window>() || t == TypeId::of::<WindowWithTitle>())
    }
    
    /// Try to get this config as a StandardWindow.
    ///
    /// # Returns
    /// Some(&dyn StandardWindow) if the config supports standard window operations, None otherwise.
    fn as_standard_window(&self) -> Option<&dyn StandardWindow> {
        let any = self.as_any();
        if let Some(w) = any.downcast_ref::<Window>() {
            Some(w as &dyn StandardWindow)
        } else {
            any.downcast_ref::<WindowWithTitle>().map(|w| w as &dyn StandardWindow)
        }
    }
    
    /// Try to get this config as a mutable StandardWindow.
    ///
    /// # Returns
    /// Some(&mut dyn StandardWindow) if the config supports standard window operations, None otherwise.
    fn as_standard_window_mut(&mut self) -> Option<&mut dyn StandardWindow> {
        let type_id = self.as_any().type_id();
        let any = self.as_any_mut();
        if type_id == TypeId::of::<Window>() {
            any.downcast_mut::<Window>().map(|w| w as &mut dyn StandardWindow)
        } else if type_id == TypeId::of::<WindowWithTitle>() {
            any.downcast_mut::<WindowWithTitle>().map(|w| w as &mut dyn StandardWindow)
        } else {
            None
        }
    }

    /// Check if this config supports the WithWrappedText capability.
    ///
    /// # Returns
    /// `true` if the config supports line wrapping.
    fn supports_wrapped_text(&self) -> bool {
        let type_id = self.as_any().type_id();
        matches!(type_id, t if t == TypeId::of::<Window>() || t == TypeId::of::<WindowWithTitle>())
    }
    
    /// Try to get this config as a WithWrappedText.
    ///
    /// # Returns
    /// Some(&dyn WithWrappedText) if the config supports line wrapping, None otherwise.
    fn as_wrapped_text(&self) -> Option<&dyn WithWrappedText> {
        let any = self.as_any();
        if let Some(w) = any.downcast_ref::<Window>() {
            Some(w as &dyn WithWrappedText)
        } else {
            any.downcast_ref::<WindowWithTitle>().map(|w| w as &dyn WithWrappedText)
        }
    }
    
    /// Try to get this config as a mutable WithWrappedText.
    ///
    /// # Returns
    /// Some(&mut dyn WithWrappedText) if the config supports line wrapping, None otherwise.
    fn as_wrapped_text_mut(&mut self) -> Option<&mut dyn WithWrappedText> {
        let type_id = self.as_any().type_id();
        let any = self.as_any_mut();
        if type_id == TypeId::of::<Window>() {
            any.downcast_mut::<Window>().map(|w| w as &mut dyn WithWrappedText)
        } else if type_id == TypeId::of::<WindowWithTitle>() {
            any.downcast_mut::<WindowWithTitle>().map(|w| w as &mut dyn WithWrappedText)
        } else {
            None
        }
    }

    /// Get a set of all capabilities supported by this config.
    ///
    /// # Returns
    /// A HashSet containing all supported capabilities.
    fn capabilities(&self) -> HashSet<Capability> {
        let mut caps = HashSet::new();
        
        // Use a macro to reduce repetition
        macro_rules! add_if_supported {
            ($cap:expr, $check:expr) => {
                if $check {
                    caps.insert($cap);
                }
            };
        }
        
        add_if_supported!(Capability::Title, self.supports_title());
        add_if_supported!(Capability::CustomSize, self.supports_custom_size());
        add_if_supported!(Capability::Emoji, self.supports_emoji());
        add_if_supported!(Capability::TitleAndEmoji, self.supports_title_and_emoji());
        add_if_supported!(Capability::StandardWindow, self.supports_standard_window());
        add_if_supported!(Capability::WrappedText, self.supports_wrapped_text());
        
        caps
    }
    
    /// Check if this config supports a specific capability.
    ///
    /// # Returns
    /// `true` if the config supports the specified capability.
    fn supports_capability(&self, capability: Capability) -> bool {
        match capability {
            Capability::Title => self.supports_title(),
            Capability::CustomSize => self.supports_custom_size(),
            Capability::Emoji => self.supports_emoji(),
            Capability::TitleAndEmoji => self.supports_title_and_emoji(),
            Capability::StandardWindow => self.supports_standard_window(),
            Capability::WrappedText => self.supports_wrapped_text(),
        }
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
    thread_buffers: HashMap<String, VecDeque<String>>,
    is_threaded_mode: bool,
    line_wrapping: bool,
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
                mode_name: "Window".to_string(),
                reason: Some("Window mode requires at least 1 line to display content".to_string()),
            });
        }
        
        Ok(Self {
            base: BaseConfig::new(total_jobs),
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
            thread_buffers: HashMap::new(),
            is_threaded_mode: false,
            line_wrapping: false,
        })
    }
    
    /// Add a message to the window.
    ///
    /// # Parameters
    /// * `message` - The message to add
    pub fn add_message(&mut self, message: String) {
        if self.line_wrapping {
            // Use TextWrapper to wrap long lines
            // Default to 80 columns if we can't detect terminal size
            let terminal_width = 80;
            
            let wrapper = crate::terminal::TextWrapper::new(terminal_width);
            let wrapped_lines = wrapper.wrap(&message);
            
            // Add each wrapped line to the window
            for line in wrapped_lines {
                self.add_single_line(line);
            }
        } else {
            // No wrapping, add as a single line
            self.add_single_line(message);
        }
    }
    
    /// Add a single line to the window without wrapping.
    ///
    /// # Parameters
    /// * `line` - The line to add
    fn add_single_line(&mut self, line: String) {
        // If we're in threaded mode, add to thread buffer
        if self.is_threaded_mode {
            // Check if the message is a thread message (Thread X: ...)
            if let Some(thread_id) = line.split(':').next() {
                if thread_id.starts_with("Thread ") {
                    // Get or create buffer for this thread
                    let buffer = self.thread_buffers
                        .entry(thread_id.to_string())
                        .or_default();
                    
                    // Add message to thread buffer
                    buffer.push_back(line);
                    
                    // Ensure buffer doesn't exceed max_lines
                    while buffer.len() > self.max_lines {
                        buffer.pop_front();
                    }
                    
                    return;
                }
            }
            
            // If not a thread message, revert to single-thread mode
            self.is_threaded_mode = false;
        }
        
        // Add to the single window buffer
        self.lines.push_back(line);
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
    }
    
    /// Get the current lines in the window.
    ///
    /// # Returns
    /// A vector of strings representing the current lines
    pub fn get_lines(&self) -> Vec<String> {
        if !self.is_threaded_mode {
            return self.lines.iter().cloned().collect();
        }
        
        // In threaded mode, combine messages from all threads
        let mut all_lines = Vec::new();
        
        // Sort thread IDs to ensure consistent ordering
        let mut thread_ids: Vec<_> = self.thread_buffers.keys().cloned().collect();
        thread_ids.sort();
        
        // Add messages from each thread
        for thread_id in thread_ids {
            if let Some(buffer) = self.thread_buffers.get(&thread_id) {
                all_lines.extend(buffer.iter().cloned());
            }
        }
        
        all_lines
    }
    
    /// Get the maximum number of lines this window can display.
    ///
    /// # Returns
    /// The maximum number of lines
    pub fn max_lines(&self) -> usize {
        self.max_lines
    }
    
    /// Clear all lines from the window.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.thread_buffers.clear();
        self.is_threaded_mode = false;
    }
    
    /// Check if the window is empty.
    ///
    /// # Returns
    /// true if the window is empty, false otherwise
    pub fn is_empty(&self) -> bool {
        if self.is_threaded_mode {
            self.thread_buffers.values().all(|b| b.is_empty())
        } else {
            self.lines.is_empty()
        }
    }
    
    /// Get the current number of lines in the window.
    ///
    /// # Returns
    /// The number of lines currently in the window
    pub fn line_count(&self) -> usize {
        if self.is_threaded_mode {
            self.thread_buffers.values().map(|b| b.len()).sum()
        } else {
            self.lines.len()
        }
    }
    
    /// Get a reference to the base configuration.
    ///
    /// # Returns
    /// A reference to the BaseConfig
    pub fn base_config(&self) -> &BaseConfig {
        &self.base
    }
    
    /// Get a mutable reference to the base configuration.
    ///
    /// # Returns
    /// A mutable reference to the BaseConfig
    pub fn base_config_mut(&mut self) -> &mut BaseConfig {
        &mut self.base
    }

    /// Enable or disable line wrapping.
    ///
    /// # Parameters
    /// * `enabled` - Whether to enable or disable line wrapping
    pub fn set_line_wrapping(&mut self, enabled: bool) {
        self.line_wrapping = enabled;
    }
    
    /// Check if line wrapping is enabled.
    ///
    /// # Returns
    /// true if line wrapping is enabled, false otherwise
    pub fn has_line_wrapping(&self) -> bool {
        self.line_wrapping
    }
}

/// Base implementation for single-line display modes.
///
/// SingleLineBase provides a foundation for modes that display a single
/// line of output, with optional passthrough to stdout/stderr.
#[derive(Debug)]
pub struct SingleLineBase {
    base: BaseConfig,
    current_line: String,
    passthrough: bool,
    passthrough_writer: Option<Box<dyn ProgressWriter + Send + 'static>>,
}

impl Clone for SingleLineBase {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            current_line: self.current_line.clone(),
            passthrough: self.passthrough,
            passthrough_writer: None, // We can't clone the writer, so we set it to None
        }
    }
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
            passthrough_writer: None,
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

impl WithPassthrough for SingleLineBase {
    fn set_passthrough(&mut self, enabled: bool) {
        self.passthrough = enabled;
    }
    
    fn has_passthrough(&self) -> bool {
        self.passthrough
    }
    
    fn get_passthrough_writer_mut(&mut self) -> Option<&mut dyn ProgressWriter> {
        self.passthrough_writer.as_mut().map(|w| w.as_mut() as &mut dyn ProgressWriter)
    }
    
    fn set_passthrough_writer(&mut self, writer: Box<dyn ProgressWriter + Send + 'static>) -> Result<(), ModeCreationError> {
        self.passthrough_writer = Some(writer);
        Ok(())
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
        let factory = factory::ModeFactory::new();
        let config = factory.create_mode(mode, total_jobs)?;
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
            with_title.set_title(title)
        } else {
            Err(ModeCreationError::Implementation(
                "Config does not support titles".to_string()
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
                "Config does not support custom size".to_string()
            ))
        }
    }
    
    /// Get the maximum number of lines for this config if it supports custom size
    pub fn get_max_lines(&self) -> Option<usize> {
        self.config.as_custom_size().map(|s| s.get_max_lines())
    }

    /// Check if this config supports adding emojis
    pub fn supports_emoji(&self) -> bool {
        self.config.supports_emoji()
    }
    
    /// Add an emoji to the display if the config supports emojis
    pub fn add_emoji(&mut self, emoji: &str) -> Result<(), ModeCreationError> {
        if let Some(with_emoji) = self.config.as_emoji_mut() {
            with_emoji.add_emoji(emoji)
        } else {
            Err(ModeCreationError::Implementation(
                "Config does not support emojis".to_string()
            ))
        }
    }
    
    /// Get the emojis for this config if it supports emojis
    pub fn get_emojis(&self) -> Option<Vec<String>> {
        self.config.as_emoji().map(|e| e.get_emojis())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(ThreadMode::Limited, 1).unwrap()
    }
}

impl From<Box<dyn ThreadConfig>> for Config {
    fn from(config: Box<dyn ThreadConfig>) -> Self {
        Self { config }
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

/// Standardized parameters for mode creation
///
/// This struct provides a consistent way to pass parameters to mode creators
/// and includes validation methods to ensure parameters are valid.
#[derive(Debug, Clone)]
pub struct ModeParameters {
    /// The total number of jobs to track
    pub total_jobs: usize,
    /// The maximum number of lines to display (if applicable)
    pub max_lines: Option<usize>,
    /// The title to display (if applicable)
    pub title: Option<String>,
    /// Whether to enable emoji support (if applicable)
    pub emoji_support: Option<bool>,
    /// Whether to enable title support (if applicable)
    pub title_support: Option<bool>,
    /// Whether to enable passthrough (if applicable)
    pub passthrough: Option<bool>,
}

impl ModeParameters {
    /// Create a new ModeParameters with required parameters
    pub fn new(total_jobs: usize) -> Self {
        Self {
            total_jobs,
            max_lines: None,
            title: None,
            emoji_support: None,
            title_support: None,
            passthrough: None,
        }
    }

    /// Create parameters for Limited mode
    pub fn limited(total_jobs: usize) -> Self {
        Self::new(total_jobs)
    }

    /// Create parameters for Capturing mode
    pub fn capturing(total_jobs: usize) -> Self {
        Self::new(total_jobs)
    }

    /// Create parameters for Window mode
    pub fn window(total_jobs: usize, max_lines: usize) -> Self {
        Self {
            total_jobs,
            max_lines: Some(max_lines),
            title: None,
            emoji_support: None,
            title_support: None,
            passthrough: None,
        }
    }

    /// Create parameters for WindowWithTitle mode
    pub fn window_with_title(total_jobs: usize, max_lines: usize, title: String) -> Self {
        Self {
            total_jobs,
            max_lines: Some(max_lines),
            title: Some(title),
            emoji_support: Some(true),
            title_support: Some(true),
            passthrough: None,
        }
    }

    /// Validate parameters for a specific mode
    pub fn validate(&self, mode_name: &str) -> Result<(), ModeCreationError> {
        // Validate total_jobs
        if self.total_jobs == 0 {
            return Err(ModeCreationError::ValidationError {
                mode_name: mode_name.to_string(),
                rule: "total_jobs".to_string(),
                value: "0".to_string(),
                reason: Some("Total jobs must be greater than 0".to_string()),
            });
        }

        // Validate max_lines for window modes
        if mode_name == "window" || mode_name == "window_with_title" {
            if let Some(max_lines) = self.max_lines {
                let min_size = if mode_name == "window" { 1 } else { 2 };
                if max_lines < min_size {
                    return Err(ModeCreationError::InvalidWindowSize {
                        size: max_lines,
                        min_size,
                        mode_name: mode_name.to_string(),
                        reason: Some(format!("{} mode requires at least {} lines", mode_name, min_size)),
                    });
                }
            } else {
                return Err(ModeCreationError::MissingParameter {
                    param_name: "max_lines".to_string(),
                    mode_name: mode_name.to_string(),
                    reason: Some(format!("{} mode requires max_lines parameter", mode_name)),
                });
            }
        }

        // Validate title for WindowWithTitle mode
        if mode_name == "window_with_title" && self.title.is_none() {
            return Err(ModeCreationError::MissingParameter {
                param_name: "title".to_string(),
                mode_name: mode_name.to_string(),
                reason: Some("WindowWithTitle mode requires a title".to_string()),
            });
        }

        Ok(())
    }

    /// Get the total jobs parameter
    pub fn total_jobs(&self) -> usize {
        self.total_jobs
    }

    /// Get the max lines parameter
    pub fn max_lines(&self) -> Option<usize> {
        self.max_lines
    }

    /// Get the title parameter
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Get the emoji support parameter
    pub fn emoji_support(&self) -> Option<bool> {
        self.emoji_support
    }

    /// Get the title support parameter
    pub fn title_support(&self) -> Option<bool> {
        self.title_support
    }

    /// Get the passthrough parameter
    pub fn passthrough(&self) -> Option<bool> {
        self.passthrough
    }

    /// Set the maximum number of lines
    pub fn with_max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    /// Set the title
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Set emoji support
    pub fn with_emoji_support(mut self, enabled: bool) -> Self {
        self.emoji_support = Some(enabled);
        self
    }

    /// Set title support
    pub fn with_title_support(mut self, enabled: bool) -> Self {
        self.title_support = Some(enabled);
        self
    }

    /// Set passthrough
    pub fn with_passthrough(mut self, enabled: bool) -> Self {
        self.passthrough = Some(enabled);
        self
    }

    /// Validate that all required parameters are present for a specific mode
    pub fn validate_required_params(&self, mode_name: &str) -> Result<(), ModeCreationError> {
        match mode_name {
            "limited" | "capturing" => {
                // These modes only require total_jobs, which is always present
                Ok(())
            },
            "window" => {
                if self.max_lines.is_none() {
                    return Err(ModeCreationError::MissingParameter {
                        param_name: "max_lines".to_string(),
                        mode_name: mode_name.to_string(),
                        reason: Some("Window mode requires max_lines parameter".to_string()),
                    });
                }
                Ok(())
            },
            "window_with_title" => {
                if self.max_lines.is_none() {
                    return Err(ModeCreationError::MissingParameter {
                        param_name: "max_lines".to_string(),
                        mode_name: mode_name.to_string(),
                        reason: Some("WindowWithTitle mode requires max_lines parameter".to_string()),
                    });
                }
                if self.title.is_none() {
                    return Err(ModeCreationError::MissingParameter {
                        param_name: "title".to_string(),
                        mode_name: mode_name.to_string(),
                        reason: Some("WindowWithTitle mode requires a title".to_string()),
                    });
                }
                Ok(())
            },
            _ => Err(ModeCreationError::ModeNotRegistered {
                mode_name: mode_name.to_string(),
                available_modes: vec!["limited".to_string(), "capturing".to_string(), 
                                    "window".to_string(), "window_with_title".to_string()],
            }),
        }
    }

    /// Validate parameter values for a specific mode
    pub fn validate_param_values(&self, mode_name: &str) -> Result<(), ModeCreationError> {
        // Validate total_jobs
        if self.total_jobs == 0 {
            return Err(ModeCreationError::ValidationError {
                mode_name: mode_name.to_string(),
                rule: "total_jobs".to_string(),
                value: "0".to_string(),
                reason: Some("Total jobs must be greater than 0".to_string()),
            });
        }

        // Validate max_lines if present
        if let Some(max_lines) = self.max_lines {
            let min_size = if mode_name == "window_with_title" { 2 } else { 1 };
            if max_lines < min_size {
                return Err(ModeCreationError::InvalidWindowSize {
                    size: max_lines,
                    min_size,
                    mode_name: mode_name.to_string(),
                    reason: Some(format!("{} mode requires at least {} lines", mode_name, min_size)),
                });
            }
        }

        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_with_title_capabilities() {
        let mode = WindowWithTitle::new(10, 5, "Test Title".to_string()).unwrap();
        let mut config = Config::from(Box::new(mode) as Box<dyn ThreadConfig>);

        // Test title capability
        {
            let title_config = config.as_type_mut::<WindowWithTitle>().unwrap();
            assert!(title_config.supports_title());
            assert!(title_config.set_title("New Title".to_string()).is_ok());
            assert_eq!(title_config.get_title(), "New Title");
        }

        // Test emoji capability
        {
            let title_config = config.as_type_mut::<WindowWithTitle>().unwrap();
            assert!(title_config.supports_emoji());
            assert!(title_config.add_emoji("✨").is_ok());
            assert_eq!(title_config.get_emojis(), vec!["✨".to_string()]);
        }

        // Test title and emoji capability
        {
            let title_config = config.as_type::<WindowWithTitle>().unwrap();
            assert!(title_config.supports_title_and_emoji());
            assert_eq!(title_config.get_formatted_title(), "✨ New Title");
        }

        // Test custom size capability
        {
            let title_config = config.as_type_mut::<WindowWithTitle>().unwrap();
            assert!(title_config.supports_custom_size());
            assert!(title_config.set_max_lines(8).is_ok());
            assert_eq!(title_config.get_max_lines(), 8);
        }

        // Test standard window capability
        {
            let title_config = config.as_type_mut::<WindowWithTitle>().unwrap();
            assert!(title_config.supports_standard_window());
            title_config.add_line("Test Line".to_string());
            assert_eq!(title_config.line_count(), 1);
            assert!(!title_config.is_empty());
            title_config.clear();
            assert!(title_config.is_empty());
        }

        // Test capability set
        {
            let title_config = config.as_type::<WindowWithTitle>().unwrap();
            let caps = title_config.capabilities();
            assert!(caps.contains(&Capability::Title));
            assert!(caps.contains(&Capability::Emoji));
            assert!(caps.contains(&Capability::TitleAndEmoji));
            assert!(caps.contains(&Capability::CustomSize));
            assert!(caps.contains(&Capability::StandardWindow));
        }
    }

    #[test]
    fn test_window_capabilities() {
        let mode = Window::new(10, 5).unwrap();
        let mut config = Config::from(Box::new(mode) as Box<dyn ThreadConfig>);

        // Test title capability (should not be supported)
        {
            let window_config = config.as_type::<Window>().unwrap();
            assert!(!window_config.supports_title());
        }

        // Test emoji capability (should not be supported)
        {
            let window_config = config.as_type::<Window>().unwrap();
            assert!(!window_config.supports_emoji());
        }

        // Test title and emoji capability (should not be supported)
        {
            let window_config = config.as_type::<Window>().unwrap();
            assert!(!window_config.supports_title_and_emoji());
        }

        // Test custom size capability
        {
            let window_config = config.as_type_mut::<Window>().unwrap();
            assert!(window_config.supports_custom_size());
            assert!(window_config.set_max_lines(8).is_ok());
            assert_eq!(window_config.get_max_lines(), 8);
        }

        // Test standard window capability
        {
            let window_config = config.as_type_mut::<Window>().unwrap();
            assert!(window_config.supports_standard_window());
            window_config.add_line("Test Line".to_string());
            assert_eq!(window_config.line_count(), 1);
            assert!(!window_config.is_empty());
            window_config.clear();
            assert!(window_config.is_empty());
        }

        // Test capability set
        {
            let window_config = config.as_type::<Window>().unwrap();
            let caps = window_config.capabilities();
            assert!(!caps.contains(&Capability::Title));
            assert!(!caps.contains(&Capability::Emoji));
            assert!(!caps.contains(&Capability::TitleAndEmoji));
            assert!(caps.contains(&Capability::CustomSize));
            assert!(caps.contains(&Capability::StandardWindow));
        }
    }

    #[test]
    fn test_limited_capabilities() {
        let mode = Limited::new(10);
        let config = Config::from(Box::new(mode) as Box<dyn ThreadConfig>);

        // Test that no window capabilities are supported
        {
            let limited_config = config.as_type::<Limited>().unwrap();
            assert!(!limited_config.supports_title());
            assert!(!limited_config.supports_emoji());
            assert!(!limited_config.supports_title_and_emoji());
            assert!(!limited_config.supports_custom_size());
            assert!(!limited_config.supports_standard_window());
        }

        // Test capability set
        {
            let limited_config = config.as_type::<Limited>().unwrap();
            let caps = limited_config.capabilities();
            assert!(caps.is_empty());
        }
    }

    #[test]
    fn test_capability_conversion() {
        let mode = WindowWithTitle::new(10, 5, "Test".to_string()).unwrap();
        let mut config = Config::from(Box::new(mode) as Box<dyn ThreadConfig>);

        // Test successful conversions
        assert!(config.as_type::<WindowWithTitle>().is_some());
        assert!(config.as_type_mut::<WindowWithTitle>().is_some());

        // Test failed conversions
        assert!(config.as_type::<Window>().is_none());
        assert!(config.as_type_mut::<Window>().is_none());
        assert!(config.as_type::<Limited>().is_none());
        assert!(config.as_type_mut::<Limited>().is_none());
    }

    #[test]
    fn test_capability_error_handling() {
        let mode = Limited::new(10);
        let mut config = Config::from(Box::new(mode) as Box<dyn ThreadConfig>);

        // Test error handling for unsupported capabilities
        {
            let limited_config = config.as_type_mut::<Limited>().unwrap();
            assert!(!limited_config.supports_title());
            assert!(!limited_config.supports_emoji());
            assert!(!limited_config.supports_custom_size());
        }
    }

    #[test]
    fn test_capability_set() {
        let window = Window::new(10, 3).unwrap();
        let window_with_title = WindowWithTitle::new(10, 3, "Test".to_string()).unwrap();
        let limited = Limited::new(10);
        
        // Check Window capabilities
        let window_caps = window.capabilities();
        assert!(window_caps.contains(&Capability::CustomSize));
        assert!(window_caps.contains(&Capability::StandardWindow));
        assert!(!window_caps.contains(&Capability::Title));
        assert!(!window_caps.contains(&Capability::Emoji));
        assert!(!window_caps.contains(&Capability::TitleAndEmoji));
        
        // Check WindowWithTitle capabilities
        let window_with_title_caps = window_with_title.capabilities();
        assert!(window_with_title_caps.contains(&Capability::CustomSize));
        assert!(window_with_title_caps.contains(&Capability::StandardWindow));
        assert!(window_with_title_caps.contains(&Capability::Title));
        assert!(window_with_title_caps.contains(&Capability::Emoji));
        assert!(window_with_title_caps.contains(&Capability::TitleAndEmoji));
        
        // Check Limited capabilities
        let limited_caps = limited.capabilities();
        assert!(limited_caps.is_empty());
    }

    #[test]
    fn test_mode_parameters_validation() {
        // Test Limited mode parameters
        let params = ModeParameters::limited(10);
        assert!(params.validate("limited").is_ok());
        assert!(params.validate_required_params("limited").is_ok());
        assert!(params.validate_param_values("limited").is_ok());

        // Test Window mode parameters
        let params = ModeParameters::window(10, 5);
        assert!(params.validate("window").is_ok());
        assert!(params.validate_required_params("window").is_ok());
        assert!(params.validate_param_values("window").is_ok());

        // Test WindowWithTitle mode parameters
        let params = ModeParameters::window_with_title(10, 5, "Test Title".to_string());
        assert!(params.validate("window_with_title").is_ok());
        assert!(params.validate_required_params("window_with_title").is_ok());
        assert!(params.validate_param_values("window_with_title").is_ok());
    }

    #[test]
    fn test_mode_parameters_validation_errors() {
        // Test invalid total_jobs
        let params = ModeParameters::new(0);
        assert!(params.validate("limited").is_err());
        assert!(params.validate_param_values("limited").is_err());

        // Test missing max_lines for window mode
        let params = ModeParameters::new(10);
        assert!(params.validate("window").is_err());
        assert!(params.validate_required_params("window").is_err());

        // Test invalid max_lines for window mode
        let params = ModeParameters::window(10, 0);
        assert!(params.validate("window").is_err());
        assert!(params.validate_param_values("window").is_err());

        // Test missing title for window_with_title mode
        let params = ModeParameters::new(10);
        assert!(params.validate("window_with_title").is_err());
        assert!(params.validate_required_params("window_with_title").is_err());

        // Test invalid max_lines for window_with_title mode
        let params = ModeParameters::window_with_title(10, 1, "Test Title".to_string());
        assert!(params.validate("window_with_title").is_err());
        assert!(params.validate_param_values("window_with_title").is_err());
    }

    #[test]
    fn test_mode_parameters_builder_pattern() {
        let params = ModeParameters::new(10)
            .with_max_lines(5)
            .with_title("Test Title".to_string())
            .with_emoji_support(true)
            .with_title_support(true)
            .with_passthrough(false);

        assert_eq!(params.total_jobs(), 10);
        assert_eq!(params.max_lines(), Some(5));
        assert_eq!(params.title(), Some("Test Title"));
        assert_eq!(params.emoji_support(), Some(true));
        assert_eq!(params.title_support(), Some(true));
        assert_eq!(params.passthrough(), Some(false));
    }

    #[test]
    fn test_mode_parameters_unknown_mode() {
        let params = ModeParameters::new(10);
        assert!(params.validate_required_params("unknown_mode").is_err());
    }
}