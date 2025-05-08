use crate::errors::ModeCreationError;

/// Enum defining the available thread display modes.
///
/// This enum is used to select the mode for displaying
/// thread output in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Parameters for creating a display mode.
///
/// This struct contains the parameters needed to create a display mode,
/// providing a unified way to configure different modes with optional
/// parameters.
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
    /// Create a new ModeParameters with default values.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    ///
    /// # Returns
    /// A new ModeParameters instance with default values
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
    
    /// Create parameters for Limited mode.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    ///
    /// # Returns
    /// A ModeParameters instance configured for Limited mode
    pub fn limited(total_jobs: usize) -> Self {
        Self::new(total_jobs).with_passthrough(false)
    }
    
    /// Create parameters for Capturing mode.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    ///
    /// # Returns
    /// A ModeParameters instance configured for Capturing mode
    pub fn capturing(total_jobs: usize) -> Self {
        Self::new(total_jobs)
    }
    
    /// Create parameters for Window mode.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    /// * `max_lines` - The maximum number of lines to display
    ///
    /// # Returns
    /// A ModeParameters instance configured for Window mode
    pub fn window(total_jobs: usize, max_lines: usize) -> Self {
        Self::new(total_jobs)
            .with_max_lines(max_lines)
            .with_emoji_support(false)
            .with_title_support(false)
    }
    
    /// Create parameters for WindowWithTitle mode.
    ///
    /// # Parameters
    /// * `total_jobs` - The total number of jobs to track
    /// * `max_lines` - The maximum number of lines to display
    /// * `title` - The title to display
    ///
    /// # Returns
    /// A ModeParameters instance configured for WindowWithTitle mode
    pub fn window_with_title(total_jobs: usize, max_lines: usize, title: String) -> Self {
        Self::new(total_jobs)
            .with_max_lines(max_lines)
            .with_title(title)
            .with_emoji_support(true)
            .with_title_support(true)
    }
    
    /// Validate that the parameters are valid for the given mode.
    ///
    /// # Parameters
    /// * `mode_name` - The name of the mode to validate for
    ///
    /// # Returns
    /// Ok(()) if the parameters are valid, ModeCreationError otherwise
    ///
    /// # Errors
    /// Returns ModeCreationError if the parameters are invalid for the mode
    pub fn validate(&self, mode_name: &str) -> Result<(), ModeCreationError> {
        // First validate that required parameters are present
        self.validate_required_params(mode_name)?;
        
        // Then validate that parameter values are valid
        self.validate_param_values(mode_name)?;
        
        Ok(())
    }
    
    /// Get the total number of jobs.
    ///
    /// # Returns
    /// The total number of jobs
    pub fn total_jobs(&self) -> usize {
        self.total_jobs
    }
    
    /// Get the maximum number of lines to display.
    ///
    /// # Returns
    /// The maximum number of lines, or None if not set
    pub fn max_lines(&self) -> Option<usize> {
        self.max_lines
    }
    
    /// Get the title string.
    ///
    /// # Returns
    /// A reference to the title string, or None if not set
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    
    /// Check if emoji support is enabled.
    ///
    /// # Returns
    /// true if emoji support is enabled, None if not set
    pub fn emoji_support(&self) -> Option<bool> {
        self.emoji_support
    }
    
    /// Check if title support is enabled.
    ///
    /// # Returns
    /// true if title support is enabled, None if not set
    pub fn title_support(&self) -> Option<bool> {
        self.title_support
    }
    
    /// Check if passthrough is enabled.
    ///
    /// # Returns
    /// true if passthrough is enabled, None if not set
    pub fn passthrough(&self) -> Option<bool> {
        self.passthrough
    }
    
    /// Set the maximum number of lines to display.
    ///
    /// # Parameters
    /// * `max_lines` - The maximum number of lines
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = Some(max_lines);
        self
    }
    
    /// Set the title string.
    ///
    /// # Parameters
    /// * `title` - The title string
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
    
    /// Set whether emoji support is enabled.
    ///
    /// # Parameters
    /// * `enabled` - Whether emoji support is enabled
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_emoji_support(mut self, enabled: bool) -> Self {
        self.emoji_support = Some(enabled);
        self
    }
    
    /// Set whether title support is enabled.
    ///
    /// # Parameters
    /// * `enabled` - Whether title support is enabled
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_title_support(mut self, enabled: bool) -> Self {
        self.title_support = Some(enabled);
        self
    }
    
    /// Set whether passthrough is enabled.
    ///
    /// # Parameters
    /// * `enabled` - Whether passthrough is enabled
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_passthrough(mut self, enabled: bool) -> Self {
        self.passthrough = Some(enabled);
        self
    }
    
    /// Validate that the required parameters are present for the given mode.
    ///
    /// # Parameters
    /// * `mode_name` - The name of the mode to validate for
    ///
    /// # Returns
    /// Ok(()) if the required parameters are present, ModeCreationError otherwise
    ///
    /// # Errors
    /// Returns ModeCreationError if a required parameter is missing
    fn validate_required_params(&self, mode_name: &str) -> Result<(), ModeCreationError> {
        match mode_name.to_lowercase().as_str() {
            "window" => {
                if self.max_lines.is_none() {
                    return Err(ModeCreationError::MissingParameter {
                        param_name: "max_lines".to_string(),
                        mode_name: mode_name.to_string(),
                        reason: Some("Max lines is required for window modes".to_string()),
                    });
                }
            }
            "window_with_title" | "windowwithtitle" => {
                if self.max_lines.is_none() {
                    return Err(ModeCreationError::MissingParameter {
                        param_name: "max_lines".to_string(),
                        mode_name: mode_name.to_string(),
                        reason: Some("Max lines is required for window modes".to_string()),
                    });
                }
                
                if self.title.is_none() {
                    return Err(ModeCreationError::MissingParameter {
                        param_name: "title".to_string(),
                        mode_name: mode_name.to_string(),
                        reason: Some("Title is required for WindowWithTitle mode".to_string()),
                    });
                }
            }
            "limited" | "capturing" => {
                // These modes don't have required parameters
            }
            _ => {
                return Err(ModeCreationError::Implementation(
                    format!("Unknown mode: {}", mode_name)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate that the parameter values are valid for the given mode.
    ///
    /// # Parameters
    /// * `mode_name` - The name of the mode to validate for
    ///
    /// # Returns
    /// Ok(()) if the parameter values are valid, ModeCreationError otherwise
    ///
    /// # Errors
    /// Returns ModeCreationError if a parameter value is invalid
    fn validate_param_values(&self, mode_name: &str) -> Result<(), ModeCreationError> {
        // Check that max_lines is valid if present
        if let Some(max_lines) = self.max_lines {
            match mode_name.to_lowercase().as_str() {
                "window" => {
                    if max_lines < 1 {
                        return Err(ModeCreationError::InvalidWindowSize {
                            size: max_lines,
                            min_size: 1,
                            mode_name: "Window".to_string(),
                            reason: Some("Window size must be at least 1 line".to_string()),
                        });
                    }
                }
                "window_with_title" | "windowwithtitle" => {
                    if max_lines < 2 {
                        return Err(ModeCreationError::InvalidWindowSize {
                            size: max_lines,
                            min_size: 2,
                            mode_name: "WindowWithTitle".to_string(),
                            reason: Some("WindowWithTitle requires at least 2 lines (1 for title, 1 for content)".to_string()),
                        });
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mode_parameters_creation() {
        let params = ModeParameters::new(10);
        assert_eq!(params.total_jobs(), 10);
        assert_eq!(params.max_lines(), None);
        assert_eq!(params.title(), None);
        assert_eq!(params.emoji_support(), None);
        assert_eq!(params.title_support(), None);
        assert_eq!(params.passthrough(), None);
        
        let params = ModeParameters::window(10, 5);
        assert_eq!(params.total_jobs(), 10);
        assert_eq!(params.max_lines(), Some(5));
        assert_eq!(params.emoji_support(), Some(false));
        assert_eq!(params.title_support(), Some(false));
        
        let params = ModeParameters::window_with_title(10, 5, "Title".to_string());
        assert_eq!(params.total_jobs(), 10);
        assert_eq!(params.max_lines(), Some(5));
        assert_eq!(params.title(), Some("Title"));
        assert_eq!(params.emoji_support(), Some(true));
        assert_eq!(params.title_support(), Some(true));
        
        let params = ModeParameters::limited(10);
        assert_eq!(params.total_jobs(), 10);
        assert_eq!(params.passthrough(), Some(false));
        
        let params = ModeParameters::capturing(10);
        assert_eq!(params.total_jobs(), 10);
    }
    
    #[test]
    fn test_mode_parameters_validation() {
        let valid_window = ModeParameters::window(10, 5);
        assert!(valid_window.validate("window").is_ok());
        
        let valid_title = ModeParameters::window_with_title(10, 5, "Title".to_string());
        assert!(valid_title.validate("windowwithtitle").is_ok());
        
        let valid_limited = ModeParameters::limited(10);
        assert!(valid_limited.validate("limited").is_ok());
        
        let valid_capturing = ModeParameters::capturing(10);
        assert!(valid_capturing.validate("capturing").is_ok());
    }
    
    #[test]
    fn test_mode_parameters_validation_errors() {
        // Missing max_lines for window
        let invalid_window = ModeParameters::new(10);
        assert!(matches!(
            invalid_window.validate("window"),
            Err(ModeCreationError::MissingParameter { .. })
        ));
        
        // Missing title for WindowWithTitle
        let invalid_title = ModeParameters::new(10).with_max_lines(5);
        assert!(matches!(
            invalid_title.validate("windowwithtitle"),
            Err(ModeCreationError::MissingParameter { .. })
        ));
        
        // Invalid max_lines for window
        let invalid_size = ModeParameters::window(10, 0);
        assert!(matches!(
            invalid_size.validate("window"),
            Err(ModeCreationError::InvalidWindowSize { .. })
        ));
        
        // Invalid max_lines for WindowWithTitle
        let invalid_size = ModeParameters::window_with_title(10, 1, "Title".to_string());
        assert!(matches!(
            invalid_size.validate("windowwithtitle"),
            Err(ModeCreationError::InvalidWindowSize { .. })
        ));
        
        // Unknown mode
        let params = ModeParameters::new(10);
        assert!(matches!(
            params.validate("unknown"),
            Err(ModeCreationError::Implementation(_))
        ));
    }
    
    #[test]
    fn test_mode_parameters_builder_pattern() {
        let params = ModeParameters::new(10)
            .with_max_lines(5)
            .with_title("Title".to_string())
            .with_emoji_support(true)
            .with_title_support(true)
            .with_passthrough(false);
        
        assert_eq!(params.total_jobs(), 10);
        assert_eq!(params.max_lines(), Some(5));
        assert_eq!(params.title(), Some("Title"));
        assert_eq!(params.emoji_support(), Some(true));
        assert_eq!(params.title_support(), Some(true));
        assert_eq!(params.passthrough(), Some(false));
    }
    
    #[test]
    fn test_mode_parameters_unknown_mode() {
        let params = ModeParameters::new(10);
        assert!(matches!(
            params.validate("unknown"),
            Err(ModeCreationError::Implementation(_))
        ));
    }
} 