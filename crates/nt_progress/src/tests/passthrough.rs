use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::terminal::TestEnv;
use crate::tests::common::with_timeout;
use crate::io::ProgressWriter;
use anyhow::Result;
use std::io::Write;

#[tokio::test]
async fn test_limited_passthrough_basic() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.create_task(ThreadMode::Limited, 1).await?;
        
        // Enable passthrough
        task.with_type_mut::<crate::modes::Limited, _, _>(|limited| {
            limited.set_passthrough(true);
        }).await;
        
        // Test message handling with passthrough
        task.capture_stdout("Test message".to_string()).await?;
        env.writeln("Test message");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_limited_passthrough_multiple_messages() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.create_task(ThreadMode::Limited, 1).await?;
        
        // Enable passthrough
        task.with_type_mut::<crate::modes::Limited, _, _>(|limited| {
            limited.set_passthrough(true);
        }).await;
        
        // Test multiple messages
        for i in 0..3 {
            let message = format!("Message {}", i);
            task.capture_stdout(message.clone()).await?;
            env.writeln(&message);
        }
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_limited_passthrough_toggle() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.create_task(ThreadMode::Limited, 1).await?;
        
        // Test with passthrough disabled
        task.capture_stdout("Message 1".to_string()).await?;
        // No output expected as passthrough is disabled
        
        // Enable passthrough
        task.with_type_mut::<crate::modes::Limited, _, _>(|limited| {
            limited.set_passthrough(true);
        }).await;
        task.capture_stdout("Message 2".to_string()).await?;
        env.writeln("Message 2");
        
        // Disable passthrough
        task.with_type_mut::<crate::modes::Limited, _, _>(|limited| {
            limited.set_passthrough(false);
        }).await;
        task.capture_stdout("Message 3".to_string()).await?;
        // No output expected as passthrough is disabled
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_limited_passthrough_custom_writer() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.create_task(ThreadMode::Limited, 1).await?;
        
        // Create a custom writer that adds a prefix
        #[derive(Debug)]
        struct PrefixedWriter {
            prefix: String,
        }
        
        impl Write for PrefixedWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                let s = String::from_utf8_lossy(buf);
                println!("{} {}", self.prefix, s);
                Ok(buf.len())
            }
            
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        
        impl ProgressWriter for PrefixedWriter {
            fn write_line(&mut self, line: &str) -> anyhow::Result<()> {
                println!("{} {}", self.prefix, line);
                Ok(())
            }
            
            fn flush(&mut self) -> anyhow::Result<()> {
                Ok(())
            }
            
            fn is_ready(&self) -> bool {
                true
            }
        }
        
        // Set custom writer and enable passthrough
        task.with_type_mut::<crate::modes::Limited, _, _>(|limited| {
            limited.set_passthrough_writer(Box::new(PrefixedWriter {
                prefix: "[CUSTOM]".to_string(),
            })).unwrap();
            limited.set_passthrough(true);
        }).await;
        
        // Test message with custom writer
        task.capture_stdout("Test message".to_string()).await?;
        env.writeln("[CUSTOM] Test message");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_limited_passthrough_error_handling() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.create_task(ThreadMode::Limited, 1).await?;
        
        // Test setting passthrough writer without enabling passthrough
        let result = task.with_type_mut::<crate::modes::Limited, _, _>(|limited| {
            limited.set_passthrough_writer(Box::new(TestEnv::new()))
        }).await;
        assert!(result.is_some(), "Should be able to set writer even if passthrough is disabled");
        
        // Test setting invalid writer
        let result = task.with_type_mut::<crate::modes::Limited, _, _>(|limited| {
            limited.set_passthrough_writer(Box::new(TestEnv::new()))
        }).await;
        assert!(result.is_some(), "Should be able to set writer multiple times");
        
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
} 