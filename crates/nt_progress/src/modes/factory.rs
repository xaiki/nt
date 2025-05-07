use std::collections::HashMap;
use super::{ThreadConfig, ThreadMode, Limited, Capturing, Window, WindowWithTitle, ModeParameters};
use crate::errors::ModeCreationError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::fmt::Debug;

// Flag to control error propagation behavior - false means recover with fallbacks (default),
// true means propagate errors for tests that check error conditions
static PROPAGATE_ERRORS: AtomicBool = AtomicBool::new(false);

/// Set whether invalid configurations should propagate errors instead of using fallbacks
/// 
/// This function is primarily for testing - to allow some tests to verify error
/// handling while others verify recovery behavior
pub fn set_error_propagation(propagate: bool) {
    PROPAGATE_ERRORS.store(propagate, Ordering::SeqCst);
}

/// Check if errors should be propagated rather than recovered from
pub fn should_propagate_errors() -> bool {
    PROPAGATE_ERRORS.load(Ordering::SeqCst)
}

/// Trait for creating mode instances
pub trait ModeCreator: Send + Sync + Debug {
    /// Get the name of the mode this creator creates
    fn mode_name(&self) -> &'static str;
    
    /// Create a new mode instance
    fn create(&self, params: &ModeParameters) -> Result<Box<dyn ThreadConfig>, ModeCreationError>;
    
    /// Create a ThreadConfig with fallback options if the primary creation fails
    fn create_with_fallback(&self, params: &ModeParameters) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        // By default, just try the regular create method
        self.create(params)
    }
}

/// Registry for mode creators
#[derive(Debug)]
pub struct ModeRegistry {
    creators: HashMap<String, Box<dyn ModeCreator>>,
}

impl ModeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            creators: HashMap::new(),
        }
    }
    
    /// Register a new mode creator
    pub fn register<T: ModeCreator + 'static>(&mut self, creator: T) {
        let name = creator.mode_name().to_string();
        self.creators.insert(name, Box::new(creator));
    }
    
    /// Validate parameters before mode creation
    fn validate_params(&self, mode_name: &str, params: &ModeParameters) -> Result<(), ModeCreationError> {
        if let Some(creator) = self.creators.get(mode_name) {
            params.validate(mode_name)?;
            Ok(())
        } else {
            let available_modes: Vec<String> = self.creators.keys().cloned().collect();
            Err(ModeCreationError::ModeNotRegistered {
                mode_name: mode_name.to_string(),
                available_modes,
            })
        }
    }
    
    /// Create a ThreadConfig instance using the specified mode and parameters
    pub fn create(&self, mode_name: &str, params: &ModeParameters) 
        -> Result<Box<dyn ThreadConfig>, ModeCreationError> 
    {
        self.validate_params(mode_name, params)?;

        if let Some(creator) = self.creators.get(mode_name) {
            let result = creator.create(params);
            match result {
                Ok(config) => Ok(config),
                Err(err) => Err(err), // Since the error type is already ModeCreationError
            }
        } else {
            unreachable!()
        }
    }
    
    /// Create a ThreadConfig instance from a ThreadMode enum
    pub fn create_from_mode(&self, mode: ThreadMode, total_jobs: usize) 
        -> Result<Box<dyn ThreadConfig>, ModeCreationError> 
    {
        match mode {
            ThreadMode::Limited => self.create("limited", &ModeParameters::limited(total_jobs)),
            ThreadMode::Capturing => self.create("capturing", &ModeParameters::capturing(total_jobs)),
            ThreadMode::Window(max_lines) => self.create("window", &ModeParameters::window(total_jobs, max_lines)),
            ThreadMode::WindowWithTitle(max_lines) => self.create("window_with_title", &ModeParameters::window_with_title(total_jobs, max_lines, "Progress".to_string())),
        }
    }
}

// Concrete creator implementations

/// Creator for Limited mode
#[derive(Debug)]
pub struct LimitedCreator;

impl ModeCreator for LimitedCreator {
    fn create(&self, params: &ModeParameters) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        params.validate(self.mode_name())?;
        Ok(Box::new(Limited::new(params.total_jobs())))
    }
    
    fn mode_name(&self) -> &'static str {
        "limited"
    }
}

/// Creator for Capturing mode
#[derive(Debug)]
pub struct CapturingCreator;

