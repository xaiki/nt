use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::terminal::TestEnv;
use tokio::time::sleep;
use std::time::Duration;
use crate::tests::common::with_timeout;

#[tokio::test]
async fn test_capturing_basic() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await.unwrap();
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        let _handle = display.spawn_with_mode(ThreadMode::Capturing, || "capturing-test").await.unwrap();
        env.writeln("Test message");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 5).await.unwrap();
}

#[tokio::test]
async fn test_capturing_multi_line() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await.unwrap();
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        let _handle = display.spawn_with_mode(ThreadMode::Capturing, || "multi-line-test").await.unwrap();
        for i in 0..5 {
            env.writeln(&format!("Line {}", i));
        }
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 5).await.unwrap();
}

#[tokio::test]
async fn test_capturing_concurrent() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await.unwrap();
    
    // Run test within timeout
    with_timeout(async {
        let total_jobs = 5;
        let mut handles = vec![];
        
        // Spawn multiple tasks
        for i in 0..total_jobs {
            let display = display.clone();
            let mut env = TestEnv::new(80, 24);
            let i = i;
            handles.push(tokio::spawn(async move {
                display.spawn_with_mode(ThreadMode::Capturing, move || format!("task-{}", i)).await.unwrap();
                for j in 0..5 {
                    env.writeln(&format!("Thread {}: Message {}", i, j));
                    sleep(Duration::from_millis(50)).await;
                }
                env
            }));
        }
        
        // Wait for all tasks to complete and merge their outputs
        let mut final_env = TestEnv::new(80, 24);
        for handle in handles {
            let task_env = handle.await.unwrap();
            final_env.merge(task_env);
        }
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        final_env.verify();
    }, 5).await.unwrap();
}

#[tokio::test]
async fn test_capturing_error_handling() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await.unwrap();
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        let _handle = display.spawn_with_mode(ThreadMode::Capturing, || "error-test").await.unwrap();
        env.writeln("Error handling test");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 5).await.unwrap();
}

#[tokio::test]
async fn test_capturing_special_chars() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await.unwrap();
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        let _handle = display.spawn_with_mode(ThreadMode::Capturing, || "special-chars-test").await.unwrap();
        env.writeln("Special chars test");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 5).await.unwrap();
}

#[tokio::test]
async fn test_capturing_long_lines() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await.unwrap();
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        let _handle = display.spawn_with_mode(ThreadMode::Capturing, || "long-lines-test").await.unwrap();
        env.writeln(&"x".repeat(100));
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 5).await.unwrap();
} 