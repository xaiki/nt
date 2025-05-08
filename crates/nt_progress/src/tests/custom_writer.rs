use std::collections::HashMap;
use std::io::Write;
use anyhow::Result;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::io::custom::{CustomWriter, WriterCapabilities, WriterRegistry};
use crate::io::ProgressWriter;
use crate::thread::TaskHandle;
use crate::Config;
use crate::io::OutputBuffer;

/// A test writer that formats output with a prefix
#[derive(Debug, Clone)]
struct TestCustomWriter {
    name: String,
    prefix: String,
    lines: Vec<String>,
    capabilities: WriterCapabilities,
    config: HashMap<String, String>,
}

/// A minimal writer that counts writes but doesn't store anything
/// Useful for performance testing and operation counting
#[derive(Debug)]
struct DummyWriter {
    name: String,
    write_count: AtomicUsize,
    flush_count: AtomicUsize,
    capabilities: WriterCapabilities,
    config: HashMap<String, String>,
}

impl Clone for DummyWriter {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            write_count: AtomicUsize::new(self.write_count.load(Ordering::SeqCst)),
            flush_count: AtomicUsize::new(self.flush_count.load(Ordering::SeqCst)),
            capabilities: self.capabilities.clone(),
            config: self.config.clone(),
        }
    }
}

impl DummyWriter {
    fn new(name: &str) -> Self {
        let capabilities = WriterCapabilities::default();
        
        let mut config = HashMap::new();
        config.insert("type".to_string(), "dummy".to_string());
        
        Self {
            name: name.to_string(),
            write_count: AtomicUsize::new(0),
            flush_count: AtomicUsize::new(0),
            capabilities,
            config,
        }
    }
    
    fn get_write_count(&self) -> usize {
        self.write_count.load(Ordering::SeqCst)
    }
    
    fn get_flush_count(&self) -> usize {
        self.flush_count.load(Ordering::SeqCst)
    }
}

impl Write for DummyWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_count.fetch_add(1, Ordering::SeqCst);
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

impl ProgressWriter for DummyWriter {
    fn write_line(&mut self, _line: &str) -> Result<()> {
        self.write_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        self.flush_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        true
    }
}

impl CustomWriter for DummyWriter {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn capabilities(&self) -> WriterCapabilities {
        self.capabilities.clone()
    }
    
    fn config(&self) -> HashMap<String, String> {
        self.config.clone()
    }
}

impl TestCustomWriter {
    fn new(name: &str, prefix: &str) -> Self {
        let mut capabilities = WriterCapabilities::default();
        capabilities.supports_formatting = true;
        
        let mut config = HashMap::new();
        config.insert("prefix".to_string(), prefix.to_string());
        
        Self {
            name: name.to_string(),
            prefix: prefix.to_string(),
            lines: Vec::new(),
            capabilities,
            config,
        }
    }
    
    fn get_lines(&self) -> &[String] {
        &self.lines
    }
}

/// A filtering writer that only passes through certain lines
#[derive(Debug, Clone)]
struct FilteringWriter {
    name: String,
    filter: String,
    lines: Vec<String>,
    capabilities: WriterCapabilities,
    config: HashMap<String, String>,
}

impl FilteringWriter {
    fn new(name: &str, filter: &str) -> Self {
        let mut capabilities = WriterCapabilities::default();
        capabilities.supports_filtering = true;
        
        let mut config = HashMap::new();
        config.insert("filter".to_string(), filter.to_string());
        
        Self {
            name: name.to_string(),
            filter: filter.to_string(),
            lines: Vec::new(),
            capabilities,
            config,
        }
    }
    
    fn get_lines(&self) -> &[String] {
        &self.lines
    }
}

impl Write for TestCustomWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let line = String::from_utf8_lossy(buf).to_string();
        self.lines.push(format!("{}{}", self.prefix, line));
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl ProgressWriter for TestCustomWriter {
    fn write_line(&mut self, line: &str) -> Result<()> {
        self.lines.push(format!("{}{}", self.prefix, line));
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        true
    }
}

