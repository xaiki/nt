use std::collections::HashMap;
use std::fmt::Debug;
use anyhow::Result;

use super::ProgressWriter;

/// Capabilities that a custom writer can support
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WriterCapabilities {
    /// Whether the writer supports custom formatting
    pub supports_formatting: bool,
    /// Whether the writer supports output filtering
    pub supports_filtering: bool,
    /// Whether the writer supports output redirection
    pub supports_redirection: bool,
    /// Whether the writer supports async operations
    pub supports_async: bool,
}

/// A trait for custom writers that can be registered with the system
pub trait CustomWriter: ProgressWriter {
    /// Get the name of this writer
    fn name(&self) -> &str;

    /// Get the capabilities of this writer
    fn capabilities(&self) -> WriterCapabilities;

    /// Get the configuration of this writer
    fn config(&self) -> HashMap<String, String>;
}

/// A registry for managing custom writers
#[derive(Debug, Default)]
pub struct WriterRegistry {
    writers: HashMap<String, Box<dyn CustomWriter>>,
}

impl WriterRegistry {
    /// Create a new empty writer registry
    pub fn new() -> Self {
        Self {
            writers: HashMap::new(),
        }
    }

    /// Register a new writer
    pub fn register<W: CustomWriter + 'static>(&mut self, writer: W) -> Result<()> {
        let name = writer.name().to_string();
        if self.writers.contains_key(&name) {
            anyhow::bail!("Writer '{}' is already registered", name);
        }
        self.writers.insert(name, Box::new(writer));
        Ok(())
    }

    /// Get a writer by name
    pub fn get(&self, name: &str) -> Option<&dyn CustomWriter> {
        self.writers.get(name).map(|w| w.as_ref())
    }

    /// List all registered writers
    pub fn list(&self) -> Vec<&str> {
        self.writers.keys().map(|k| k.as_str()).collect()
    }

    /// Remove a writer by name
    pub fn remove(&mut self, name: &str) -> Option<Box<dyn CustomWriter>> {
        self.writers.remove(name)
    }
} 