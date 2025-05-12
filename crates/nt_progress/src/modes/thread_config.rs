use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::fmt::Debug;

use super::window::Window;
use super::window_with_title::WindowWithTitle;
use super::limited::Limited;
use super::capturing::Capturing;
use super::capabilities::{
    Capability, WithTitle, WithCustomSize, WithEmoji, WithTitleAndEmoji, 
    StandardWindow, WithWrappedText, WithProgress
};
use super::job_traits::{PrioritizedJob, PausableJob, DependentJob};

// Re-export the ThreadConfig trait from core
pub use crate::core::thread_config::ThreadConfig;

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

/// Extension trait for ThreadConfig that provides capability checking and conversion.
///
/// This trait extends ThreadConfig to provide methods for checking if a
/// mode supports specific capabilities and converting it to those capability
/// interfaces.
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
            Capability::Progress => self.supports_progress(),
            Capability::PrioritizedJob => self.supports_prioritized_job(),
            Capability::PausableJob => self.supports_pausable_job(),
            Capability::DependentJob => self.supports_dependent_job(),
        }
    }

    /// Add support for checking if a config supports progress tracking.
    ///
    /// # Returns
    /// `true` if the config supports progress tracking and display.
    fn supports_progress(&self) -> bool {
        let type_id = self.as_any().type_id();
        matches!(type_id, t if t == TypeId::of::<Window>() || t == TypeId::of::<WindowWithTitle>())
    }
    
    /// Try to get this config as a WithProgress.
    ///
    /// # Returns
    /// Some(&dyn WithProgress) if the config supports progress tracking, None otherwise.
    fn as_progress(&self) -> Option<&dyn WithProgress> {
        let any = self.as_any();
        if let Some(w) = any.downcast_ref::<Window>() {
            Some(w as &dyn WithProgress)
        } else {
            any.downcast_ref::<WindowWithTitle>().map(|w| w as &dyn WithProgress)
        }
    }
    
    /// Try to get this config as a mutable WithProgress.
    ///
    /// # Returns
    /// Some(&mut dyn WithProgress) if the config supports progress tracking, None otherwise.
    fn as_progress_mut(&mut self) -> Option<&mut dyn WithProgress> {
        let type_id = self.as_any().type_id();
        let any = self.as_any_mut();
        if type_id == TypeId::of::<Window>() {
            any.downcast_mut::<Window>().map(|w| w as &mut dyn WithProgress)
        } else if type_id == TypeId::of::<WindowWithTitle>() {
            any.downcast_mut::<WindowWithTitle>().map(|w| w as &mut dyn WithProgress)
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
        add_if_supported!(Capability::Progress, self.supports_progress());
        add_if_supported!(Capability::PrioritizedJob, self.supports_prioritized_job());
        add_if_supported!(Capability::PausableJob, self.supports_pausable_job());
        add_if_supported!(Capability::DependentJob, self.supports_dependent_job());
        
        caps
    }

    /// Add support for checking if a config supports job prioritization.
    ///
    /// # Returns
    /// `true` if the config supports job prioritization.
    fn supports_prioritized_job(&self) -> bool {
        // All of our modes support PrioritizedJob via blanket implementation
        true
    }

    /// Try to get this config as a PrioritizedJob.
    ///
    /// # Returns
    /// Some(&dyn PrioritizedJob) if the config supports job prioritization, None otherwise.
    fn as_prioritized_job(&self) -> Option<&dyn PrioritizedJob> {
        let any = self.as_any();
        if let Some(w) = any.downcast_ref::<Window>() {
            Some(w as &dyn PrioritizedJob)
        } else if let Some(w) = any.downcast_ref::<WindowWithTitle>() {
            Some(w as &dyn PrioritizedJob)
        } else if let Some(l) = any.downcast_ref::<Limited>() {
            Some(l as &dyn PrioritizedJob)
        } else {
            any.downcast_ref::<Capturing>().map(|c| c as &dyn PrioritizedJob)
        }
    }

    /// Try to get this config as a mutable PrioritizedJob.
    ///
    /// # Returns
    /// Some(&mut dyn PrioritizedJob) if the config supports job prioritization, None otherwise.
    fn as_prioritized_job_mut(&mut self) -> Option<&mut dyn PrioritizedJob> {
        let type_id = self.as_any().type_id();
        let any = self.as_any_mut();
        if type_id == TypeId::of::<Window>() {
            any.downcast_mut::<Window>().map(|w| w as &mut dyn PrioritizedJob)
        } else if type_id == TypeId::of::<WindowWithTitle>() {
            any.downcast_mut::<WindowWithTitle>().map(|w| w as &mut dyn PrioritizedJob)
        } else if type_id == TypeId::of::<Limited>() {
            any.downcast_mut::<Limited>().map(|l| l as &mut dyn PrioritizedJob)
        } else {
            any.downcast_mut::<Capturing>().map(|c| c as &mut dyn PrioritizedJob)
        }
    }

    /// Add support for checking if a config supports pausing and resuming jobs.
    ///
    /// # Returns
    /// `true` if the config supports pausing and resuming jobs.
    fn supports_pausable_job(&self) -> bool {
        // All of our modes support PausableJob via blanket implementation
        true
    }

    /// Try to get this config as a PausableJob.
    ///
    /// # Returns
    /// Some(&dyn PausableJob) if the config supports pausing and resuming jobs, None otherwise.
    fn as_pausable_job(&self) -> Option<&dyn PausableJob> {
        let any = self.as_any();
        if let Some(w) = any.downcast_ref::<Window>() {
            Some(w as &dyn PausableJob)
        } else if let Some(w) = any.downcast_ref::<WindowWithTitle>() {
            Some(w as &dyn PausableJob)
        } else if let Some(l) = any.downcast_ref::<Limited>() {
            Some(l as &dyn PausableJob)
        } else {
            any.downcast_ref::<Capturing>().map(|c| c as &dyn PausableJob)
        }
    }

    /// Try to get this config as a mutable PausableJob.
    ///
    /// # Returns
    /// Some(&mut dyn PausableJob) if the config supports pausing and resuming jobs, None otherwise.
    fn as_pausable_job_mut(&mut self) -> Option<&mut dyn PausableJob> {
        let type_id = self.as_any().type_id();
        let any = self.as_any_mut();
        if type_id == TypeId::of::<Window>() {
            any.downcast_mut::<Window>().map(|w| w as &mut dyn PausableJob)
        } else if type_id == TypeId::of::<WindowWithTitle>() {
            any.downcast_mut::<WindowWithTitle>().map(|w| w as &mut dyn PausableJob)
        } else if type_id == TypeId::of::<Limited>() {
            any.downcast_mut::<Limited>().map(|l| l as &mut dyn PausableJob)
        } else {
            any.downcast_mut::<Capturing>().map(|c| c as &mut dyn PausableJob)
        }
    }

    /// Add support for checking if a config supports job dependencies.
    ///
    /// # Returns
    /// `true` if the config supports job dependencies.
    fn supports_dependent_job(&self) -> bool {
        // All of our modes support DependentJob via blanket implementation
        true
    }

    /// Try to get this config as a DependentJob.
    ///
    /// # Returns
    /// Some(&dyn DependentJob) if the config supports job dependencies, None otherwise.
    fn as_dependent_job(&self) -> Option<&dyn DependentJob> {
        let any = self.as_any();
        if let Some(w) = any.downcast_ref::<Window>() {
            Some(w as &dyn DependentJob)
        } else if let Some(w) = any.downcast_ref::<WindowWithTitle>() {
            Some(w as &dyn DependentJob)
        } else if let Some(l) = any.downcast_ref::<Limited>() {
            Some(l as &dyn DependentJob)
        } else {
            any.downcast_ref::<Capturing>().map(|c| c as &dyn DependentJob)
        }
    }

    /// Try to get this config as a mutable DependentJob.
    ///
    /// # Returns
    /// Some(&mut dyn DependentJob) if the config supports job dependencies, None otherwise.
    fn as_dependent_job_mut(&mut self) -> Option<&mut dyn DependentJob> {
        let type_id = self.as_any().type_id();
        let any = self.as_any_mut();
        if type_id == TypeId::of::<Window>() {
            any.downcast_mut::<Window>().map(|w| w as &mut dyn DependentJob)
        } else if type_id == TypeId::of::<WindowWithTitle>() {
            any.downcast_mut::<WindowWithTitle>().map(|w| w as &mut dyn DependentJob)
        } else if type_id == TypeId::of::<Limited>() {
            any.downcast_mut::<Limited>().map(|l| l as &mut dyn DependentJob)
        } else {
            any.downcast_mut::<Capturing>().map(|c| c as &mut dyn DependentJob)
        }
    }
}