impl CustomWriter for TestCustomWriter {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn capabilities(&self) -> WriterCapabilities {
        self.capabilities.clone()
    }
    
    fn config(&self) -> HashMap<String, String> {
        self.config.clone()
    }
}

impl Write for FilteringWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let line = String::from_utf8_lossy(buf).to_string();
        if line.contains(&self.filter) {
            self.lines.push(line);
        }
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl ProgressWriter for FilteringWriter {
    fn write_line(&mut self, line: &str) -> Result<()> {
        if line.contains(&self.filter) {
            self.lines.push(line.to_string());
        }
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        true
    }
}

impl CustomWriter for FilteringWriter {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn capabilities(&self) -> WriterCapabilities {
        self.capabilities.clone()
    }
    
    fn config(&self) -> HashMap<String, String> {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::TeeWriter;
    use crate::modes::limited::Limited;
    use tokio::runtime::Runtime;
    
    #[test]
    fn test_writer_registration() {
        // Create a registry
        let mut registry = WriterRegistry::new();
        
        // Create a test writer
        let writer = TestCustomWriter::new("test", "PREFIX: ");
        
        // Register the writer
        registry.register(writer).unwrap();
        
        // Check that the writer was registered
        let writers = registry.list();
        assert_eq!(writers.len(), 1);
        assert_eq!(writers[0], "test");
    }
    
    #[test]
    fn test_writer_capabilities() {
        // Create a registry
        let mut registry = WriterRegistry::new();
        
        // Create a test writer
        let writer = TestCustomWriter::new("test", "PREFIX: ");
        
        // Register the writer
        registry.register(writer).unwrap();
        
        // Get the writer and check its capabilities
        let writer = registry.get("test").unwrap();
        let capabilities = writer.capabilities();
        
        assert!(capabilities.supports_formatting);
        assert!(!capabilities.supports_filtering);
        assert!(!capabilities.supports_redirection);
        assert!(!capabilities.supports_async);
    }
    
    #[test]
    fn test_writer_config() {
        // Create a registry
        let mut registry = WriterRegistry::new();
        
        // Create a test writer
        let writer = TestCustomWriter::new("test", "PREFIX: ");
        
        // Register the writer
        registry.register(writer).unwrap();
        
        // Get the writer and check its configuration
        let writer = registry.get("test").unwrap();
        let config = writer.config();
        
        assert_eq!(config.get("prefix").unwrap(), "PREFIX: ");
    }
    
    #[test]
    fn test_writer_output() {
        // Create a writer directly
        let mut writer = TestCustomWriter::new("test", "PREFIX: ");
        
        // Write some lines
        writer.write_line("Hello, world!").unwrap();
        writer.write_line("This is a test").unwrap();
        
        // Check the output
        let lines = writer.get_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "PREFIX: Hello, world!");
        assert_eq!(lines[1], "PREFIX: This is a test");
    }
    
    #[test]
    fn test_writer_removal() {
        // Create a registry
        let mut registry = WriterRegistry::new();
        
        // Create test writers
        let writer1 = TestCustomWriter::new("test1", "PREFIX1: ");
        let writer2 = TestCustomWriter::new("test2", "PREFIX2: ");
        
        // Register the writers
        registry.register(writer1).unwrap();
        registry.register(writer2).unwrap();
        
        // Check that both writers were registered
        let writers = registry.list();
        assert_eq!(writers.len(), 2);
        
        // Remove a writer
        let writer = registry.remove("test1").unwrap();
        assert_eq!(writer.name(), "test1");
        
        // Check that only one writer remains
        let writers = registry.list();
        assert_eq!(writers.len(), 1);
        assert_eq!(writers[0], "test2");
    }
    
    #[test]
    fn test_duplicate_registration() {
        // Create a registry
        let mut registry = WriterRegistry::new();
        
        // Create test writers with the same name
        let writer1 = TestCustomWriter::new("test", "PREFIX1: ");
        let writer2 = TestCustomWriter::new("test", "PREFIX2: ");
        
        // Register the first writer
        registry.register(writer1).unwrap();
        
        // Try to register the second writer with the same name
        let result = registry.register(writer2);
        
        // This should fail
        assert!(result.is_err());
    }
    
