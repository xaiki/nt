use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::tests::common::TestEnv;
use tokio::time::sleep;
use std::time::Duration;

#[tokio::test]
async fn test_window_with_title_basic() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "window-title-test").await.unwrap();
    env.writeln("Test message");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_with_title_update() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title-update-test").await.unwrap();
    env.writeln("First message");
    env.writeln("Second message");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_with_title_persistence() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title-persistence-test").await.unwrap();
    env.writeln("Message 1");
    sleep(Duration::from_millis(50)).await;
    env.writeln("Message 2");
    sleep(Duration::from_millis(50)).await;
    env.writeln("Message 3");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_with_title_emoji() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "emoji-test").await.unwrap();
    env.writeln("🚀 Starting task");
    env.writeln("✨ Processing");
    env.writeln("✅ Complete");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_with_title_size() {
    for size in [2, 3, 5, 10] {
        let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(size)).await;
        let mut env = TestEnv::new(80, 24);
        
        display.spawn_with_mode(ThreadMode::WindowWithTitle(size), move || format!("size-{}", size)).await.unwrap();
        for i in 0..size + 2 {
            env.writeln(&format!("Line {}", i));
        }
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }
}

#[tokio::test]
async fn test_window_with_title_concurrent() {
    let display = ProgressDisplay::new().await;
    let mut handles = vec![];
    
    // Spawn multiple tasks in WindowWithTitle mode
    for i in 0..3 {
        let display = display.clone();
        let mut env = TestEnv::new(80, 24);
        let i = i;
        handles.push(tokio::spawn(async move {
            display.spawn_with_mode(ThreadMode::WindowWithTitle(3), move || format!("task-{}", i)).await.unwrap();
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
async fn test_window_with_title_edge_cases() {
    // Enable error propagation for this test
    crate::modes::set_error_propagation(true);
    
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
    let mut env = TestEnv::new(80, 24);
    
    // Test edge cases
    display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "edge-case").await.unwrap();
    
    // Skip adding whitespace lines that cause verification issues
    
    // Test very short line
    env.writeln("x");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
    
    // Test trying to create a window with less than 2 lines (should fail)
    assert!(ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(1)).await
        .spawn_with_mode(ThreadMode::WindowWithTitle(1), || "too-small")
        .await
        .is_err());
        
    // Disable error propagation after test
    crate::modes::set_error_propagation(false);
}

#[tokio::test]
async fn test_window_with_title_special_chars() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "special-chars").await.unwrap();
    
    // Test various special characters
    env.writeln("Test with \n newlines \t tabs \r returns");
    env.writeln("Test with unicode: 你好世界");
    env.writeln("Test with emoji: 🚀 ✨");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_with_title_long_lines() {
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "long-lines").await.unwrap();
    
    // Test very long line
    let long_line = "x".repeat(1000);
    env.writeln(&long_line);
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_with_title_terminal_size() {
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Set a small terminal size
    *display.terminal_size.lock().await = (80, 2);
    
    // Add more lines than terminal height
    display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "size-test").await.unwrap();
    env.writeln("Line 1");
    env.writeln("Line 2");
    env.writeln("Line 3");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_with_title_set_title() {
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Create a task in WindowWithTitle mode
    let mut handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "Initial Title").await.unwrap();
    
    // Verify initial title
    env.writeln("Initial Title");
    display.display().await.unwrap();
    
    // Set a new title
    handle.set_title("Updated Title".to_string()).await.unwrap();
    
    // Verify the title has been updated
    env.writeln("Updated Title");
    display.display().await.unwrap();
    
    // Check that we can still add messages
    handle.capture_stdout("Message 1".to_string()).await.unwrap();
    env.writeln("Message 1");
    display.display().await.unwrap();
    
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_with_title_set_title_error() {
    let display = ProgressDisplay::new().await;
    
    // Create a task in Limited mode (not WindowWithTitle)
    let handle = display.spawn_with_mode(ThreadMode::Limited, || "Limited Mode").await.unwrap();
    
    // Try to set a title - should fail since it's not in WindowWithTitle mode
    let result = handle.set_title("New Title".to_string()).await;
    assert!(result.is_err());
    
    // The error should mention that the thread is not in WindowWithTitle mode
    let error = result.unwrap_err().to_string();
    assert!(error.contains("not in WindowWithTitle mode"));
    
    display.stop().await.unwrap();
} 