impl ModeCreator for CapturingCreator {
    fn create(&self, params: &ModeParameters) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        params.validate(self.mode_name())?;
        Ok(Box::new(Capturing::new(params.total_jobs())))
    }
    
    fn mode_name(&self) -> &'static str {
        "capturing"
    }
}

/// Creator for Window mode
#[derive(Debug)]
pub struct WindowCreator;

impl ModeCreator for WindowCreator {
    fn create(&self, params: &ModeParameters) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        params.validate(self.mode_name())?;
        let max_lines = params.max_lines().unwrap();
        Window::new(params.total_jobs(), max_lines).map(|w| Box::new(w) as Box<dyn ThreadConfig>)
    }
    
    fn create_with_fallback(&self, params: &ModeParameters) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        let result = self.create(params);
        
        // If creation fails, try with a reasonable fallback size
        if result.is_err() {
            let max_lines = params.max_lines().unwrap_or(0);
            eprintln!("Warning: Requested window size {} was invalid, using size 3 instead", max_lines);
            
            // Try with a reasonable fallback size
            let mut fallback_params = params.clone();
            fallback_params.max_lines = Some(3);
            if let Ok(window) = Window::new(params.total_jobs(), 3) {
                return Ok(Box::new(window));
            }
            
            // Last resort: fall back to Limited mode
            eprintln!("Warning: Could not create Window mode, falling back to Limited mode");
            return Ok(Box::new(Limited::new(params.total_jobs())));
        }
        
        result
    }
    
    fn mode_name(&self) -> &'static str {
        "window"
    }
}

/// Creator for WindowWithTitle mode
#[derive(Debug)]
pub struct WindowWithTitleCreator;

impl ModeCreator for WindowWithTitleCreator {
    fn create(&self, params: &ModeParameters) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        params.validate(self.mode_name())?;
        let max_lines = params.max_lines().unwrap();
        let title = params.title().unwrap_or("Progress").to_string();
        let mut mode = WindowWithTitle::new(params.total_jobs(), max_lines, title)?;
        
        // Set support flags if provided
        if let Some(emoji_support) = params.emoji_support() {
            mode.set_emoji_support(emoji_support);
        }
        if let Some(title_support) = params.title_support() {
            mode.set_title_support(title_support);
        }
        
        Ok(Box::new(mode) as Box<dyn ThreadConfig>)
    }
    
    fn create_with_fallback(&self, params: &ModeParameters) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        let result = self.create(params);
        
        // If creation fails, try with a reasonable fallback size
        if result.is_err() {
            let max_lines = params.max_lines().unwrap_or(0);
            eprintln!("Warning: Requested window size {} was invalid, using size 3 instead", max_lines);
            
            // Try with a reasonable fallback size
            let mut fallback_params = params.clone();
            fallback_params.max_lines = Some(3);
            if let Ok(mut window) = WindowWithTitle::new(params.total_jobs(), 3, params.title().unwrap_or("Progress").to_string()) {
                // Set support flags if provided
                if let Some(emoji_support) = params.emoji_support() {
                    window.set_emoji_support(emoji_support);
                }
                if let Some(title_support) = params.title_support() {
                    window.set_title_support(title_support);
                }
                return Ok(Box::new(window));
            }
            
            // Last resort: fall back to Limited mode
            eprintln!("Warning: Could not create WindowWithTitle mode, falling back to Limited mode");
            return Ok(Box::new(Limited::new(params.total_jobs())));
        }
        
        result
    }
    
    fn mode_name(&self) -> &'static str {
        "window_with_title"
    }
}

/// A factory for creating mode instances
///
/// ModeFactory provides a way to create mode instances without using static
/// references. It maintains a registry of mode creators and can create new
/// instances on demand.
#[derive(Debug, Clone)]
pub struct ModeFactory {
    registry: Arc<ModeRegistry>,
    default_mode: ThreadMode,
}

impl ModeFactory {
    /// Create a new ModeFactory with a custom registry
    pub fn with_registry(registry: Arc<ModeRegistry>) -> Self {
        Self {
            registry,
            default_mode: ThreadMode::Limited,
        }
    }

    /// Create a new ModeFactory with the default set of modes
    pub fn new() -> Self {
        let mut registry = ModeRegistry::new();
        
        // Register standard modes
        registry.register(LimitedCreator);
        registry.register(CapturingCreator);
        registry.register(WindowCreator);
        registry.register(WindowWithTitleCreator);
        
        Self::with_registry(Arc::new(registry))
    }

