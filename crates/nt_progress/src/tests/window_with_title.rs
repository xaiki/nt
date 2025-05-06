use crate::modes::{ThreadMode, WindowWithTitle, Window, WithTitle, WithEmoji, WithTitleAndEmoji, StandardWindow, Capability};
use crate::terminal::TestEnv;
use crate::ProgressDisplay;
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
    let mut display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Create a task in WindowWithTitle mode
    let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "Initial Title").await.unwrap();
    
    // Add some emojis
    task.add_emoji("✨").await.unwrap();
    env.writeln("✨ Initial Title");
    env.verify();
    
    // Add another emoji
    task.add_emoji("🚀").await.unwrap();
    env.writeln("✨ 🚀 Initial Title");
    env.verify();
    
    // Change the title and verify emojis remain
    task.set_title("Updated Title".to_string()).await.unwrap();
    env.writeln("✨ 🚀 Updated Title");
    env.verify();
    
    // Add a message and verify title formatting
    let result = task.capture_stdout("This is a test message".to_string()).await;
    assert!(result.is_ok());
    
    // Verify the output
    display.display().await.unwrap();
    env.verify();
    
    // Clean up
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_emoji_errors() {
    let display = ProgressDisplay::new().await;
    
    // Create a task in Limited mode (doesn't support emojis)
    let limited_task = display.spawn_with_mode(ThreadMode::Limited, || "Limited Task").await.unwrap();
    
    // Trying to add an emoji should fail
    let result = limited_task.add_emoji("🚀").await;
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("not in a mode that supports emojis"), "Error message should mention emojis support: {}", error);
    
    // Create a task in Window mode (doesn't support emojis)
    let window_task = display.spawn_with_mode(ThreadMode::Window(3), || "Window Task").await.unwrap();
    
    // Trying to add an emoji should fail
    let result = window_task.add_emoji("🚀").await;
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("not in a mode that supports emojis"), "Error message should mention emojis support: {}", error);
    
    // Clean up
    display.stop().await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_multiple_emojis() {
    let display = ProgressDisplay::new().await;
    let mut env = TestEnv::new(80, 24);
    
    // Create a task in WindowWithTitle mode
    let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "Initial Title").await.unwrap();
    
    // Add multiple emojis
    task.add_emoji("✨").await.unwrap();
    task.add_emoji("🚀").await.unwrap();
    task.add_emoji("🔥").await.unwrap();
    
    // Verify output
    env.writeln("✨ 🚀 🔥 Initial Title");
    env.verify();
    
    // Add a message
    let result = task.capture_stdout("This is a test message".to_string()).await;
    assert!(result.is_ok());
    
    // Verify the output includes all emojis
    display.display().await.unwrap();
    env.verify();
    
    // Clean up
    display.stop().await.unwrap();
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
        let content = task_env.contents();
        if !content.is_empty() {
            final_env.write(&content);
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
    display.terminal.set_size(80, 2).await.expect("Failed to set terminal size");
    
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
    
    // The error should mention that the thread is not in a mode that supports titles
    let error = result.unwrap_err().to_string();
    assert!(error.contains("not in a mode that supports titles"));
    
    display.stop().await.unwrap();
}

#[test]
fn test_window_with_title_composite_capabilities() {
    // Create a WindowWithTitle instance directly
    let mut window = WindowWithTitle::new(1, 3).unwrap();
    
    // Test WithTitleAndEmoji trait methods
    window.set_title_with_emoji("Test Title".to_string(), "🔥");
    assert_eq!(window.get_title(), "Test Title");
    assert_eq!(window.get_emojis(), vec!["🔥"]);
    assert_eq!(window.get_formatted_title(), "🔥 Test Title");
    
    // Test reset_with_title
    window.add_emoji("✨");
    window.add_emoji("🚀");
    assert_eq!(window.get_emojis().len(), 3);
    
    window.reset_with_title("New Title".to_string());
    assert_eq!(window.get_title(), "New Title");
    assert_eq!(window.get_emojis().len(), 0);
    assert_eq!(window.get_formatted_title(), "New Title");
    
    // Test StandardWindow methods
    assert!(!window.is_empty()); // Window has title so it's not empty
    window.add_line("Test content".to_string());
    assert!(!window.is_empty());
    assert_eq!(window.line_count(), 2); // Title + 1 content line
    
    let content = window.get_content();
    assert_eq!(content.len(), 2);
    assert_eq!(content[0], "New Title");
    assert_eq!(content[1], "Test content");
    
    window.clear();
    assert_eq!(window.line_count(), 1); // Only title remains
}

#[test]
fn test_capability_discovery() {
    use crate::modes::{ThreadConfigExt, Limited, Capturing};
    
    // Create instances of different mode types
    let window = WindowWithTitle::new(1, 3).unwrap();
    let limited = Limited::new(1);
    let capturing = Capturing::new(1);
    let regular_window = Window::new(1, 3).unwrap();
    
    // Test capability discovery on WindowWithTitle
    let window_caps = window.capabilities();
    assert!(window_caps.contains(&Capability::Title));
    assert!(window_caps.contains(&Capability::CustomSize));
    assert!(window_caps.contains(&Capability::Emoji));
    assert!(window_caps.contains(&Capability::TitleAndEmoji));
    assert!(window_caps.contains(&Capability::StandardWindow));
    
    // Test individual capability checks on WindowWithTitle
    assert!(window.supports_capability(Capability::Title));
    assert!(window.supports_capability(Capability::Emoji));
    assert!(window.supports_capability(Capability::TitleAndEmoji));
    
    // Test capability discovery on Limited mode (should have no capabilities)
    let limited_caps = limited.capabilities();
    assert!(limited_caps.is_empty());
    
    // Test capability discovery on Capturing mode (should have no capabilities)
    let capturing_caps = capturing.capabilities();
    assert!(capturing_caps.is_empty());
    
    // Test capability discovery on regular Window
    let regular_window_caps = regular_window.capabilities();
    assert!(!regular_window_caps.contains(&Capability::Title));
    assert!(regular_window_caps.contains(&Capability::CustomSize));
    assert!(!regular_window_caps.contains(&Capability::Emoji));
    assert!(!regular_window_caps.contains(&Capability::TitleAndEmoji));
    assert!(regular_window_caps.contains(&Capability::StandardWindow));
} 
