use std::fmt;
use std::error::Error;
use std::io;

/// Errors that can occur when working with the nt_progress library
#[derive(Debug)]
pub enum ProgressError {
    /// Error when creating or configuring a display mode
    ModeCreation(ModeCreationError),
    /// Error when interacting with a task handle
    TaskOperation(String),
    /// Error when operating on the progress display
    DisplayOperation(String),
    /// Error from an external source (e.g., IO)
    External(Box<dyn Error + Send + Sync>),
    /// IO error
    Io(io::Error),
    /// Error with context information
    WithContext(Box<ProgressError>, ErrorContext),
}

/// Context information for errors
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The operation that failed
    pub operation: String,
    /// The component where the error occurred
    pub component: String,
    /// Additional context information
    pub details: Option<String>,
    /// Thread ID if applicable
    pub thread_id: Option<usize>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            component: component.into(),
            details: None,
            thread_id: None,
        }
    }

    /// Add details to the context
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add thread ID to the context
    pub fn with_thread_id(mut self, thread_id: usize) -> Self {
        self.thread_id = Some(thread_id);
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "in {} during {}", self.component, self.operation)?;
        
        if let Some(thread_id) = self.thread_id {
            write!(f, " (thread {})", thread_id)?;
        }
        
        if let Some(details) = &self.details {
            write!(f, ": {}", details)?;
        }
        
        Ok(())
    }
}

impl fmt::Display for ProgressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgressError::ModeCreation(err) => write!(f, "Mode creation error: {}", err),
            ProgressError::TaskOperation(msg) => write!(f, "Task operation error: {}", msg),
            ProgressError::DisplayOperation(msg) => write!(f, "Display operation error: {}", msg),
            ProgressError::External(err) => write!(f, "External error: {}", err),
            ProgressError::Io(err) => write!(f, "IO error: {}", err),
            ProgressError::WithContext(err, ctx) => write!(f, "{} ({})", err, ctx),
        }
    }
}

impl Error for ProgressError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProgressError::ModeCreation(err) => Some(err),
            ProgressError::External(err) => Some(err.as_ref()),
            ProgressError::Io(err) => Some(err),
            ProgressError::WithContext(err, _) => Some(err.as_ref()),
            _ => None,
        }
    }
}

/// Extension trait to add context to errors
pub trait ContextExt<T, E> {
    /// Add context information to the error
    fn context(self, ctx: ErrorContext) -> Result<T, ProgressError>;
    
    /// Add context with operation and component information
    fn with_context(self, operation: impl Into<String>, component: impl Into<String>) -> Result<T, ProgressError>;
}

impl<T, E> ContextExt<T, E> for Result<T, E>
where
    E: Into<ProgressError>,
{
    fn context(self, ctx: ErrorContext) -> Result<T, ProgressError> {
        self.map_err(|e| {
            let err = e.into();
            ProgressError::WithContext(Box::new(err), ctx)
        })
    }
    
    fn with_context(self, operation: impl Into<String>, component: impl Into<String>) -> Result<T, ProgressError> {
        let ctx = ErrorContext::new(operation, component);
        self.context(ctx)
    }
}

/// Common error context builders for frequently used components
///
/// This module provides pre-configured error context builders for
/// the most commonly used components, reducing boilerplate code.
pub mod context {
    use super::ErrorContext;
    
    /// Context builder for ProgressDisplay operations
    pub fn display(operation: impl Into<String>) -> ErrorContext {
        ErrorContext::new(operation, "ProgressDisplay")
    }
    
    /// Context builder for TaskHandle operations
    pub fn task(operation: impl Into<String>, thread_id: usize) -> ErrorContext {
        ErrorContext::new(operation, "TaskHandle").with_thread_id(thread_id)
    }
    
    /// Context builder for ThreadLogger operations
    pub fn logger(operation: impl Into<String>, thread_id: usize) -> ErrorContext {
        ErrorContext::new(operation, "ThreadLogger").with_thread_id(thread_id)
    }
    
    /// Context builder for Config operations
    pub fn config(operation: impl Into<String>) -> ErrorContext {
        ErrorContext::new(operation, "Config")
    }
    
