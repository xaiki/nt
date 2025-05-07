use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom, BufReader, BufWriter, BufRead};
use std::path::{Path, PathBuf};
use std::fmt::Debug;
use anyhow::{Result, Context};

use super::io_trait::{IO, InputIO, OutputIO, SeekableIO, IOCapabilities, IOMode};

// Type alias for the complex return type
type FileIOHandles = (Option<BufReader<File>>, Option<BufWriter<File>>);

/// A file-based IO implementation
pub struct FileIO {
    /// The path to the file
    path: PathBuf,
    /// The mode in which the file is opened
    mode: IOMode,
    /// The capabilities of this implementation
    capabilities: IOCapabilities,
    /// The underlying file reader (if in read mode)
    reader: Option<BufReader<File>>,
    /// The underlying file writer (if in write mode)
    writer: Option<BufWriter<File>>,
    /// Whether the file has been closed
    closed: bool,
}

impl FileIO {
    /// Create a new FileIO instance
    pub fn new(path: &Path, mode: IOMode) -> Result<Self> {
        let file_path = path.to_path_buf();
        
        let mut capabilities = IOCapabilities {
            supports_input: matches!(mode, IOMode::Read | IOMode::ReadWrite),
            supports_output: matches!(mode, IOMode::Write | IOMode::Append | IOMode::ReadWrite),
            supports_sync: true,
            supports_async: false,
            supports_seek: true,
            supports_filtering: false,
        };
        
        let (reader, writer) = Self::open_file(path, mode, &mut capabilities)?;
        
        Ok(Self {
            path: file_path,
            mode,
            capabilities,
            reader,
            writer,
            closed: false,
        })
    }
    
    /// Open the file in the specified mode
    fn open_file(path: &Path, mode: IOMode, _capabilities: &mut IOCapabilities) -> Result<FileIOHandles> {
        match mode {
            IOMode::Read => {
                let file = OpenOptions::new()
                    .read(true)
                    .open(path)
                    .with_context(|| format!("Failed to open file for reading: {}", path.display()))?;
                    
                Ok((Some(BufReader::new(file)), None))
            },
            IOMode::Write => {
                let file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)
                    .with_context(|| format!("Failed to open file for writing: {}", path.display()))?;
                    
                Ok((None, Some(BufWriter::new(file))))
            },
            IOMode::Append => {
                let file = OpenOptions::new()
                    .read(false)
                    .append(true)
                    .create(true)
                    .open(path)
                    .with_context(|| format!("Failed to open file for appending: {}", path.display()))?;
                    
                Ok((None, Some(BufWriter::new(file))))
            },
            IOMode::ReadWrite => {
                let file = OpenOptions::new()
                    .read(false)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)
                    .with_context(|| format!("Failed to open file for reading and writing: {}", path.display()))?;
                
                // For read-write mode, we need a separate file handle for reading and writing
                let read_file = OpenOptions::new()
                    .read(true)
                    .open(path)
                    .with_context(|| format!("Failed to open file for reading: {}", path.display()))?;
                
                Ok((Some(BufReader::new(read_file)), Some(BufWriter::new(file))))
            }
        }
    }
    
    /// Ensure the file is open for reading
    fn ensure_readable(&self) -> Result<()> {
        if self.closed {
            anyhow::bail!("Cannot read from closed file: {}", self.path.display());
        }
        
        if !self.capabilities.supports_input {
            anyhow::bail!("File not opened for reading: {}", self.path.display());
        }
        
        if self.reader.is_none() {
            anyhow::bail!("No reader available for file: {}", self.path.display());
        }
        
        Ok(())
    }
    
    /// Ensure the file is open for writing
    fn ensure_writable(&self) -> Result<()> {
        if self.closed {
            anyhow::bail!("Cannot write to closed file: {}", self.path.display());
        }
        
        if !self.capabilities.supports_output {
            anyhow::bail!("File not opened for writing: {}", self.path.display());
        }
        
        if self.writer.is_none() {
            anyhow::bail!("No writer available for file: {}", self.path.display());
        }
        
        Ok(())
    }
}

