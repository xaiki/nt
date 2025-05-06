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
}

impl fmt::Display for ProgressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgressError::ModeCreation(err) => write!(f, "Mode creation error: {}", err),
            ProgressError::TaskOperation(msg) => write!(f, "Task operation error: {}", msg),
            ProgressError::DisplayOperation(msg) => write!(f, "Display operation error: {}", msg),
            ProgressError::External(err) => write!(f, "External error: {}", err),
            ProgressError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl Error for ProgressError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProgressError::ModeCreation(err) => Some(err),
            ProgressError::External(err) => Some(err.as_ref()),
            ProgressError::Io(err) => Some(err),
            _ => None,
        }
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