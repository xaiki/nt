use std::collections::HashMap;
use super::{ThreadConfig, ThreadMode, Limited, Capturing, Window, WindowWithTitle};
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
    
    /// Get the minimum number of parameters required
    fn min_params(&self) -> usize;
    
    /// Create a new mode instance
    fn create(&self, total_jobs: usize, params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError>;
    
    /// Create a ThreadConfig with fallback options if the primary creation fails
    fn create_with_fallback(&self, total_jobs: usize, params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        // By default, just try the regular create method
        self.create(total_jobs, params)
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
    
    /// Create a ThreadConfig instance using the specified mode and parameters
    pub fn create(&self, mode_name: &str, total_jobs: usize, params: &[usize]) 
        -> Result<Box<dyn ThreadConfig>, ModeCreationError> 
    {
        if let Some(creator) = self.creators.get(mode_name) {
            if params.len() < creator.min_params() {
                return Err(ModeCreationError::MissingParameter {
                    param_name: format!("parameter {}", creator.min_params()),
                    mode_name: mode_name.to_string(),
                });
            }
            
            let result = creator.create(total_jobs, params);
            match result {
                Ok(config) => Ok(config),
                Err(err) => Err(err), // Since the error type is already ModeCreationError
            }
        } else {
            Err(ModeCreationError::Implementation(
                format!("Unknown mode: {}", mode_name)
            ))
        }
    }
    
    /// Create a ThreadConfig instance from a ThreadMode enum
    pub fn create_from_mode(&self, mode: ThreadMode, total_jobs: usize) 
        -> Result<Box<dyn ThreadConfig>, ModeCreationError> 
    {
        match mode {
            ThreadMode::Limited => self.create("limited", total_jobs, &[]),
            ThreadMode::Capturing => self.create("capturing", total_jobs, &[]),
            ThreadMode::Window(max_lines) => self.create("window", total_jobs, &[max_lines]),
            ThreadMode::WindowWithTitle(max_lines) => self.create("window_with_title", total_jobs, &[max_lines]),
        }
    }
}

// Concrete creator implementations

/// Creator for Limited mode
#[derive(Debug)]
pub struct LimitedCreator;

impl ModeCreator for LimitedCreator {
    fn create(&self, total_jobs: usize, _params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        Ok(Box::new(Limited::new(total_jobs)))
    }
    
    fn mode_name(&self) -> &'static str {
        "limited"
    }
    
    fn min_params(&self) -> usize {
        0
    }
}

/// Creator for Capturing mode
#[derive(Debug)]
pub struct CapturingCreator;

impl ModeCreator for CapturingCreator {
    fn create(&self, total_jobs: usize, _params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        Ok(Box::new(Capturing::new(total_jobs)))
    }
    
    fn mode_name(&self) -> &'static str {
        "capturing"
    }
    
    fn min_params(&self) -> usize {
        0
    }
}

/// Creator for Window mode
#[derive(Debug)]
pub struct WindowCreator;

impl ModeCreator for WindowCreator {
    fn create(&self, total_jobs: usize, params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        let max_lines = params[0];
        Window::new(total_jobs, max_lines).map(|w| Box::new(w) as Box<dyn ThreadConfig>)
    }
    
    fn create_with_fallback(&self, total_jobs: usize, params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        let result = self.create(total_jobs, params);
        
        // If creation fails, try with a reasonable fallback size
        if result.is_err() {
            let max_lines = if params.is_empty() { 0 } else { params[0] };
            eprintln!("Warning: Requested window size {} was invalid, using size 3 instead", max_lines);
            
            // Try with a reasonable fallback size
            if let Ok(window) = Window::new(total_jobs, 3) {
                return Ok(Box::new(window));
            }
            
            // Last resort: fall back to Limited mode
            eprintln!("Warning: Could not create Window mode, falling back to Limited mode");
            return Ok(Box::new(Limited::new(total_jobs)));
        }
        
        result
    }
    
    fn mode_name(&self) -> &'static str {
        "window"
    }
    
    fn min_params(&self) -> usize {
        1
    }
}