    /// Context builder for Template operations
    pub fn template(operation: impl Into<String>) -> ErrorContext {
        ErrorContext::new(operation, "ProgressTemplate")
    }
}

/// Extension trait to add context to result types with pre-defined components
pub trait CommonContextExt<T, E> {
    /// Add ProgressDisplay context
    fn display_context(self, operation: impl Into<String>) -> Result<T, ProgressError>;
    
    /// Add TaskHandle context
    fn task_context(self, operation: impl Into<String>, thread_id: usize) -> Result<T, ProgressError>;
    
    /// Add ThreadLogger context
    fn logger_context(self, operation: impl Into<String>, thread_id: usize) -> Result<T, ProgressError>;
    
    /// Add Config context
    fn config_context(self, operation: impl Into<String>) -> Result<T, ProgressError>;
    
    /// Add ProgressTemplate context
    fn template_context(self, operation: impl Into<String>) -> Result<T, ProgressError>;
}

impl<T, E> CommonContextExt<T, E> for Result<T, E>
where
    E: Into<ProgressError>,
{
    fn display_context(self, operation: impl Into<String>) -> Result<T, ProgressError> {
        self.context(context::display(operation))
    }
    
    fn task_context(self, operation: impl Into<String>, thread_id: usize) -> Result<T, ProgressError> {
        self.context(context::task(operation, thread_id))
    }
    
    fn logger_context(self, operation: impl Into<String>, thread_id: usize) -> Result<T, ProgressError> {
        self.context(context::logger(operation, thread_id))
    }
    
    fn config_context(self, operation: impl Into<String>) -> Result<T, ProgressError> {
        self.context(context::config(operation))
    }
    
    fn template_context(self, operation: impl Into<String>) -> Result<T, ProgressError> {
        self.context(context::template(operation))
    }
}

/// Errors that can occur when creating or configuring a display mode
#[derive(Debug)]
pub enum ModeCreationError {
    /// Window size is invalid (e.g., zero or too small)
    InvalidWindowSize {
        /// The size that was provided
        size: usize,
        /// The minimum required size
        min_size: usize,
        /// The mode that was being created
        mode_name: String,
        /// Optional reason for the failure
        reason: Option<String>,
    },
    /// A required window parameter is missing (e.g., title)
    MissingParameter {
        /// The name of the missing parameter
        param_name: String,
        /// The mode that was being created
        mode_name: String,
        /// Optional reason why the parameter is required
        reason: Option<String>,
    },
    /// An error occurred in the underlying mode implementation
    Implementation(String),
    /// Operation attempted on a mode that does not support titles
    TitleNotSupported {
        /// The mode that was being used
        mode_name: String,
        /// Optional reason why titles are not supported
        reason: Option<String>,
    },
    /// Operation attempted on a mode that does not support emojis
    EmojiNotSupported {
        /// The mode that was being used
        mode_name: String,
        /// Optional reason why emojis are not supported
        reason: Option<String>,
    },
    /// The mode is not registered in the factory
    ModeNotRegistered {
        /// The name of the mode that was requested
        mode_name: String,
        /// List of available modes
        available_modes: Vec<String>,
    },
    /// The mode is not compatible with the current configuration
    IncompatibleConfiguration {
        /// The mode that was being used
        mode_name: String,
        /// The configuration that caused the incompatibility
        config: String,
        /// Optional reason for the incompatibility
        reason: Option<String>,
    },
    /// The mode requires a feature that is not available
    FeatureNotAvailable {
        /// The mode that was being used
        mode_name: String,
        /// The feature that is not available
        feature: String,
        /// Optional reason why the feature is not available
        reason: Option<String>,
    },
    /// Validation failed before mode creation
    ValidationError {
        /// The mode that was being created
        mode_name: String,
        /// The validation rule that failed
        rule: String,
        /// The value that failed validation
        value: String,
        /// Optional reason for the validation failure
        reason: Option<String>,
    },
}

