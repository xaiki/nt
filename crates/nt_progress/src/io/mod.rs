use std::io::{self, Write};
use std::fmt::Debug;
use anyhow::Result;

pub mod custom;
pub mod io_trait;
pub mod file_io;
pub mod memory_io;
pub mod default_factory;
pub mod network_io;

// Re-export important types from io_trait
pub use io_trait::{IO, InputIO, OutputIO, SeekableIO, FilterableIO, IOFactory, IOCapabilities, IOMode};
// Re-export FileIO and MemoryIO
pub use file_io::FileIO;
pub use memory_io::MemoryIO;

// Re-export DefaultIOFactory as the default in-memory IO factory
pub use default_factory::DefaultIOFactory;

// Re-export NetworkIO stub for network I/O
pub use network_io::NetworkIO;

/// A trait for writers that can handle both synchronous and asynchronous writes
pub trait ProgressWriter: Write + Send + Sync + Debug {
    /// Write a line of text
    fn write_line(&mut self, line: &str) -> Result<()>;
    
    /// Flush any buffered content
    fn flush(&mut self) -> Result<()>;
    
    /// Check if the writer is ready to accept more data
    fn is_ready(&self) -> bool;
}

/// A buffer that can be used to store and manage output
pub struct OutputBuffer {
    /// The maximum number of lines to store
    max_lines: usize,
    /// The stored lines
    lines: Vec<String>,
}

impl OutputBuffer {
    /// Create a new output buffer with the specified maximum number of lines
    pub fn new(max_lines: usize) -> Self {
        Self {
            max_lines,
            lines: Vec::with_capacity(max_lines),
        }
    }

    /// Add a line to the buffer
    pub fn add_line(&mut self, line: String) {
        if self.lines.len() >= self.max_lines {
            self.lines.remove(0);
        }
        self.lines.push(line);
    }

    /// Get all lines in the buffer
    pub fn get_lines(&self) -> &[String] {
        &self.lines
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

impl Write for OutputBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let line = String::from_utf8_lossy(buf).to_string();
        self.add_line(line);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl ProgressWriter for OutputBuffer {
    fn write_line(&mut self, line: &str) -> Result<()> {
        self.add_line(line.to_string());
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    fn is_ready(&self) -> bool {
        true
    }
}

impl Debug for OutputBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputBuffer")
            .field("max_lines", &self.max_lines)
            .field("lines", &self.lines)
            .finish()
    }
}

/// A writer that can tee output to multiple destinations
pub struct TeeWriter<W1: ProgressWriter, W2: ProgressWriter> {
    writer1: W1,
    writer2: W2,
}

impl<W1: ProgressWriter, W2: ProgressWriter> TeeWriter<W1, W2> {
    /// Create a new tee writer that writes to two destinations
    pub fn new(writer1: W1, writer2: W2) -> Self {
        Self { writer1, writer2 }
    }

    /// Get a reference to the first writer
    pub fn writer1(&self) -> &W1 {
        &self.writer1
    }

    /// Get a reference to the second writer
    pub fn writer2(&self) -> &W2 {
        &self.writer2
    }
}

impl<W1: ProgressWriter, W2: ProgressWriter> Write for TeeWriter<W1, W2> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer1.write(buf)?;
        self.writer2.write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        io::Write::flush(&mut self.writer1)?;
        io::Write::flush(&mut self.writer2)?;
        Ok(())
    }
}

impl<W1: ProgressWriter, W2: ProgressWriter> ProgressWriter for TeeWriter<W1, W2> {
    fn write_line(&mut self, line: &str) -> Result<()> {
        self.writer1.write_line(line)?;
        self.writer2.write_line(line)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        ProgressWriter::flush(&mut self.writer1)?;
        ProgressWriter::flush(&mut self.writer2)?;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.writer1.is_ready() && self.writer2.is_ready()
    }
}

impl<W1: ProgressWriter, W2: ProgressWriter> Debug for TeeWriter<W1, W2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TeeWriter")
            .field("writer1", &self.writer1)
            .field("writer2", &self.writer2)
            .finish()
    }
}

/// Helper function to create a TeeWriter from boxed writers
pub fn new_tee_writer(
    writer1: Box<dyn ProgressWriter + Send + 'static>,
    writer2: Box<dyn ProgressWriter + Send + 'static>
) -> Box<dyn ProgressWriter + Send + 'static> {
    struct BoxedTeeWriter {
        writer1: Box<dyn ProgressWriter + Send + 'static>,
        writer2: Box<dyn ProgressWriter + Send + 'static>,
    }
    
    impl Write for BoxedTeeWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.writer1.write(buf)?;
            self.writer2.write(buf)?;
            Ok(buf.len())
        }
        
        fn flush(&mut self) -> io::Result<()> {
            self.writer1.flush()?;
            self.writer2.flush()?;
            Ok(())
        }
    }
    
    impl ProgressWriter for BoxedTeeWriter {
        fn write_line(&mut self, line: &str) -> Result<()> {
            self.writer1.write_line(line)?;
            self.writer2.write_line(line)?;
            Ok(())
        }
        
        fn flush(&mut self) -> Result<()> {
            self.writer1.flush()?;
            self.writer2.flush()?;
            Ok(())
        }
        
        fn is_ready(&self) -> bool {
            self.writer1.is_ready() && self.writer2.is_ready()
        }
    }
    
    impl Debug for BoxedTeeWriter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("BoxedTeeWriter")
                .field("writer1", &"Box<dyn ProgressWriter>")
                .field("writer2", &"Box<dyn ProgressWriter>")
                .finish()
        }
    }
    
    Box::new(BoxedTeeWriter {
        writer1,
        writer2,
    })
} 