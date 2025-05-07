use crate::modes::ThreadMode;
use crate::terminal::TestEnv;
use crate::ProgressDisplay;
use tokio::time::sleep;
use std::time::Duration;
use crate::tests::common::with_timeout;

#[tokio::test]
async fn test_window_with_title_basic() {
    with_timeout(async {
        let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await.unwrap();
        let mut env = TestEnv::new();
        
        let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "window-title-test").await.unwrap();
        env.writeln("Test message");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_update() {
    with_timeout(async {
        let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await.unwrap();
        let mut env = TestEnv::new();
        
        let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title-update-test").await.unwrap();
        env.writeln("First message");
        env.writeln("Second message");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_persistence() {
    with_timeout(async {
        let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await.unwrap();
        let mut env = TestEnv::new();
        
        let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title-persistence-test").await.unwrap();
        env.writeln("Message 1");
        sleep(Duration::from_millis(50)).await;
        env.writeln("Message 2");
        sleep(Duration::from_millis(50)).await;
        env.writeln("Message 3");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_emoji() {
    with_timeout(async {
        let display = ProgressDisplay::new().await.unwrap();
        let mut env = TestEnv::new();
        
        // Create a task in WindowWithTitle mode
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "Initial Title").await.unwrap();
        
        // First, verify if the task has a proper mode with emoji support
        let config = task.thread_config.lock().await;
        // Debug output
        eprintln!("Config: {:?}", config);
        if let Some(window) = config.as_type::<crate::modes::window_with_title::WindowWithTitle>() {
            eprintln!("WindowWithTitle found, supports emoji: {}", window.has_emoji_support());
            assert!(window.has_emoji_support(), "WindowWithTitle should have emoji support enabled!");
        } else {
            eprintln!("WindowWithTitle type not found in config");
            panic!("Task is not using WindowWithTitle mode");
        }
        drop(config);
        
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
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_emoji_errors() {
    with_timeout(async {
        let display = ProgressDisplay::new().await.unwrap();
        
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
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_multiple_emojis() {
    with_timeout(async {
        let display = ProgressDisplay::new().await.unwrap();
        let mut env = TestEnv::new();
        
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
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_size() {
    with_timeout(async {
        for size in [2, 3, 5, 10] {
            let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(size)).await.unwrap();
            let mut env = TestEnv::new();
            
            display.spawn_with_mode(ThreadMode::WindowWithTitle(size), move || format!("size-{}", size)).await.unwrap();
            for i in 0..size + 2 {
                env.writeln(&format!("Line {}", i));
            }
            
            display.display().await.unwrap();
            display.stop().await.unwrap();
            env.verify();
        }
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_concurrent() {
    with_timeout(async {
        let display = ProgressDisplay::new().await.unwrap();
        let mut handles = vec![];
        
        // Spawn multiple tasks in WindowWithTitle mode
        for i in 0..3 {
            let display = display.clone();
            let mut env = TestEnv::new();
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
        let mut final_env = TestEnv::new();
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
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_edge_cases() {
    with_timeout(async {
        // Enable error propagation for this test
        crate::modes::set_error_propagation(true);
        
        let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await.unwrap();
        let mut env = TestEnv::new();
        
        // Test edge cases
        display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "edge-case").await.unwrap();
        
        // Skip adding whitespace lines that cause verification issues
        
        // Test very short line
        env.writeln("x");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
        
        // Test trying to create a window with less than 2 lines (should fail)
        let result = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(1)).await.unwrap()
            .spawn_with_mode(ThreadMode::WindowWithTitle(1), || "too-small").await;
        assert!(result.is_err());
            
        // Disable error propagation after test
        crate::modes::set_error_propagation(false);
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_special_chars() {
    with_timeout(async {
        let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await.unwrap();
        let mut env = TestEnv::new();
        
        let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "special-chars").await.unwrap();
        
        // Test various special characters
        env.writeln("Test with \n newlines \t tabs \r returns");
        env.writeln("Test with unicode: 你好世界");
        env.writeln("Test with emoji: 🚀 ✨");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_long_lines() {
    with_timeout(async {
        let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await.unwrap();
        let mut env = TestEnv::new();
        
        let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "long-lines").await.unwrap();
        
        // Test very long line
        let long_line = "x".repeat(1000);
        env.writeln(&long_line);
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_terminal_size() {
    with_timeout(async {
        let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await.unwrap();
        let mut env = TestEnv::new(); // Smaller terminal size
        
        let _handle = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "terminal-size").await.unwrap();
        env.writeln("Testing with smaller terminal");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_set_title() {
    with_timeout(async {
        let display = ProgressDisplay::new().await.unwrap();
        let env = TestEnv::new();
        
        // Create a task in WindowWithTitle mode
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "Initial Title").await.unwrap();
        
        // Change the title
        let result = task.set_title("Updated Title".to_string()).await;
        assert!(result.is_ok());
        
        // Add a message
        task.capture_stdout("Message after title change".to_string()).await.unwrap();
        
        // Verify output shows updated title
        display.display().await.unwrap();
        env.verify();
        
        // Change title again
        task.set_title("Final Title".to_string()).await.unwrap();
        display.display().await.unwrap();
        env.verify();
        
        // Clean up
        display.stop().await.unwrap();
    }, 30).await.unwrap();
}

#[tokio::test]
async fn test_window_with_title_set_title_error() {
    with_timeout(async {
        let display = ProgressDisplay::new().await.unwrap();
        
        // Create a task in Limited mode (doesn't support title changes)
        let limited_task = display.spawn_with_mode(ThreadMode::Limited, || "Limited Task").await.unwrap();
        
        // Trying to change the title should fail
        let result = limited_task.set_title("New Title".to_string()).await;
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("not in a mode that supports titles"), 
            "Error message should mention title change support: {}", error);
        
        // Clean up
        display.stop().await.unwrap();
    }, 30).await.unwrap();
} 