impl fmt::Display for ModeCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModeCreationError::InvalidWindowSize { size, min_size, mode_name, reason } => {
                write!(f, "Invalid window size for {} mode: {} (minimum: {})", mode_name, size, min_size)?;
                if let Some(reason) = reason {
                    write!(f, " - {}", reason)?;
                }
                Ok(())
            },
            ModeCreationError::MissingParameter { param_name, mode_name, reason } => {
                write!(f, "Missing required parameter '{}' for {} mode", param_name, mode_name)?;
                if let Some(reason) = reason {
                    write!(f, " - {}", reason)?;
                }
                Ok(())
            },
            ModeCreationError::Implementation(msg) => {
                write!(f, "Mode implementation error: {}", msg)
            },
            ModeCreationError::TitleNotSupported { mode_name, reason } => {
                write!(f, "Operation attempted on {} mode which does not support titles", mode_name)?;
                if let Some(reason) = reason {
                    write!(f, " - {}", reason)?;
                }
                Ok(())
            },
            ModeCreationError::EmojiNotSupported { mode_name, reason } => {
                write!(f, "Operation attempted on {} mode which does not support emojis", mode_name)?;
                if let Some(reason) = reason {
                    write!(f, " - {}", reason)?;
                }
                Ok(())
            },
            ModeCreationError::ModeNotRegistered { mode_name, available_modes } => {
                write!(f, "Mode '{}' is not registered. Available modes: {}", 
                    mode_name, available_modes.join(", "))
            },
            ModeCreationError::IncompatibleConfiguration { mode_name, config, reason } => {
                write!(f, "Mode '{}' is not compatible with configuration: {}", mode_name, config)?;
                if let Some(reason) = reason {
                    write!(f, " - {}", reason)?;
                }
                Ok(())
            },
            ModeCreationError::FeatureNotAvailable { mode_name, feature, reason } => {
                write!(f, "Mode '{}' requires feature '{}' which is not available", mode_name, feature)?;
                if let Some(reason) = reason {
                    write!(f, " - {}", reason)?;
                }
                Ok(())
            },
            ModeCreationError::ValidationError { mode_name, rule, value, reason } => {
                write!(f, "Validation failed for {} mode: rule '{}' failed for value '{}'", mode_name, rule, value)?;
                if let Some(reason) = reason {
                    write!(f, " - {}", reason)?;
                }
                Ok(())
            },
        }
    }
}

impl Error for ModeCreationError {}

/// Conversion from ModeCreationError to ProgressError
impl From<ModeCreationError> for ProgressError {
    fn from(err: ModeCreationError) -> Self {
        ProgressError::ModeCreation(err)
    }
}

/// Conversion from a string error to ModeCreationError
impl From<String> for ModeCreationError {
    fn from(msg: String) -> Self {
        ModeCreationError::Implementation(msg)
    }
}

/// Conversion from ModeCreationError to String (for backward compatibility)
impl From<ModeCreationError> for String {
    fn from(err: ModeCreationError) -> Self {
        err.to_string()
    }
}

/// Conversion from io::Error to ProgressError
impl From<io::Error> for ProgressError {
    fn from(err: io::Error) -> Self {
        ProgressError::Io(err)
    }
}

/// Convert &str to ProgressError::TaskOperation
impl From<&str> for ProgressError {
    fn from(msg: &str) -> Self {
        ProgressError::TaskOperation(msg.to_string())
    }
}

/// Logging helper for error debugging
pub fn log_error(error: &ProgressError) {
    let mut current_error: Option<&dyn Error> = Some(error);
    let mut depth = 0;
    
    eprintln!("Error chain:");
    while let Some(error) = current_error {
        eprintln!("  {}: {}", depth, error);
        current_error = error.source();
        depth += 1;
    }
}

/// Creates a debug-friendly error description with full context
pub fn format_error_debug(error: &ProgressError) -> String {
    let mut result = String::new();
    let mut current_error: Option<&dyn Error> = Some(error);
    let mut depth = 0;
    
    result.push_str("Error chain:\n");
    while let Some(error) = current_error {
        result.push_str(&format!("  {}: {}\n", depth, error));
        current_error = error.source();
        depth += 1;
    }
    
    result
}

impl ProgressError {
    /// Add context to an existing ProgressError
    pub fn into_context(self, ctx: ErrorContext) -> Self {
        ProgressError::WithContext(Box::new(self), ctx)
    }
} 