impl Debug for FileIO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileIO")
            .field("path", &self.path)
            .field("mode", &self.mode)
            .field("capabilities", &self.capabilities)
            .field("closed", &self.closed)
            .finish()
    }
}

impl IO for FileIO {
    fn name(&self) -> &str {
        "file_io"
    }
    
    fn capabilities(&self) -> IOCapabilities {
        self.capabilities.clone()
    }
    
    fn is_ready(&self) -> bool {
        !self.closed
    }
    
    fn close(&mut self) -> Result<()> {
        if !self.closed {
            // Ensure writers are flushed before closing
            if let Some(ref mut writer) = self.writer {
                writer.flush().with_context(|| format!("Failed to flush file: {}", self.path.display()))?;
            }
            
            // Clear readers and writers
            self.reader = None;
            self.writer = None;
            
            self.closed = true;
        }
        
        Ok(())
    }
}

impl Read for FileIO {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.ensure_readable().is_err() {
            return Err(io::Error::other("Cannot read from file"));
        }
        
        if let Some(ref mut reader) = self.reader {
            reader.read(buf)
        } else {
            Err(io::Error::other("No reader available"))
        }
    }
}

impl Write for FileIO {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.ensure_writable().is_err() {
            return Err(io::Error::other("Cannot write to file"));
        }
        
        if let Some(ref mut writer) = self.writer {
            writer.write(buf)
        } else {
            Err(io::Error::other("No writer available"))
        }
    }
    
    fn flush(&mut self) -> io::Result<()> {
        if self.ensure_writable().is_err() {
            return Err(io::Error::other("Cannot flush file"));
        }
        
        if let Some(ref mut writer) = self.writer {
            writer.flush()
        } else {
            Err(io::Error::other("No writer available"))
        }
    }
}

impl InputIO for FileIO {
    fn read_line(&mut self) -> Result<String> {
        self.ensure_readable()?;
        
        let mut line = String::new();
        if let Some(ref mut reader) = self.reader {
            reader.read_line(&mut line)?;
        }
        
        // Trim newline characters from the end
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        
        Ok(line)
    }
    
    fn has_data_available(&self) -> bool {
        if self.ensure_readable().is_err() {
            return false;
        }
        
        // For files, we don't have a good way to check without reading
        // We'll assume there's data if the file is open
        true
    }
}

impl OutputIO for FileIO {
    fn write_line(&mut self, line: &str) -> Result<()> {
        self.ensure_writable()?;
        
        if let Some(ref mut writer) = self.writer {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }
        
        Ok(())
    }
    
    fn flush_output(&mut self) -> Result<()> {
        self.ensure_writable()?;
        
        if let Some(ref mut writer) = self.writer {
            writer.flush()?;
        }
        
        Ok(())
    }
}

impl SeekableIO for FileIO {
    fn seek(&mut self, position: u64) -> Result<u64> {
        if self.closed {
            anyhow::bail!("Cannot seek in closed file: {}", self.path.display());
        }
        
        // If we have a writer, flush it before seeking
        if let Some(ref mut writer) = self.writer {
            writer.flush()?;
        }
        
        // Seek in both reader and writer if available
        let mut final_pos = 0;
        
        if let Some(ref mut reader) = self.reader {
            let read_file = reader.get_mut();
            final_pos = read_file.seek(SeekFrom::Start(position))?;
        }
        
        if let Some(ref mut writer) = self.writer {
            let write_file = writer.get_mut();
            final_pos = write_file.seek(SeekFrom::Start(position))?;
        }
        
        Ok(final_pos)
    }
    
    fn position(&self) -> Result<u64> {
        if self.closed {
            anyhow::bail!("Cannot get position of closed file: {}", self.path.display());
        }
        
        // We can't get the position without seeking, which would require a mutable reference
        // In a real implementation, we'd track the position internally
        // For now, this is a limitation
        anyhow::bail!("Getting position without seeking is not implemented for FileIO")
    }
    
    fn size(&self) -> Result<u64> {
        if self.closed {
            anyhow::bail!("Cannot get size of closed file: {}", self.path.display());
        }
        
        let metadata = std::fs::metadata(&self.path)
            .with_context(|| format!("Failed to get metadata for file: {}", self.path.display()))?;
            
        Ok(metadata.len())
    }
} 