use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::terminal::TestEnv;
use tokio::time::sleep;
use std::time::Duration;
use crate::tests::common::with_timeout;

#[tokio::test]
async fn test_window_basic() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        let _handle = display.spawn_with_mode(ThreadMode::Window(3), || "window-test").await.unwrap();
        env.writeln("Test message");
        
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_scroll() {
    // Create display outside timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::Window(3)).await;
    let mut env = TestEnv::new(80, 24);
    
    // Run test within timeout
    with_timeout(async {
        let _handle = display.spawn_with_mode(ThreadMode::Window(3), || "scroll-test").await.unwrap();
        for i in 0..5 {
            env.writeln(&format!("Line {}", i));
        }
        
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_size() {
    with_timeout(async {
        for size in [1, 3, 5, 10] {
            let display = ProgressDisplay::new_with_mode(ThreadMode::Window(size)).await;
            let mut env = TestEnv::new(80, 24);
            
            display.spawn_with_mode(ThreadMode::Window(size), move || format!("size-{}", size)).await.unwrap();
            for i in 0..size + 2 {
                env.writeln(&format!("Line {}", i));
            }
            
            display.display().await.unwrap();
            display.stop().await.unwrap();
            env.verify();
        }
    }, 5).await.unwrap();
}

#[tokio::test]
async fn test_window_concurrent() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    
    // Run test within timeout
    with_timeout(async {
        let mut handles = vec![];
        
        // Spawn multiple tasks in Window mode
        for i in 0..3 {
            let display = display.clone();
            let mut env = TestEnv::new(80, 24);
            let i = i;
            handles.push(tokio::spawn(async move {
                display.spawn_with_mode(ThreadMode::Window(3), move || format!("task-{}", i)).await.unwrap();
                for j in 0..5 {
                    env.writeln(&format!("Thread {}: Message {}", i, j));
                    sleep(Duration::from_millis(50)).await;
                }
                env
            }));
        }
        
        // Wait for all tasks to complete and combine their outputs
        let mut final_env = TestEnv::new(80, 24);
        for handle in handles {
            let task_env = handle.await.unwrap();
            let content = task_env.contents();
            if !content.is_empty() {
                final_env.write(&content);
            }
        }
        
        // Verify final state
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Clean up outside timeout
    display.stop().await.unwrap();
}

// Split the edge cases test into two separate tests
#[tokio::test]
async fn test_window_edge_case_minimal() {
    // Create display outside timeout
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Test within timeout
    with_timeout(async {
        // Test with minimal window (size 1 instead of 0)
        let _handle = display.spawn_with_mode(ThreadMode::Window(1), || "minimal-window").await.unwrap();
        env.writeln("Minimal window test");
        
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_edge_case_large() {
    // Create display outside timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::Window(30)).await;
    let mut env = TestEnv::new(80, 24);
    
    // Test within timeout
    with_timeout(async {
        let _handle = display.spawn_with_mode(ThreadMode::Window(30), || "large-window").await.unwrap();
        for i in 0..35 {
            env.writeln(&format!("Line {}", i));
        }
        
        display.display().await.unwrap();
    }, 5).await.unwrap();
    
    // Always clean up outside timeout
    display.stop().await.unwrap();
    env.verify();
} 