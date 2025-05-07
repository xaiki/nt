use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::terminal::TestEnv;
use tokio::time::sleep;
use std::time::Duration;
use crate::tests::common::with_timeout;
use anyhow::Result;

#[tokio::test]
async fn test_window_basic() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new_with_size(80, 24);
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Window(3), || "window-test").await?;
        task.capture_stdout("Test message".to_string()).await?;
        display.display().await?;
        env.writeln("Test message");
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 5).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_scroll() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::Window(3)).await?;
    let mut env = TestEnv::new_with_size(80, 24);
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Window(3), || "scroll-test").await?;
        for i in 0..5 {
            let message = format!("Line {}", i);
            task.capture_stdout(message.clone()).await?;
            env.writeln(&message);
        }
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 5).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_size() -> Result<()> {
    for size in [1, 3, 5, 10] {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new_with_mode(ThreadMode::Window(size)).await?;
        let mut env = TestEnv::new_with_size(80, 24);
        
        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            let mut task = display.spawn_with_mode(ThreadMode::Window(size), move || format!("size-{}", size)).await?;
            for i in 0..size + 2 {
                let message = format!("Line {}", i);
                task.capture_stdout(message.clone()).await?;
                env.writeln(&message);
            }
            
            display.display().await?;
            env.verify();
            Ok::<(), anyhow::Error>(())
        }, 5).await?;

        // Clean up OUTSIDE timeout
        display.stop().await?;
    }
    Ok(())
}

#[tokio::test]
async fn test_window_concurrent() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let total_jobs = 5;
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut handles = vec![];
        
        // Spawn multiple tasks
        for i in 0..total_jobs {
            let display_ref = display.clone();
            let mut env = TestEnv::new_with_size(80, 24);
            let i = i;
            handles.push(tokio::spawn(async move {
                let mut task = display_ref.spawn_with_mode(ThreadMode::Window(3), move || format!("task-{}", i)).await?;
                for j in 0..5 {
                    let message = format!("Thread {}: Message {}", i, j);
                    task.capture_stdout(message.clone()).await?;
                    env.writeln(&message);
                    sleep(Duration::from_millis(50)).await;
                }
                Ok::<TestEnv, anyhow::Error>(env)
            }));
        }
        
        // Wait for all tasks to complete and merge their outputs
        let mut final_env = TestEnv::new_with_size(80, 24);
        for handle in handles {
            let task_env = handle.await??;
            final_env.merge(task_env);
        }
        
        display.display().await?;
        final_env.verify();
        Ok::<(), anyhow::Error>(())
    }, 5).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_edge_case_minimal() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new_with_size(80, 24);
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Window(1), || "minimal-window").await?;
        task.capture_stdout("Minimal window test".to_string()).await?;
        env.writeln("Minimal window test");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 5).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_edge_case_large() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::Window(30)).await?;
    let mut env = TestEnv::new_with_size(80, 24);
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Window(30), || "large-window").await?;
        for i in 0..35 {
            let message = format!("Line {}", i);
            task.capture_stdout(message.clone()).await?;
            env.writeln(&message);
        }
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 5).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
} 