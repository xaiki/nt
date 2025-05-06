use std::collections::HashMap;
use super::{ThreadConfig, ThreadMode, Limited, Capturing, Window, WindowWithTitle};
use crate::errors::ModeCreationError;
use std::sync::Once;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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

/// Trait for mode creators that can create a specific ThreadConfig implementation
pub trait ModeCreator: Send + Sync {
    /// Create a ThreadConfig instance with the given parameters
    fn create(&self, total_jobs: usize, params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError>;
    
    /// Create a ThreadConfig with fallback options if the primary creation fails
    fn create_with_fallback(&self, total_jobs: usize, params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        // By default, just try the regular create method
        self.create(total_jobs, params)
    }
    
    /// Get the name of the mode this creator creates
    fn mode_name(&self) -> &'static str;
    
    /// Get the minimum required parameters for this mode
    fn min_params(&self) -> usize;
}

/// A registry for mode creators
/// 
/// This registry allows for centralized management of mode creation
/// and enables easy addition of new modes without modifying existing code.
#[derive(Default)]
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
pub struct WindowWithTitleCreator;

impl ModeCreator for WindowWithTitleCreator {
    fn create(&self, total_jobs: usize, params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        let max_lines = params[0];
        WindowWithTitle::new(total_jobs, max_lines).map(|w| Box::new(w) as Box<dyn ThreadConfig>)
    }
    
    fn create_with_fallback(&self, total_jobs: usize, params: &[usize]) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
        let result = self.create(total_jobs, params);
        
        // If creation fails, try with a reasonable fallback size
        if result.is_err() {
            let max_lines = if params.is_empty() { 0 } else { params[0] };
            eprintln!("Warning: Requested window size {} was invalid, using size 3 instead", max_lines);
            
            // Try with a reasonable fallback size
            if let Ok(window) = WindowWithTitle::new(total_jobs, 3) {
                return Ok(Box::new(window));
            }
            
            // Try with Window mode as a fallback
            if let Ok(window) = Window::new(total_jobs, 3) {
                eprintln!("Warning: Could not create WindowWithTitle mode, falling back to Window mode");
                return Ok(Box::new(window));
            }
            
            // Last resort: fall back to Limited mode
            eprintln!("Warning: Could not create any window mode, falling back to Limited mode");
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

// Singleton registry instance
static REGISTRY_INIT: Once = Once::new();
static mut REGISTRY: Option<Arc<Mutex<ModeRegistry>>> = None;

/// Get the global ModeRegistry instance.
/// 
/// This function initializes the registry with the standard mode creators on first call.
pub fn get_registry() -> Arc<Mutex<ModeRegistry>> {
    unsafe {
        REGISTRY_INIT.call_once(|| {
            let mut registry = ModeRegistry::new();
            
            // Register standard creators
            registry.register(LimitedCreator);
            registry.register(CapturingCreator);
            registry.register(WindowCreator);
            registry.register(WindowWithTitleCreator);
            
            REGISTRY = Some(Arc::new(Mutex::new(registry)));
        });
        
        REGISTRY.clone().unwrap()
    }
}

/// Create a thread config using the global registry
pub fn create_thread_config(mode: ThreadMode, total_jobs: usize) -> Result<Box<dyn ThreadConfig>, ModeCreationError> {
    let registry = get_registry();
    
    // Lock registry first and then use it
    let registry_guard = registry.lock().unwrap();
    
    // Try to create the config normally first
    let creation_result = registry_guard.create_from_mode(mode.clone(), total_jobs);
    
    // Drop the lock before processing the result
    drop(registry_guard);
    
    if let Err(_) = &creation_result {
        // If error propagation is enabled, return the error as-is
        if should_propagate_errors() {
            return creation_result;
        }
        
        // For non-error-propagation cases, try to recover using the fallback mechanism
        match &mode {
            ThreadMode::Window(_) => {
                if let Some(creator) = registry.lock().unwrap().creators.get("window") {
                    return creator.create_with_fallback(total_jobs, &[3]);
                }
            },
            ThreadMode::WindowWithTitle(_) => {
                if let Some(creator) = registry.lock().unwrap().creators.get("window_with_title") {
                    return creator.create_with_fallback(total_jobs, &[3]);
                }
            },
            _ => {}
        }
    }
    
    creation_result
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
    fn test_global_registry() {
        let registry = get_registry();
        let registry = registry.lock().unwrap();
        
        assert!(registry.creators.contains_key("limited"));
        assert!(registry.creators.contains_key("capturing"));
        assert!(registry.creators.contains_key("window"));
        assert!(registry.creators.contains_key("window_with_title"));
    }
    
    #[test]
    fn test_create_from_mode() {
        let registry = get_registry();
        let registry = registry.lock().unwrap();
        
        // Test creating from ThreadMode::Limited
        let config = registry.create_from_mode(ThreadMode::Limited, 10).unwrap();
        assert_eq!(config.lines_to_display(), 1);
        
        // Test creating from ThreadMode::Window
        let config = registry.create_from_mode(ThreadMode::Window(3), 10).unwrap();
        assert_eq!(config.lines_to_display(), 3);
    }
} 