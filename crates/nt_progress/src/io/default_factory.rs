use std::path::Path;
use std::fmt::Debug;
use anyhow::Result;

use super::io_trait::{IOFactory, IO, IOMode};
use super::memory_io::MemoryIO;

/// Default IO factory: always uses in-memory backend by default
#[derive(Debug, Default)]
pub struct DefaultIOFactory;

impl IOFactory for DefaultIOFactory {
    /// For file paths, return an in-memory buffer instead of actual disk IO
    fn create_file_io(&self, _path: &Path, mode: IOMode) -> Result<Box<dyn IO>> {
        Ok(Box::new(MemoryIO::new(None, mode)))
    }

    /// Create an in-memory buffer with optional initial data
    fn create_memory_io(&self, initial_data: Option<Vec<u8>>, mode: IOMode) -> Result<Box<dyn IO>> {
        Ok(Box::new(MemoryIO::new(initial_data, mode)))
    }

    /// Create a string-based in-memory IO
    fn create_string_io(&self, initial_data: Option<String>, mode: IOMode) -> Result<Box<dyn IO>> {
        let data = initial_data.map(|s| s.into_bytes());
        Ok(Box::new(MemoryIO::new(data, mode)))
    }

    /// Create a custom in-memory IO initialized from source bytes
    fn create_custom_io(&self, source: &str, _config: &[(&str, &str)], mode: IOMode) -> Result<Box<dyn IO>> {
        let data = Some(source.as_bytes().to_vec());
        Ok(Box::new(MemoryIO::new(data, mode)))
    }
} 