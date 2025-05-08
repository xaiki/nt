use crate::modes::ThreadMode;
use crate::terminal::TestEnv;
use crate::ProgressDisplay;
use tokio::time::sleep;
use std::time::Duration;
use crate::tests::common::with_timeout;
use anyhow::Result;

#[tokio::test]
async fn test_window_with_title_basic() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "window-title-test").await?;
        task.capture_stdout("Test message".to_string()).await?;
        env.writeln("Test message");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_update() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title-update-test").await?;
        task.capture_stdout("First message".to_string()).await?;
        task.capture_stdout("Second message".to_string()).await?;
        env.writeln("First message");
        env.writeln("Second message");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_persistence() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title-persistence-test").await?;
        task.capture_stdout("Message 1".to_string()).await?;
        env.writeln("Message 1");
        sleep(Duration::from_millis(50)).await;
        
        task.capture_stdout("Message 2".to_string()).await?;
        env.writeln("Message 2");
        sleep(Duration::from_millis(50)).await;
        
        task.capture_stdout("Message 3".to_string()).await?;
        env.writeln("Message 3");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_emoji() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Create a task in WindowWithTitle mode
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "Initial Title").await?;
        
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
        task.add_emoji("✨").await?;
        env.writeln("✨ Initial Title");
        env.verify();
        
        // Add another emoji
        task.add_emoji("🚀").await?;
        env.writeln("✨ 🚀 Initial Title");
        env.verify();
        
        // Change the title and verify emojis remain
        task.set_title("Updated Title".to_string()).await?;
        env.writeln("✨ 🚀 Updated Title");
        env.verify();
        
        // Add a message and verify title formatting
        task.capture_stdout("This is a test message".to_string()).await?;
        
        // Verify the output
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_emoji_errors() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Create a task in Limited mode (doesn't support emojis)
        let limited_task = display.spawn_with_mode(ThreadMode::Limited, || "Limited Task").await?;
        
        // Trying to add an emoji should fail
        let result = limited_task.add_emoji("🚀").await;
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("not in a mode that supports emojis"), "Error message should mention emojis support: {}", error);
        
        // Create a task in Window mode (doesn't support emojis)
        let window_task = display.spawn_with_mode(ThreadMode::Window(3), || "Window Task").await?;
        
        // Trying to add an emoji should fail
        let result = window_task.add_emoji("🚀").await;
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("not in a mode that supports emojis"), "Error message should mention emojis support: {}", error);
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_multiple_emojis() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Create a task in WindowWithTitle mode
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "Initial Title").await?;
        
        // Add multiple emojis
        task.add_emoji("✨").await?;
        task.add_emoji("🚀").await?;
        task.add_emoji("🔥").await?;
        
        // Verify output
        env.writeln("✨ 🚀 🔥 Initial Title");
        env.verify();
        
        // Add a message
        task.capture_stdout("This is a test message".to_string()).await?;
        
        // Verify the output includes all emojis
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_size() -> Result<()> {
    for size in [2, 3, 5, 10] {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(size)).await?;
        let mut env = TestEnv::new();
        
        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(size), move || format!("size-{}", size)).await?;
            for i in 0..size + 2 {
                let message = format!("Line {}", i);
                task.capture_stdout(message.clone()).await?;
                env.writeln(&message);
            }
            
            display.display().await?;
            env.verify();
            Ok::<(), anyhow::Error>(())
        }, 30).await?;

        // Clean up OUTSIDE timeout
        display.stop().await?;
    }
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_concurrent() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut handles = vec![];
        
        // Spawn multiple tasks in WindowWithTitle mode
        for i in 0..3 {
            let display_ref = display.clone();
            let mut env = TestEnv::new();
            handles.push(tokio::spawn(async move {
                let mut task = display_ref.spawn_with_mode(ThreadMode::WindowWithTitle(3), move || format!("task-{}", i)).await?;
                for j in 0..5 {
                    let message = format!("Thread {}: Message {}", i, j);
                    task.capture_stdout(message.clone()).await?;
                    env.writeln(&message);
                    sleep(Duration::from_millis(50)).await;
                }
                Ok::<TestEnv, anyhow::Error>(env)
            }));
        }
        
        // Wait for all tasks to complete and combine their outputs
        let mut final_env = TestEnv::new();
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
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_edge_cases() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Enable error propagation for this test
        crate::modes::set_error_propagation(true);
        
        // Test edge cases
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "edge-case").await?;
        
        // Test very short line
        task.capture_stdout("x".to_string()).await?;
        env.writeln("x");
        
        display.display().await?;
        env.verify();
        
        // Test trying to create a window with less than 2 lines (should fail)
        let result = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(1)).await?
            .spawn_with_mode(ThreadMode::WindowWithTitle(1), || "too-small").await;
        assert!(result.is_err());
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_special_chars() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "special-chars").await?;
        
        // Test various special characters
        let special_chars = vec![
            "!@#$%^&*()",
            "\\n\\t\\r",
            "🎉 🌟 🚀",
            "Unicode: 你好, こんにちは",
            "Mixed: ABC123!@#",
        ];
        
        for chars in special_chars {
            task.capture_stdout(chars.to_string()).await?;
            env.writeln(chars);
        }
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_long_lines() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "long-lines").await?;
        
        // Test various line lengths
        let lines = vec![
            "Short line",
            "A bit longer line with some more content",
            "A very long line that should definitely exceed the normal terminal width and require wrapping or truncation depending on the implementation",
            "Another very long line with special characters: !@#$%^&*()_+ and some unicode: 你好, こんにちは, to make it even more challenging",
        ];
        
        for line in lines {
            task.capture_stdout(line.to_string()).await?;
            env.writeln(line);
        }
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_terminal_size() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "terminal-size").await?;
        
        // Test with different content lengths
        for i in 1..5 {
            let message = "X".repeat(i * 20);
            task.capture_stdout(message.clone()).await?;
            env.writeln(&message);
        }
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_set_title() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new_with_mode(ThreadMode::WindowWithTitle(3)).await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "initial-title").await?;
        
        // Test setting different titles
        let titles = vec![
            "New Title",
            "Another Title",
            "Title with special chars: !@#$%^&*()",
            "Title with emoji: 🎉 🌟 🚀",
            "Title with unicode: 你好, こんにちは",
        ];
        
        for title in titles {
            task.set_title(title.to_string()).await?;
            task.capture_stdout("Content after title change".to_string()).await?;
            env.writeln(title);
        }
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_window_with_title_set_title_error() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Create a task in Limited mode (doesn't support title)
        let limited_task = display.spawn_with_mode(ThreadMode::Limited, || "Limited Task").await?;
        
        // Trying to set title should fail
        let result = limited_task.set_title("New Title".to_string()).await;
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("not in a mode that supports title"), "Error message should mention title support: {}", error);
        
        // Create a task in Window mode (doesn't support title)
        let window_task = display.spawn_with_mode(ThreadMode::Window(3), || "Window Task").await?;
        
        // Trying to set title should fail
        let result = window_task.set_title("New Title".to_string()).await;
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("not in a mode that supports title"), "Error message should mention title support: {}", error);
        Ok::<(), anyhow::Error>(())
    }, 30).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
} 
