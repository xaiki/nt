use std::io::{self, Read, Write, Cursor};
use std::fmt::Debug;
use anyhow::Result;

use super::io_trait::{IO, InputIO, OutputIO, SeekableIO, IOCapabilities, IOMode};

/// A memory-based IO implementation
pub struct MemoryIO {
    /// The internal cursor for reading and writing
    cursor: Cursor<Vec<u8>>,
    /// The mode in which the memory buffer is opened
    mode: IOMode,
    /// The capabilities of this implementation
    capabilities: IOCapabilities,
    /// Whether the buffer has been closed
    closed: bool,
}

impl MemoryIO {
    /// Create a new MemoryIO instance
    pub fn new(initial_data: Option<Vec<u8>>, mode: IOMode) -> Self {
        let buffer = initial_data.unwrap_or_default();
        
        let capabilities = IOCapabilities {
            supports_input: matches!(mode, IOMode::Read | IOMode::ReadWrite),
            supports_output: matches!(mode, IOMode::Write | IOMode::Append | IOMode::ReadWrite),
            supports_sync: true,
            supports_async: false,
            supports_seek: true,
            supports_filtering: false,
        };
        
        Self {
            cursor: Cursor::new(buffer),
            mode,
            capabilities,
            closed: false,
        }
    }
    
    /// Ensure the buffer can be read from
    fn ensure_readable(&self) -> Result<()> {
        if self.closed {
            anyhow::bail!("Cannot read from closed memory buffer");
        }
        
        if !self.capabilities.supports_input {
            anyhow::bail!("Memory buffer not opened for reading");
        }
        
        Ok(())
    }
    
    /// Ensure the buffer can be written to
    fn ensure_writable(&self) -> Result<()> {
        if self.closed {
            anyhow::bail!("Cannot write to closed memory buffer");
        }
        
        if !self.capabilities.supports_output {
            anyhow::bail!("Memory buffer not opened for writing");
        }
        
        Ok(())
    }
    
    /// Get a copy of the buffer contents
    pub fn get_contents(&self) -> Vec<u8> {
        self.cursor.get_ref().clone()
    }
    
    /// Get a copy of the buffer contents as a string, if valid UTF-8
    pub fn get_contents_utf8(&self) -> Result<String> {
        let bytes = self.cursor.get_ref();
        let string = String::from_utf8(bytes.clone())
            .map_err(|e| anyhow::anyhow!("Memory buffer contains invalid UTF-8: {}", e))?;
        Ok(string)
    }
}

impl Debug for MemoryIO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryIO")
            .field("mode", &self.mode)
            .field("capabilities", &self.capabilities)
            .field("closed", &self.closed)
            .field("position", &self.cursor.position())
            .field("length", &self.cursor.get_ref().len())
            .finish()
    }
}

impl IO for MemoryIO {
    fn name(&self) -> &str {
        "memory_io"
    }
    
    fn capabilities(&self) -> IOCapabilities {
        self.capabilities.clone()
    }
    
    fn is_ready(&self) -> bool {
        !self.closed
    }
    
    fn close(&mut self) -> Result<()> {
        self.closed = true;
        Ok(())
    }
}

impl Read for MemoryIO {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.ensure_readable().is_err() {
            return Err(io::Error::other(anyhow::anyhow!("Cannot read from memory buffer")));
        }
        
        self.cursor.read(buf)
    }
}

impl Write for MemoryIO {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.ensure_writable().is_err() {
            return Err(io::Error::other(anyhow::anyhow!("Cannot write to memory buffer")));
        }
        
        // Handle append mode specially
        if self.mode == IOMode::Append && self.cursor.position() != self.cursor.get_ref().len() as u64 {
            self.cursor.set_position(self.cursor.get_ref().len() as u64);
        }
        
        self.cursor.write(buf)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        // No need to flush in-memory buffer
        Ok(())
    }
}

impl InputIO for MemoryIO {
    fn read_line(&mut self) -> Result<String> {
        self.ensure_readable()?;
        
        let buffer = self.cursor.get_ref();
        
        // Find the current position in the buffer
        let pos = self.cursor.position() as usize;
        if pos >= buffer.len() {
            return Ok(String::new()); // EOF
        }
        
        // Find the next newline character from the current position
        let slice = &buffer[pos..];
        let mut found_newline = false;
        let mut line_end = 0;
        
        for (i, &b) in slice.iter().enumerate() {
            if b == b'\n' {
                found_newline = true;
                line_end = i;
                break;
            }
        }
        
        // If no newline was found, read to the end
        if !found_newline {
            line_end = slice.len();
        }
        
        // Convert the slice to a string or return an error
        let result = match std::str::from_utf8(&slice[..line_end]) {
            Ok(s) => s.to_string(),
            Err(_) => return Err(anyhow::anyhow!("Invalid UTF-8 in line")),
        };
        
        // Update the cursor position
        let new_pos = pos + line_end + if found_newline { 1 } else { 0 };
        self.cursor.set_position(new_pos as u64);
        
        Ok(result)
    }
    
    fn has_data_available(&self) -> bool {
        if self.ensure_readable().is_err() {
            return false;
        }
        
        self.cursor.position() < self.cursor.get_ref().len() as u64
    }
}

impl OutputIO for MemoryIO {
    fn write_line(&mut self, line: &str) -> Result<()> {
        self.ensure_writable()?;
        
        // Handle append mode specially
        if self.mode == IOMode::Append {
            self.cursor.set_position(self.cursor.get_ref().len() as u64);
        }
        
        self.cursor.write_all(line.as_bytes())?;
        self.cursor.write_all(b"\n")?;
        
        Ok(())
    }
    
    fn flush_output(&mut self) -> Result<()> {
        // No need to flush in-memory buffer
        Ok(())
    }
}

impl SeekableIO for MemoryIO {
    fn seek(&mut self, position: u64) -> Result<u64> {
        if self.closed {
            anyhow::bail!("Cannot seek in closed memory buffer");
        }
        
        self.cursor.set_position(position);
        Ok(position)
    }
    
    fn position(&self) -> Result<u64> {
        if self.closed {
            anyhow::bail!("Cannot get position of closed memory buffer");
        }
        
        Ok(self.cursor.position())
    }
    
    fn size(&self) -> Result<u64> {
        if self.closed {
            anyhow::bail!("Cannot get size of closed memory buffer");
        }
        
        Ok(self.cursor.get_ref().len() as u64)
    }
} 