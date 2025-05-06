use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::tests::common::TestEnv;
use tokio::time::sleep;
use std::time::Duration;

#[tokio::test]
async fn test_window_with_title_basic() {
    let mut display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "window-title-test").await.unwrap();
    env.writeln("Test message");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_window_with_title_update() {
    let mut display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
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
    let mut display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
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
    let mut display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
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
        let mut display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(size)).await;
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
    let mut display = ProgressDisplay::new().await;
    let mut handles = vec![];
    
    // Spawn multiple tasks in WindowWithTitle mode
    for i in 0..3 {
        let mut display = display.clone();
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
    let mut display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Test with minimum size window (2 lines)
    let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(2), || "min-window").await.unwrap();
    env.writeln("Minimum window test");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
    
    // Test with large window
    let mut display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(30)).await;
    let mut env = TestEnv::new(80, 24);
    
    let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(30), || "large-window").await.unwrap();
    for i in 0..35 {
        env.writeln(&format!("Line {}", i));
    }
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
    
    // Test trying to create a window with less than 2 lines (should fail)
    assert!(ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(1)).await
        .spawn_with_mode(ThreadMode::WindowWithTitle(1), || "too-small")
        .await
        .is_err());
}

#[tokio::test]
async fn test_window_with_title_special_chars() {
    let mut display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
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
    let mut display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await;
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
    let mut display = ProgressDisplay::new().await;
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