    /// Create a new ModeFactory with a specific set of modes
    pub fn with_modes<F>(f: F) -> Self 
    where
        F: FnOnce(&mut ModeRegistry)
    {
        let mut registry = ModeRegistry::new();
        f(&mut registry);
        Self::with_registry(Arc::new(registry))
    }
    
    /// Set the default mode for this factory
    pub fn set_default_mode(&mut self, mode: ThreadMode) {
        self.default_mode = mode;
    }
    
    /// Get the default mode for this factory
    pub fn default_mode(&self) -> ThreadMode {
        self.default_mode
    }
    
    /// Create a new mode instance
    ///
    /// # Parameters
    /// * `mode` - The mode to create
    /// * `total_jobs` - The total number of jobs to track
    ///
    /// # Returns
    /// A Result containing either the created ThreadConfig or an error
    pub fn create_mode(&self, mode: ThreadMode, total_jobs: usize) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        if should_propagate_errors() {
            // If error propagation is enabled, use direct creation without fallbacks
            match mode {
                ThreadMode::Limited => self.registry.create("limited", &ModeParameters::limited(total_jobs)),
                ThreadMode::Capturing => self.registry.create("capturing", &ModeParameters::capturing(total_jobs)),
                ThreadMode::Window(size) => self.registry.create("window", &ModeParameters::window(total_jobs, size)),
                ThreadMode::WindowWithTitle(size) => self.registry.create("window_with_title", &ModeParameters::window_with_title(total_jobs, size, "Progress".to_string())),
            }
        } else {
            // With error propagation disabled, provide fallbacks
            match mode {
                ThreadMode::Limited => self.registry.create("limited", &ModeParameters::limited(total_jobs)),
                ThreadMode::Capturing => self.registry.create("capturing", &ModeParameters::capturing(total_jobs)),
                ThreadMode::Window(size) => {
                    let result = self.registry.create("window", &ModeParameters::window(total_jobs, size));
                    if result.is_err() {
                        // Try with a reasonable fallback size
                        eprintln!("Warning: Requested window size {} was invalid, using size 3 instead", size);
                        self.registry.create("window", &ModeParameters::window(total_jobs, 3))
                    } else {
                        result
                    }
                },
                ThreadMode::WindowWithTitle(size) => {
                    let result = self.registry.create("window_with_title", &ModeParameters::window_with_title(total_jobs, size, "Progress".to_string()));
                    if result.is_err() {
                        // Try with a reasonable fallback size
                        eprintln!("Warning: Requested window size {} was invalid, using size 3 instead", size);
                        let fallback = self.registry.create("window_with_title", &ModeParameters::window_with_title(total_jobs, 3, "Progress".to_string()));
                        if fallback.is_err() {
                            // Try with Window mode as a fallback
                            eprintln!("Warning: Could not create WindowWithTitle mode, falling back to Window mode");
                            let window = self.registry.create("window", &ModeParameters::window(total_jobs, 3));
                            if window.is_err() {
                                // Last resort: fall back to Limited mode
                                eprintln!("Warning: Could not create any window mode, falling back to Limited mode");
                                self.registry.create("limited", &ModeParameters::limited(total_jobs))
                            } else {
                                window
                            }
                        } else {
                            fallback
                        }
                    } else {
                        result
                    }
                }
            }
        }
    }
    
    /// Get a reference to the underlying registry
    pub fn registry(&self) -> &ModeRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_creation() {
        let registry = ModeRegistry::new();
        assert_eq!(registry.creators.len(), 0);
    }
    
    #[test]
    fn test_creator_registration() {
        let mut registry = ModeRegistry::new();
        registry.register(LimitedCreator);
        registry.register(WindowCreator);
        
        assert_eq!(registry.creators.len(), 2);
        assert!(registry.creators.contains_key("limited"));
        assert!(registry.creators.contains_key("window"));
    }
    
    #[test]
    fn test_mode_creation() {
        let mut registry = ModeRegistry::new();
        registry.register(LimitedCreator);
        registry.register(WindowCreator);
        
        // Create a Limited mode
        let config = registry.create("limited", &ModeParameters::limited(10)).unwrap();
        assert_eq!(config.lines_to_display(), 1);
        
        // Create a Window mode
        let config = registry.create("window", &ModeParameters::window(10, 3)).unwrap();
        assert_eq!(config.lines_to_display(), 3);
    }
    
    #[test]
    fn test_mode_creation_with_invalid_params() {
        let mut registry = ModeRegistry::new();
        registry.register(WindowCreator);
        
        // Try to create a Window mode without required params
        let result = registry.create("window", &ModeParameters::limited(10));
        assert!(result.is_err());
        
        // Verify the error type
        match result {
            Err(ModeCreationError::MissingParameter { param_name, mode_name, reason }) => {
                assert_eq!(mode_name, "window");
                assert_eq!(param_name, "max_lines");
                assert!(reason.is_some());
                assert!(reason.unwrap().contains("requires"));
            },
            _ => panic!("Unexpected error type"),
        }
    }
    
    #[test]
    fn test_factory_modes() {
        let factory = ModeFactory::new();
        let registry = factory.registry();
        
        // Verify registry contains all standard modes
        assert!(registry.creators.contains_key("limited"));
        assert!(registry.creators.contains_key("capturing"));
        assert!(registry.creators.contains_key("window"));
        assert!(registry.creators.contains_key("window_with_title"));
    }
    
    #[test]
    fn test_error_propagation() {
        // Test with error propagation enabled
        set_error_propagation(true);
        
        let factory = ModeFactory::new();
        
        // Create a window with invalid size
        let result = factory.create_mode(ThreadMode::Window(0), 1);
        assert!(result.is_err());
        
        // Test with error propagation disabled
        set_error_propagation(false);
        
        // Should recover with fallback
        let result = factory.create_mode(ThreadMode::Window(0), 1);
        assert!(result.is_ok());
        
        // Reset for other tests
        set_error_propagation(false);
    }
    
    #[test]
    fn test_factory_configuration() {
        let mut factory = ModeFactory::new();
        
        // Test default mode
        assert!(matches!(factory.default_mode(), ThreadMode::Limited));
        
        // Change default mode
        factory.set_default_mode(ThreadMode::Window(3));
        assert!(matches!(factory.default_mode(), ThreadMode::Window(3)));
        
        // Test mode creation with new default
        let config = factory.create_mode(ThreadMode::Window(3), 10).unwrap();
        assert_eq!(config.lines_to_display(), 3);
    }
    
    #[test]
    fn test_factory_error_handling() {
        // First test with error propagation enabled - should error on invalid sizes
        set_error_propagation(true);
        let factory = ModeFactory::new();
        
        // Test invalid window size
        let result = factory.create_mode(ThreadMode::Window(0), 10);
        assert!(result.is_err(), "With error propagation enabled, invalid window size should fail");
        
        // Test invalid window with title size
        let result = factory.create_mode(ThreadMode::WindowWithTitle(0), 10);
        assert!(result.is_err(), "With error propagation enabled, invalid window with title size should fail");
        
        // Now test with error propagation disabled - should use fallbacks
        set_error_propagation(false);
        let factory = ModeFactory::new();
        
        // Test with invalid sizes - should succeed with fallbacks
        let result = factory.create_mode(ThreadMode::Window(0), 10);
        assert!(result.is_ok(), "With error propagation disabled, invalid window size should use fallback");
        
        let result = factory.create_mode(ThreadMode::WindowWithTitle(0), 10);
        assert!(result.is_ok(), "With error propagation disabled, invalid window with title size should use fallback");
        
        // Test with valid sizes - always succeeds
        let result = factory.create_mode(ThreadMode::Window(3), 10);
        assert!(result.is_ok());
        
        let result = factory.create_mode(ThreadMode::WindowWithTitle(3), 10);
        assert!(result.is_ok());
        
        // Reset for other tests
        set_error_propagation(false);
    }
    
    #[test]
    fn test_factory_clone() {
        let mut factory1 = ModeFactory::new();
        factory1.set_default_mode(ThreadMode::Window(3));
        
        // Clone the factory
        let factory2 = factory1.clone();
        
        // Verify both have same configuration
        assert!(matches!(factory1.default_mode(), ThreadMode::Window(3)));
        assert!(matches!(factory2.default_mode(), ThreadMode::Window(3)));
        
        // Verify changes to one don't affect the other
        factory1.set_default_mode(ThreadMode::Limited);
        assert!(matches!(factory1.default_mode(), ThreadMode::Limited));
        assert!(matches!(factory2.default_mode(), ThreadMode::Window(3)));
    }
    
    #[test]
    fn test_window_with_title_support_flags() {
        let factory = ModeFactory::new();
        
        // Create a WindowWithTitle mode
        let config = factory.create_mode(ThreadMode::WindowWithTitle(3), 10).unwrap();
        
        // Downcast to WindowWithTitle to check support flags
        let window_with_title = config.as_any().downcast_ref::<WindowWithTitle>().unwrap();
        
        // Verify emoji and title support are enabled by default
        assert!(window_with_title.has_emoji_support(), "Emoji support should be enabled by default");
        assert!(window_with_title.has_title_support(), "Title support should be enabled by default");
    }

    #[test]
    fn test_factory_with_custom_registry() {
        let mut registry = ModeRegistry::new();
        registry.register(LimitedCreator);
        registry.register(WindowCreator);
        
        let factory = ModeFactory::with_registry(Arc::new(registry));
        
        // Verify registry contains only the registered modes
        let registry = factory.registry();
        assert!(registry.creators.contains_key("limited"));
        assert!(registry.creators.contains_key("window"));
        assert!(!registry.creators.contains_key("capturing"));
        assert!(!registry.creators.contains_key("window_with_title"));
    }

    #[test]
    fn test_factory_with_modes() {
        let factory = ModeFactory::with_modes(|registry| {
            registry.register(LimitedCreator);
            registry.register(WindowCreator);
        });
        
        // Verify registry contains only the registered modes
        let registry = factory.registry();
        assert!(registry.creators.contains_key("limited"));
        assert!(registry.creators.contains_key("window"));
        assert!(!registry.creators.contains_key("capturing"));
        assert!(!registry.creators.contains_key("window_with_title"));
    }

    #[test]
    fn test_validation_errors() {
        let mut registry = ModeRegistry::new();
        registry.register(LimitedCreator);
        registry.register(WindowCreator);
        registry.register(WindowWithTitleCreator);
        
        // Test zero total_jobs
        let result = registry.create("limited", &ModeParameters::limited(0));
        assert!(result.is_err());
        match result {
            Err(ModeCreationError::ValidationError { mode_name, rule, value, reason }) => {
                assert_eq!(mode_name, "limited");
                assert_eq!(rule, "total_jobs");
                assert_eq!(value, "0");
                assert!(reason.is_some());
                assert!(reason.unwrap().contains("must be greater than 0"));
            },
            _ => panic!("Expected ValidationError"),
        }
        
        // Test invalid window size
        let result = registry.create("window", &ModeParameters::window(10, 0));
        assert!(result.is_err());
        match result {
            Err(ModeCreationError::InvalidWindowSize { size, min_size, mode_name, reason }) => {
                assert_eq!(size, 0);
                assert_eq!(min_size, 1);
                assert_eq!(mode_name, "window");
                assert!(reason.is_some());
                assert!(reason.unwrap().contains("requires at least 1 lines"));
            },
            _ => panic!("Expected InvalidWindowSize error"),
        }
        
        // Test invalid window with title size
        let result = registry.create("window_with_title", &ModeParameters::window_with_title(10, 1, "Test".to_string()));
        assert!(result.is_err());
        match result {
            Err(ModeCreationError::InvalidWindowSize { size, min_size, mode_name, reason }) => {
                assert_eq!(size, 1);
                assert_eq!(min_size, 2);
                assert_eq!(mode_name, "window_with_title");
                assert!(reason.is_some());
                assert!(reason.unwrap().contains("requires at least 2 lines"));
            },
            _ => panic!("Expected InvalidWindowSize error"),
        }
    }
    
    #[test]
    fn test_validation_success() {
        let mut registry = ModeRegistry::new();
        registry.register(LimitedCreator);
        registry.register(WindowCreator);
        registry.register(WindowWithTitleCreator);
        
        // Test valid limited mode
        let result = registry.create("limited", &ModeParameters::limited(1));
        assert!(result.is_ok());
        
        // Test valid window mode
        let result = registry.create("window", &ModeParameters::window(10, 3));
        assert!(result.is_ok());
        
        // Test valid window with title mode
        let result = registry.create("window_with_title", &ModeParameters::window_with_title(10, 3, "Test".to_string()));
        assert!(result.is_ok());
    }
} 