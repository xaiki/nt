use crate::io::{ProgressWriter, OutputBuffer, TeeWriter};
use std::io::Write;
use anyhow::Result;

#[test]
fn test_output_buffer() -> Result<()> {
    let mut buffer = OutputBuffer::new(3);
    
    // Test writing lines
    buffer.write_line("line 1")?;
    buffer.write_line("line 2")?;
    buffer.write_line("line 3")?;
    buffer.write_line("line 4")?;
    
    // Verify only last 3 lines are kept
    let lines = buffer.get_lines();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line 2");
    assert_eq!(lines[1], "line 3");
    assert_eq!(lines[2], "line 4");
    
    // Test clearing
    buffer.clear();
    assert_eq!(buffer.get_lines().len(), 0);
    
    Ok(())
}

#[test]
fn test_tee_writer() -> Result<()> {
    let buffer1 = OutputBuffer::new(3);
    let buffer2 = OutputBuffer::new(3);
    let mut tee = TeeWriter::new(buffer1, buffer2);
    
    // Test writing lines
    tee.write_line("test line")?;
    
    // Verify both buffers received the line
    assert_eq!(tee.writer1().get_lines()[0], "test line");
    assert_eq!(tee.writer2().get_lines()[0], "test line");
    
    // Test Write trait implementation
    tee.write_all(b"raw bytes")?;
    assert_eq!(tee.writer1().get_lines()[1], "raw bytes");
    assert_eq!(tee.writer2().get_lines()[1], "raw bytes");
    
    Ok(())
}

#[test]
fn test_writer_ready_state() -> Result<()> {
    let buffer = OutputBuffer::new(3);
    assert!(buffer.is_ready());
    
    let tee = TeeWriter::new(
        OutputBuffer::new(3),
        OutputBuffer::new(3)
    );
    assert!(tee.is_ready());
    
    Ok(())
} 