/// Creator for WindowWithTitle mode
#[derive(Debug)]
pub struct WindowWithTitleCreator;

impl ModeCreator for WindowWithTitleCreator {
    fn create(&self, total_jobs: usize, params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        let max_lines = params[0];
        let mut mode = WindowWithTitle::new(total_jobs, max_lines, "Progress".to_string())?;
        
        // Explicitly enable emoji and title support
        mode.set_emoji_support(true);
        mode.set_title_support(true);
        
        Ok(Box::new(mode) as Box<dyn ThreadConfig>)
    }
    
    fn create_with_fallback(&self, total_jobs: usize, params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        let result = self.create(total_jobs, params);
        
        // If creation fails, try with a reasonable fallback size
        if result.is_err() {
            let max_lines = if params.is_empty() { 0 } else { params[0] };
            eprintln!("Warning: Requested window size {} was invalid, using size 3 instead", max_lines);
            
            // Try with a reasonable fallback size
            if let Ok(mut window) = WindowWithTitle::new(total_jobs, 3, "Progress".to_string()) {
                // Explicitly enable emoji and title support
                window.set_emoji_support(true);
                window.set_title_support(true);
                return Ok(Box::new(window));
            }
            
            // Last resort: fall back to Limited mode
            eprintln!("Warning: Could not create WindowWithTitle mode, falling back to Limited mode");
            return Ok(Box::new(Limited::new(total_jobs)));
        }
        
        result
    }
    
    fn mode_name(&self) -> &'static str {
        "window_with_title"
    }
    
    fn min_params(&self) -> usize {
        1
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
                ThreadMode::Limited => self.registry.create("limited", total_jobs, &[]),
                ThreadMode::Capturing => self.registry.create("capturing", total_jobs, &[]),
                ThreadMode::Window(size) => self.registry.create("window", total_jobs, &[size]),
                ThreadMode::WindowWithTitle(size) => self.registry.create("window_with_title", total_jobs, &[size]),
            }
        } else {
            // With error propagation disabled, provide fallbacks
            match mode {
                ThreadMode::Limited => self.registry.create("limited", total_jobs, &[]),
                ThreadMode::Capturing => self.registry.create("capturing", total_jobs, &[]),
                ThreadMode::Window(size) => {
                    let result = self.registry.create("window", total_jobs, &[size]);
                    if result.is_err() {
                        // Try with a reasonable fallback size
                        eprintln!("Warning: Requested window size {} was invalid, using size 3 instead", size);
                        self.registry.create("window", total_jobs, &[3])
                    } else {
                        result
                    }
                },
                ThreadMode::WindowWithTitle(size) => {
                    let result = self.registry.create("window_with_title", total_jobs, &[size]);
                    if result.is_err() {
                        // Try with a reasonable fallback size
                        eprintln!("Warning: Requested window size {} was invalid, using size 3 instead", size);
                        let fallback = self.registry.create("window_with_title", total_jobs, &[3]);
                        if fallback.is_err() {
                            // Try with Window mode as a fallback
                            eprintln!("Warning: Could not create WindowWithTitle mode, falling back to Window mode");
                            let window = self.registry.create("window", total_jobs, &[3]);
                            if window.is_err() {
                                // Last resort: fall back to Limited mode
                                eprintln!("Warning: Could not create any window mode, falling back to Limited mode");
                                self.registry.create("limited", total_jobs, &[])
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
        let config = registry.create("limited", 10, &[]).unwrap();
        assert_eq!(config.lines_to_display(), 1);
        
        // Create a Window mode
        let config = registry.create("window", 10, &[3]).unwrap();
        assert_eq!(config.lines_to_display(), 3);
    }
    
    #[test]
    fn test_mode_creation_with_invalid_params() {
        let mut registry = ModeRegistry::new();
        registry.register(WindowCreator);
        
        // Try to create a Window mode without required params
        let result = registry.create("window", 10, &[]);
        assert!(result.is_err());
        
        // Verify the error type
        match result {
            Err(ModeCreationError::MissingParameter { param_name, mode_name }) => {
                assert_eq!(mode_name, "window");
                assert_eq!(param_name, "parameter 1");
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
} 