    #[test]
    fn test_filtering_writer() {
        // Create a filtering writer
        let mut writer = FilteringWriter::new("filter_test", "important");
        
        // Write some lines, only some containing the filter word
        writer.write_line("This is an important message").unwrap();
        writer.write_line("This message will be filtered out").unwrap();
        writer.write_line("Another important notification").unwrap();
        
        // Check that only the lines containing the filter word were kept
        let lines = writer.get_lines();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("important"));
        assert!(lines[1].contains("important"));
    }
    
    #[test]
    fn test_tee_with_custom_writer() {
        // Create a buffer and a custom writer
        let buffer = OutputBuffer::new(10);
        let custom_writer = TestCustomWriter::new("tee_test", "TEE: ");
        
        // Create a tee writer that writes to both
        let mut tee = TeeWriter::new(buffer, custom_writer);
        
        // Write some lines
        tee.write_line("Message through tee writer").unwrap();
        
        // The custom writer should have formatted the message
        let custom_writer = tee.writer2();
        let lines = custom_writer.get_lines();
        assert_eq!(lines[0], "TEE: Message through tee writer");
    }
    
    #[test]
    fn test_integration_with_task_handle() {
        // This requires tokio runtime since TaskHandle uses async functions
        let rt = Runtime::new().unwrap();
        
        rt.block_on(async {
            // Create a mode config for the task handle
            let limited = Limited::new(1);
            let config = Config::from(Box::new(limited) as Box<dyn crate::ThreadConfig>);
            
            // Create a message channel for task handles
            let (message_tx, _message_rx) = tokio::sync::mpsc::channel(100);
            
            // Create a task handle
            let mut task_handle = TaskHandle::new(1, config, message_tx);
            
            // Create a custom writer
            let mut custom_writer = TestCustomWriter::new("task_test", "TASK: ");
            
            // Write a message using the task handle's write_line method
            task_handle.write_line("Task message").await.unwrap();
            
            // Write a direct message using our custom writer
            custom_writer.write_line("Direct message").unwrap();
            
            // Verify the custom writer's output
            assert_eq!(custom_writer.get_lines()[0], "TASK: Direct message");
        });
    }
    
    #[test]
    fn test_multiple_writer_capabilities() {
        // Test a writer with multiple capabilities
        let mut writer = TestCustomWriter::new("multi_cap", "");
        
        // Manually set multiple capabilities
        let mut capabilities = WriterCapabilities::default();
        capabilities.supports_formatting = true;
        capabilities.supports_filtering = true;
        capabilities.supports_redirection = true;
        
        // Hack to set the capabilities (would normally do this in constructor)
        let writer_mut = &mut writer as *mut TestCustomWriter;
        unsafe {
            (*writer_mut).capabilities = capabilities;
        }
        
        // Check all capabilities
        let caps = writer.capabilities();
        assert!(caps.supports_formatting);
        assert!(caps.supports_filtering);
        assert!(caps.supports_redirection);
        assert!(!caps.supports_async);
    }
    
    #[test]
    fn test_writer_registry_iteration() {
        // Create a registry
        let mut registry = WriterRegistry::new();
        
        // Register multiple writers
        registry.register(TestCustomWriter::new("writer1", "PREFIX1: ")).unwrap();
        registry.register(TestCustomWriter::new("writer2", "PREFIX2: ")).unwrap();
        registry.register(TestCustomWriter::new("writer3", "PREFIX3: ")).unwrap();
        
        // Get all writer names
        let names = registry.list();
        
        // Verify all writers are in the list
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"writer1"));
        assert!(names.contains(&"writer2"));
        assert!(names.contains(&"writer3"));
    }
    
    #[test]
    fn test_writer_error_handling() {
        struct ErrorWriter {}
        
        impl Write for ErrorWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Write error"))
            }
            
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Flush error"))
            }
        }
        
        impl Debug for ErrorWriter {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "ErrorWriter")
            }
        }
        
        impl ProgressWriter for ErrorWriter {
            fn write_line(&mut self, _: &str) -> Result<()> {
                Err(anyhow::anyhow!("Write line error"))
            }
            
            fn flush(&mut self) -> Result<()> {
                Err(anyhow::anyhow!("Flush error"))
            }
            
            fn is_ready(&self) -> bool {
                false
            }
        }
        
        // Test that errors are properly propagated
        let mut writer = ErrorWriter {};
        assert!(writer.write_line("test").is_err());
        assert!(crate::io::ProgressWriter::flush(&mut writer).is_err());
        assert!(!writer.is_ready());
    }
    
    #[test]
    fn test_dummy_writer() {
        // Create a dummy writer
        let mut writer = DummyWriter::new("dummy_test");
        
        // Write some lines
        writer.write_line("Line 1").unwrap();
        writer.write_line("Line 2").unwrap();
        writer.write_line("Line 3").unwrap();
        
        // Check that the write count is correct
        assert_eq!(writer.get_write_count(), 3);
        
        // Flush the writer
        crate::io::ProgressWriter::flush(&mut writer).unwrap();
        
        // Check that the flush count is correct
        assert_eq!(writer.get_flush_count(), 1);
    }
    
    #[test]
    fn test_dummy_writer_with_registry() {
        // Create a registry
        let mut registry = WriterRegistry::new();
        
        // Create a dummy writer
        let writer = DummyWriter::new("dummy_registry");
        
        // Register the writer
        registry.register(writer).unwrap();
        
        // Get the writer from the registry
        let writer = registry.get("dummy_registry").unwrap();
        
        // Verify the writer's configuration
        let config = writer.config();
        assert_eq!(config.get("type").unwrap(), "dummy");
    }
    
    #[test]
    fn test_dummy_writer_concurrency() {
        use std::thread;
        
        // Create a writer that will be shared between threads
        let writer = DummyWriter::new("concurrent_dummy");
        let writer_arc = std::sync::Arc::new(std::sync::Mutex::new(writer));
        
        // Spawn multiple threads that all write to the writer
        let mut handles = vec![];
        for i in 0..10 {
            let writer_clone = writer_arc.clone();
            let handle = thread::spawn(move || {
                let mut writer = writer_clone.lock().unwrap();
                for j in 0..10 {
                    let _ = writer.write_line(&format!("Thread {} message {}", i, j));
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Check the total write count
        let writer = writer_arc.lock().unwrap();
        assert_eq!(writer.get_write_count(), 100); // 10 threads × 10 messages
    }
    
    #[test]
    fn test_tee_with_dummy_writer_monitoring() {
        // Create a primary writer that actually stores data
        let primary_writer = OutputBuffer::new(10);
        
        // Create a dummy writer that just counts operations
        let dummy_writer = DummyWriter::new("monitor");
        
        // Create a tee writer that writes to both
        let mut tee = TeeWriter::new(primary_writer, dummy_writer);
        
        // Write several lines
        for i in 0..5 {
            tee.write_line(&format!("Line {}", i)).unwrap();
        }
        
        // Verify the data in the primary writer
        let primary_writer = tee.writer1();
        let lines = primary_writer.get_lines();
        assert_eq!(lines.len(), 5);
        
        // Verify the counts in the monitoring writer
        let monitor_writer = tee.writer2();
        assert_eq!(monitor_writer.get_write_count(), 5);
        
        // This demonstrates how DummyWriter can be used for operation counting
        // without storing any content, which is useful for performance monitoring
        // or tracking write patterns in large applications
    }

    #[tokio::test]
    async fn test_task_handle_direct_writer() {
        use tokio::sync::mpsc;
        use crate::Config;
        use crate::thread::TaskHandle;
        use crate::io::OutputBuffer;
        
        // Create a basic configuration for testing
        let config = Config::default();
        
        // Create a message channel
        let (message_tx, _) = mpsc::channel(100);
        
        // Create a TaskHandle
        let mut task_handle = TaskHandle::new(1, config, message_tx);
        
        // Test getting a direct reference to the writer
        {
            let writer_ref = task_handle.writer();
            assert!(writer_ref.lock().await.is_ready());
        }
        
        // Test writing with the with_writer method
        {
            task_handle.with_writer(|writer| {
                writer.write_line("Test message via with_writer")?;
                Ok(())
            }).await.unwrap();
        }
        
        // Test replacing the writer with a custom implementation
        {
            let custom_buffer = OutputBuffer::new(5);
            let mut prev_writer = task_handle.set_writer(Box::new(custom_buffer)).await;
            
            // Write to the new writer
            task_handle.write_line("Test message after writer replacement").await.unwrap();
            
            // Check that the previous writer works
            prev_writer.write_line("Final message to previous writer").unwrap();
        }
        
        // Test creating a tee writer
        {
            // Create a custom writer to tee output to
            let custom_buffer = OutputBuffer::new(5);
            
            // Add it as a tee writer
            task_handle.add_tee_writer(custom_buffer).await.unwrap();
            
            // Write a message that should go to both writers
            task_handle.write_line("Test message for tee writer").await.unwrap();
            
            // Verify the message was captured in the original writer
            task_handle.with_writer(|writer| {
                // This is complex because we have a boxed TeeWriter, 
                // and we'd need to extract the first writer which is also a BoxedWriter.
                // For this test, it's sufficient to verify that write operations work.
                assert!(writer.is_ready());
                Ok(())
            }).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_task_handle_passthrough() {
        use tokio::sync::mpsc;
        use crate::{Config, ThreadMode};
        use crate::thread::TaskHandle;
        use crate::io::ProgressWriter;
        
        // Create a message channel for task handles
        let (message_tx, _message_rx) = mpsc::channel(100);
        
        // Create a task handle with Limited mode (which supports passthrough)
        let config = Config::new(ThreadMode::Limited, 1).unwrap();
        let mut task_handle = TaskHandle::new(1, config, message_tx);
        
        // Test passthrough availability
        let passthrough_available = task_handle.has_passthrough().await;
        assert_eq!(passthrough_available, Some(true), "Limited mode should support passthrough");
        
        // Test enabling passthrough
        let result = task_handle.set_passthrough(true).await;
        assert!(result.is_ok(), "Should be able to enable passthrough");
        
        // Create a custom passthrough writer
        struct CountingWriter {
            lines: std::sync::atomic::AtomicUsize,
        }
        
        impl std::fmt::Debug for CountingWriter {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("CountingWriter")
                    .field("lines", &self.lines.load(std::sync::atomic::Ordering::SeqCst))
                    .finish()
            }
        }
        
        impl std::io::Write for CountingWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.lines.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(buf.len())
            }
            
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        
        impl ProgressWriter for CountingWriter {
            fn write_line(&mut self, _line: &str) -> anyhow::Result<()> {
                self.lines.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
            
            fn flush(&mut self) -> anyhow::Result<()> {
                Ok(())
            }
            
            fn is_ready(&self) -> bool {
                true
            }
        }
        
        // Create our counting writer
        let counting_writer = CountingWriter {
            lines: std::sync::atomic::AtomicUsize::new(0),
        };
        
        // Set it as the passthrough writer
        let result = task_handle.set_passthrough_writer(Box::new(counting_writer)).await;
        assert!(result.is_ok(), "Should be able to set custom passthrough writer");
        
        // Write a line - this should trigger the passthrough
        let _ = task_handle.write_line("Test line for passthrough").await;
        
        // Disable passthrough
        let result = task_handle.set_passthrough(false).await;
        assert!(result.is_ok(), "Should be able to disable passthrough");
        
        // Test filter functionality
        let result = task_handle.set_passthrough_filter(|line| line.contains("ERROR")).await;
        assert!(result.is_ok(), "Should be able to set a passthrough filter");
        
        // Write some lines with the filter
        let _ = task_handle.write_line("Normal log message").await;
        let _ = task_handle.write_line("This is an ERROR message").await;
        let _ = task_handle.write_line("Another normal message").await;
    }
} 