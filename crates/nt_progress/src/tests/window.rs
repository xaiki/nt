use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::tests::common::TestEnv;
use tokio::time::sleep;
use std::time::Duration;

#[tokio::test]
async fn test_window_basic() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::Window(3)).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::Window(3), || "window-test").await.unwrap();
    env.writeln("Test message");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_scroll() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::Window(3)).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::Window(3), || "scroll-test").await.unwrap();
    for i in 0..5 {
        env.writeln(&format!("Line {}", i));
    }
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_size() {
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
}

#[tokio::test]
async fn test_window_concurrent() {
    let display = ProgressDisplay::new().await;
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
        for line in task_env.expected {
            final_env.write(&line);
        }
    }
    
    // Verify final state
    display.display().await.unwrap();
    display.stop().await.unwrap();
    final_env.verify();
}

#[tokio::test]
async fn test_window_edge_cases() {
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Test with minimal window (size 1 instead of 0)
    let _handle = display.spawn_with_mode(ThreadMode::Window(1), || "minimal-window").await.unwrap();
    env.writeln("Minimal window test");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
    
    // Test with large window
    let display = ProgressDisplay::new_with_mode(ThreadMode::Window(30)).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::Window(30), || "large-window").await.unwrap();
    for i in 0..35 {
        env.writeln(&format!("Line {}", i));
    }
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
} 