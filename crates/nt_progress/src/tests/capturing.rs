use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::terminal::TestEnv;
use tokio::time::sleep;
use std::time::Duration;
use crate::tests::common::with_timeout;
use anyhow::Result;

#[tokio::test]
async fn test_capturing_basic() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Capturing, || "capturing-test").await?;
        let message = "Test message";
        task.capture_stdout(message.to_string()).await?;
        env.writeln(message);
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_capturing_multi_line() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Capturing, || "multi-line-test").await?;
        for i in 0..5 {
            let message = format!("Line {}", i);
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
async fn test_capturing_concurrent() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let _env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let total_jobs = 5;
        let mut handles = vec![];
        
        // Spawn multiple tasks
        for i in 0..total_jobs {
            let display_ref = display.clone();
            let mut task_env = TestEnv::new();
            handles.push(tokio::spawn(async move {
                let mut task = display_ref.spawn_with_mode(ThreadMode::Capturing, move || format!("task-{}", i)).await?;
                for j in 0..5 {
                    let message = format!("Thread {}: Message {}", i, j);
                    task.capture_stdout(message.clone()).await?;
                    task_env.writeln(&message);
                    sleep(Duration::from_millis(50)).await;
                }
                Ok::<TestEnv, anyhow::Error>(task_env)
            }));
        }
        
        // Wait for all tasks to complete and merge their outputs
        let mut final_env = TestEnv::new();
        for handle in handles {
            let task_env = handle.await??;
            final_env.merge(task_env);
        }
        
        display.display().await?;
        final_env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_capturing_error_handling() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Capturing, || "error-test").await?;
        let message = "Error handling test";
        task.capture_stdout(message.to_string()).await?;
        env.writeln(message);
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_capturing_special_chars() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Capturing, || "special-chars-test").await?;
        let special_chars = vec![
            "Test with \n newlines \t tabs \r returns",
            "Test with unicode: 你好世界",
            "Test with emoji: 🚀 ✨"
        ];
        
        for chars in special_chars {
            task.capture_stdout(chars.to_string()).await?;
            env.writeln(chars);
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
async fn test_capturing_long_lines() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Capturing, || "long-lines-test").await?;
        let long_line = "x".repeat(100);
        task.capture_stdout(long_line.clone()).await?;
        env.writeln(&long_line);
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
} 