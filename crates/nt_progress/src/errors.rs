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

/// Errors that can occur when creating or configuring a display mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModeCreationError {
    /// Window size is invalid (e.g., zero or too small)
    InvalidWindowSize {
        /// The size that was provided
        size: usize,
        /// The minimum required size
        min_size: usize,
        /// The mode that was being created
        mode_name: String,
    },
    /// A required window parameter is missing (e.g., title)
    MissingParameter {
        /// The name of the missing parameter
        param_name: String,
        /// The mode that was being created
        mode_name: String,
    },
    /// An error occurred in the underlying mode implementation
    Implementation(String),
}

impl fmt::Display for ModeCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModeCreationError::InvalidWindowSize { size, min_size, mode_name } => {
                write!(f, "Invalid window size for {} mode: {} (minimum: {})", mode_name, size, min_size)
            },
            ModeCreationError::MissingParameter { param_name, mode_name } => {
                write!(f, "Missing required parameter '{}' for {} mode", param_name, mode_name)
            },
            ModeCreationError::Implementation(msg) => {
                write!(f, "Mode implementation error: {}", msg)
            }
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