// Blanket implementation of ThreadConfigExt for all types that implement ThreadConfig
impl<T: ThreadConfig + ?Sized> ThreadConfigExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modes::window::Window;
    use crate::modes::window_with_title::WindowWithTitle;
    use crate::modes::limited::Limited;
    
    // Test helper to validate that a mode supports the expected capabilities
    fn validate_capabilities<T: ThreadConfig>(config: &T, expected: &[Capability]) {
        let caps = config.capabilities();
        for cap in expected {
            assert!(caps.contains(cap), "Expected capability {:?} to be supported", cap);
            assert!(config.supports_capability(*cap), "Expected supports_capability to return true for {:?}", cap);
        }
    }
    
    #[test]
    fn test_window_with_title_capabilities() {
        let config = WindowWithTitle::new(10, 5, "Test Title".to_string()).unwrap();
        
        // Test basic capability checks
        assert!(config.supports_title());
        assert!(config.supports_custom_size());
        assert!(config.supports_emoji());
        assert!(config.supports_title_and_emoji());
        assert!(config.supports_standard_window());
        assert!(config.supports_wrapped_text());
        assert!(config.supports_progress());
        assert!(config.supports_prioritized_job());
        assert!(config.supports_pausable_job());
        assert!(config.supports_dependent_job());
        
        // Test capability downcast methods
        assert!(config.as_title().is_some());
        assert!(config.as_custom_size().is_some());
        assert!(config.as_emoji().is_some());
        assert!(config.as_title_and_emoji().is_some());
        assert!(config.as_standard_window().is_some());
        assert!(config.as_wrapped_text().is_some());
        assert!(config.as_progress().is_some());
        assert!(config.as_prioritized_job().is_some());
        assert!(config.as_pausable_job().is_some());
        assert!(config.as_dependent_job().is_some());
        
        // Test the capabilities method
        let expected = [
            Capability::Title, Capability::CustomSize, Capability::Emoji, 
            Capability::TitleAndEmoji, Capability::StandardWindow, Capability::WrappedText,
            Capability::Progress, Capability::PrioritizedJob, Capability::PausableJob,
            Capability::DependentJob
        ];
        
        validate_capabilities(&config, &expected);
    }
    
    #[test]
    fn test_window_capabilities() {
        let config = Window::new(10, 5).unwrap();
        
        // Test basic capability checks
        assert!(!config.supports_title());
        assert!(config.supports_custom_size());
        assert!(!config.supports_emoji());
        assert!(!config.supports_title_and_emoji());
        assert!(config.supports_standard_window());
        assert!(config.supports_wrapped_text());
        assert!(config.supports_progress());
        assert!(config.supports_prioritized_job());
        assert!(config.supports_pausable_job());
        assert!(config.supports_dependent_job());
        
        // Test capability downcast methods
        assert!(config.as_title().is_none());
        assert!(config.as_custom_size().is_some());
        assert!(config.as_emoji().is_none());
        assert!(config.as_title_and_emoji().is_none());
        assert!(config.as_standard_window().is_some());
        assert!(config.as_wrapped_text().is_some());
        assert!(config.as_progress().is_some());
        assert!(config.as_prioritized_job().is_some());
        assert!(config.as_pausable_job().is_some());
        assert!(config.as_dependent_job().is_some());
        
        // Test the capabilities method
        let expected = [
            Capability::CustomSize, Capability::StandardWindow, Capability::WrappedText,
            Capability::Progress, Capability::PrioritizedJob, Capability::PausableJob,
            Capability::DependentJob
        ];
        
        validate_capabilities(&config, &expected);
    }
    
    #[test]
    fn test_limited_capabilities() {
        let config = Limited::new(10);
        
        // Test basic capability checks
        assert!(!config.supports_title());
        assert!(!config.supports_custom_size());
        assert!(!config.supports_emoji());
        assert!(!config.supports_title_and_emoji());
        assert!(!config.supports_standard_window());
        assert!(!config.supports_wrapped_text());
        assert!(!config.supports_progress());
        assert!(config.supports_prioritized_job());
        assert!(config.supports_pausable_job());
        assert!(config.supports_dependent_job());
        
        // Test capability downcast methods
        assert!(config.as_title().is_none());
        assert!(config.as_custom_size().is_none());
        assert!(config.as_emoji().is_none());
        assert!(config.as_title_and_emoji().is_none());
        assert!(config.as_standard_window().is_none());
        assert!(config.as_wrapped_text().is_none());
        assert!(config.as_progress().is_none());
        assert!(config.as_prioritized_job().is_some());
        assert!(config.as_pausable_job().is_some());
        assert!(config.as_dependent_job().is_some());
        
        // Test the capabilities method
        let expected = [
            Capability::PrioritizedJob, Capability::PausableJob, Capability::DependentJob
        ];
        
        validate_capabilities(&config, &expected);
    }
} 