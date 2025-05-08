use crate::ProgressDisplay;
use crate::ThreadMode;
use crate::terminal::TestEnv;
use tokio::time::sleep;
use std::time::Duration;
use crate::tests::common::with_timeout;
use anyhow::Result;

#[tokio::test]
async fn test_limited_basic() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::Limited).await?;
    let mut env = TestEnv::new_with_size(80, 24);

    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Limited, || "limited-test").await?;
        task.capture_stdout("Test message".to_string()).await?;
        display.display().await?;
        env.writeln("Test message");
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 60).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_limited_concurrent() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let total_jobs = 5;

    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Spawn multiple tasks in Limited mode
        let mut handles = vec![];
        for i in 0..total_jobs {
            let display_ref = display.clone();
            let mut env = TestEnv::new_with_size(80, 24);
            handles.push(tokio::spawn(async move {
                let mut task = display_ref.spawn_with_mode(ThreadMode::Limited, move || format!("task-{}", i)).await?;
                for j in 0..3 {
                    task.capture_stdout(format!("Thread {}: Message {}", i, j)).await?;
                    env.writeln(&format!("Thread {}: Message {}", i, j));
                    sleep(Duration::from_millis(50)).await;
                }
                Ok::<TestEnv, anyhow::Error>(env)
            }));
        }
        
        // Wait for all tasks to complete and combine their outputs
        let mut final_env = TestEnv::new_with_size(80, 24);
        for handle in handles {
            let task_env = handle.await??;
            let content = task_env.contents();
            if !content.is_empty() {
                final_env.write(&content);
            }
        }
        
        // Verify final state
        display.display().await?;
        final_env.verify();
        Ok::<(), anyhow::Error>(())
    }, 60).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
} 