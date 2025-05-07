use std::io::{Read, Write};
use std::path::Path;
use std::fmt::Debug;
use anyhow::Result;

/// IOCapabilities defines the capabilities of an IO implementation
#[derive(Debug, Clone, PartialEq)]
pub struct IOCapabilities {
    /// Whether the implementation supports input operations
    pub supports_input: bool,
    /// Whether the implementation supports output operations
    pub supports_output: bool,
    /// Whether the implementation supports synchronous operations
    pub supports_sync: bool,
    /// Whether the implementation supports asynchronous operations
    pub supports_async: bool,
    /// Whether the implementation supports seeking
    pub supports_seek: bool,
    /// Whether the implementation supports filtering
    pub supports_filtering: bool,
}

impl Default for IOCapabilities {
    fn default() -> Self {
        Self {
            supports_input: false,
            supports_output: true,  // Most common case
            supports_sync: true,    // Most common case
            supports_async: false,
            supports_seek: false,
            supports_filtering: false,
        }
    }
}

/// Common trait for all IO operations
/// 
/// This trait defines the core functionality that all IO implementations
/// must provide, regardless of their specific capabilities.
pub trait IO: Send + Sync + Debug {
    /// Get the name of this IO implementation
    fn name(&self) -> &str;
    
    /// Get the capabilities of this IO implementation
    fn capabilities(&self) -> IOCapabilities;
    
    /// Check if the IO implementation is ready for operations
    fn is_ready(&self) -> bool;
    
    /// Close the IO implementation and release any resources
    fn close(&mut self) -> Result<()>;
}

/// Trait for IO implementations that support output operations
pub trait OutputIO: IO + Write {
    /// Write a line of text
    fn write_line(&mut self, line: &str) -> Result<()>;
    
    /// Flush any buffered content
    fn flush_output(&mut self) -> Result<()>;
}

/// Trait for IO implementations that support input operations
pub trait InputIO: IO + Read {
    /// Read a line of text
    fn read_line(&mut self) -> Result<String>;
    
    /// Check if there is data available to read
    fn has_data_available(&self) -> bool;
}

/// Trait for IO implementations that support seeking
pub trait SeekableIO: IO {
    /// Seek to a specific position
    fn seek(&mut self, position: u64) -> Result<u64>;
    
    /// Get the current position
    fn position(&self) -> Result<u64>;
    
    /// Get the size of the underlying resource
    fn size(&self) -> Result<u64>;
}

/// Trait for IO implementations that support filtering
pub trait FilterableIO: IO {
    /// Set a filter pattern to apply to input/output
    fn set_filter(&mut self, pattern: &str) -> Result<()>;
    
    /// Remove any existing filter
    fn clear_filter(&mut self) -> Result<()>;
    
    /// Get the current filter pattern, if any
    fn get_filter(&self) -> Option<String>;
}

/// Factory trait for creating IO implementations
pub trait IOFactory: Send + Sync + Debug {
    /// Create an IO implementation for a file at the given path
    fn create_file_io(&self, path: &Path, mode: IOMode) -> Result<Box<dyn IO>>;
    
    /// Create an IO implementation for a memory buffer
    fn create_memory_io(&self, initial_data: Option<Vec<u8>>, mode: IOMode) -> Result<Box<dyn IO>>;
    
    /// Create an IO implementation for a string buffer
    fn create_string_io(&self, initial_data: Option<String>, mode: IOMode) -> Result<Box<dyn IO>>;
    
    /// Create an IO implementation for a custom source
    fn create_custom_io(&self, source: &str, config: &[(&str, &str)], mode: IOMode) -> Result<Box<dyn IO>>;
}

/// Mode for opening an IO implementation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IOMode {
    /// Read-only mode
    Read,
    /// Write-only mode
    Write,
    /// Append mode (write but don't truncate)
    Append,
    /// Read and write mode
    ReadWrite,
} 