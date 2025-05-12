use std::time::Duration;
use tokio::time::sleep;
use crate::ProgressDisplay;
use crate::ThreadMode;
use crate::terminal::TestEnv;
use crate::tests::common::with_timeout;
use anyhow::Result;
use crate::ui::formatter::{ProgressTemplate, TemplateContext};
use crate::modes::factory::set_error_propagation;

/**
 * IMPORTANT: Testing Pattern to Prevent Test Hangs
 * 
 * ProgressDisplay tests MUST follow this pattern to avoid hanging:
 * 
 * 1. Create ProgressDisplay OUTSIDE the timeout block
 * 2. Run test logic INSIDE the timeout block 
 * 3. Call display.stop() OUTSIDE the timeout block (to ensure cleanup even if timeout occurs)
 */

#[tokio::test]
async fn test_progress_display_high_concurrency() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let _env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let total_jobs = 20;
        
        // First create all tasks upfront to reduce setup overhead
        let mut tasks = Vec::with_capacity(total_jobs);
        for i in 0..total_jobs {
            let task = display.spawn_with_mode(ThreadMode::Window(5), move || format!("task-{}", i)).await?;
            tasks.push(task);
        }
        
        // Spawn multiple tasks that send messages concurrently
        let mut handles = vec![];
        for (i, mut task) in tasks.into_iter().enumerate() {
            handles.push(tokio::spawn(async move {
                for j in 0..10 {
                    let message = format!("Thread {}: Message {}", i, j);
                    task.capture_stdout(message).await?;
                    // Reduce sleep time to speed up the test
                    sleep(Duration::from_millis(5)).await;
                }
                Ok::<(), anyhow::Error>(())
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await??;
        }
        
        // Display once at the end instead of per task
        display.display().await?;
        
        Ok::<(), anyhow::Error>(())
    }, 60).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_progress_display_different_modes() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Spawn tasks with different modes
        let mut task1 = display.spawn_with_mode(ThreadMode::Limited, || "limited-task".to_string()).await?;
        task1.capture_stdout("Test 1".to_string()).await?;
        task1.capture_stdout("Test 2".to_string()).await?;
        env.writeln("Test 1");
        env.writeln("Test 2");
        display.display().await?;

        let mut task2 = display.spawn_with_mode(ThreadMode::Window(2), || "window-task".to_string()).await?;
        task2.capture_stdout("Test 3".to_string()).await?;
        task2.capture_stdout("Test 4".to_string()).await?;
        env.writeln("Test 3");
        env.writeln("Test 4");
        display.display().await?;

        let mut task3 = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title-task".to_string()).await?;
        task3.capture_stdout("Test 5".to_string()).await?;
        env.writeln("Test 5");
        display.display().await?;
        
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 5).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_progress_display_error_handling() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let _env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Enable error propagation for this test
        set_error_propagation(true);
        
        // Test invalid mode configuration
        let result = display.spawn_with_mode(ThreadMode::Window(0), || "invalid".to_string()).await;
        assert!(result.is_err());
        Ok::<(), anyhow::Error>(())
    }, 3).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    
    // Reset error propagation
    set_error_propagation(false);
    Ok(())
}

#[tokio::test]
async fn test_progress_display_limited_mode() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test Limited mode
        let mut task = display.spawn_with_mode(ThreadMode::Limited, || "limited".to_string()).await?;
        for i in 0..10 {
            let message = format!("Line {}", i);
            task.capture_stdout(message.clone()).await?;
            env.writeln(&message);
        }
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 3).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_progress_display_capturing_mode() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test Capturing mode
        let mut task = display.spawn_with_mode(ThreadMode::Capturing, || "capturing".to_string()).await?;
        for i in 0..10 {
            let message = format!("Line {}", i);
            task.capture_stdout(message.clone()).await?;
            env.writeln(&message);
        }
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 3).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_progress_display_window_mode() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test Window mode
        let mut task = display.spawn_with_mode(ThreadMode::Window(3), || "window".to_string()).await?;
        for i in 0..10 {
            let message = format!("Line {}", i);
            task.capture_stdout(message.clone()).await?;
            env.writeln(&message);
        }
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 3).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_progress_display_window_with_title_mode() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test WindowWithTitle mode
        let mut task = display.spawn_with_mode(ThreadMode::WindowWithTitle(3), || "title".to_string()).await?;
        task.capture_stdout("Test with emoji 🚀".to_string()).await?;
        task.capture_stdout("Another line 📝".to_string()).await?;
        env.writeln("Test with emoji 🚀");
        env.writeln("Another line 📝");
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 3).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_progress_display_empty_output() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test empty output
        let mut task = display.spawn_with_mode(ThreadMode::Limited, || "empty".to_string()).await?;
        task.capture_stdout("".to_string()).await?;
        env.writeln("");
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 3).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_progress_display_long_lines() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test very long lines
        let long_line = "x".repeat(1000);
        let mut task = display.spawn_with_mode(ThreadMode::Limited, || "long".to_string()).await?;
        task.capture_stdout(long_line.clone()).await?;
        env.writeln(&long_line);
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 3).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_progress_display_special_chars() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test special characters
        let mut task = display.spawn_with_mode(ThreadMode::Limited, || "special".to_string()).await?;
        let special_chars = "Special characters: !@#$%^&*()_+{}|:<>?~`-=[]\\;',./";
        task.capture_stdout(special_chars.to_string()).await?;
        env.writeln(special_chars);
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 3).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_progress_display_concurrency() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let _env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut handles = vec![];
        
        // Spawn multiple tasks
        for i in 0..5 {
            let display_ref = display.clone();
            let mut task_env = TestEnv::new();
            handles.push(tokio::spawn(async move {
                let mut task = display_ref.spawn_with_mode(ThreadMode::Window(3), move || format!("task-{}", i)).await?;
                for j in 0..5 {
                    let message = format!("Thread {}: Message {}", i, j);
                    task.capture_stdout(message.clone()).await?;
                    task_env.writeln(&message);
                    sleep(Duration::from_millis(50)).await;
                }
                Ok::<TestEnv, anyhow::Error>(task_env)
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
async fn test_progress_display_resource_cleanup() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Create multiple tasks and verify they're cleaned up properly
        let mut tasks = vec![];
        for i in 0..5 {
            let mut task = display.spawn_with_mode(ThreadMode::Window(3), move || format!("task-{}", i)).await?;
            task.capture_stdout(format!("Message from task {}", i)).await?;
            tasks.push(task);
        }
        
        // Verify all tasks are working
        for (i, _) in tasks.iter().enumerate() {
            env.writeln(&format!("Message from task {}", i));
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
async fn test_progress_display_custom_indicators() -> Result<()> {
    // Create a new TestEnv and ProgressDisplay
    let env = TestEnv::new();
    let display = ProgressDisplay::new().await?;
    
    // Create a task with Window mode
    let mut task = display.spawn_with_mode(ThreadMode::Window(5), || "Progress Indicators Test").await?;
    
    // Test each progress indicator type
    let progress_values = [0.0, 0.25, 0.5, 0.75, 1.0];
    
    // Traditional bar indicator
    task.capture_stdout("Bar indicator:".to_string()).await?;
    for progress in progress_values {
        let template = ProgressTemplate::new("{progress:bar}");
        let mut ctx = TemplateContext::new();
        ctx.set("progress", progress);
        let message = template.render(&ctx)?;
        task.capture_stdout(message).await?;
    }
    
    // Block indicator
    task.capture_stdout("Block indicator:".to_string()).await?;
    for progress in progress_values {
        let template = ProgressTemplate::new("{progress:bar:block}");
        let mut ctx = TemplateContext::new();
        ctx.set("progress", progress);
        let message = template.render(&ctx)?;
        task.capture_stdout(message).await?;
    }
    
    // Spinner indicator
    task.capture_stdout("Spinner indicator:".to_string()).await?;
    for progress in progress_values {
        let template = ProgressTemplate::new("{progress:bar:spinner}");
        let mut ctx = TemplateContext::new();
        ctx.set("progress", progress);
        let message = template.render(&ctx)?;
        task.capture_stdout(message).await?;
    }
    
    // Numeric indicator
    task.capture_stdout("Numeric indicator:".to_string()).await?;
    for progress in progress_values {
        let template = ProgressTemplate::new("{progress:bar:numeric}");
        let mut ctx = TemplateContext::new();
        ctx.set("progress", progress);
        let message = template.render(&ctx)?;
        task.capture_stdout(message).await?;
    }
    
    // Custom indicator configurations
    task.capture_stdout("Custom configurations:".to_string()).await?;
    
    // Custom bar width and characters
    let template = ProgressTemplate::new("Custom bar: {progress:bar:bar:20:#}");
    let mut ctx = TemplateContext::new();
    ctx.set("progress", 0.5);
    task.capture_stdout(template.render(&ctx)?).await?;
    
    // Custom block characters
    let template = ProgressTemplate::new("Custom blocks: {progress:bar:block:10:▮▯}");
    let mut ctx = TemplateContext::new();
    ctx.set("progress", 0.5);
    task.capture_stdout(template.render(&ctx)?).await?;
    
    // Custom spinner frames
    let template = ProgressTemplate::new("Custom spinner: {progress:bar:spinner:▖▘▝▗}");
    let mut ctx = TemplateContext::new();
    ctx.set("progress", 0.5);
    task.capture_stdout(template.render(&ctx)?).await?;
    
    // Custom numeric without percent sign
    let template = ProgressTemplate::new("Custom numeric: {progress:bar:numeric:false}/100");
    let mut ctx = TemplateContext::new();
    ctx.set("progress", 0.5);
    task.capture_stdout(template.render(&ctx)?).await?;
    
    // New custom indicators
    task.capture_stdout("New custom indicators:".to_string()).await?;
    
    // Dots indicator
    let template = ProgressTemplate::new("Dots indicator: {progress:bar:custom:dots:10}");
    let mut ctx = TemplateContext::new();
    ctx.set("progress", 0.5);
    task.capture_stdout(template.render(&ctx)?).await?;
    
    // Braille indicator
    let template = ProgressTemplate::new("Braille indicator: {progress:bar:custom:braille:10}");
    let mut ctx = TemplateContext::new();
    ctx.set("progress", 0.5);
    task.capture_stdout(template.render(&ctx)?).await?;
    
    // Gradient indicator
    let template = ProgressTemplate::new("Gradient indicator: {progress:bar:custom:gradient:10:blue:cyan}");
    let mut ctx = TemplateContext::new();
    ctx.set("progress", 0.5);
    task.capture_stdout(template.render(&ctx)?).await?;
    
    // Display and verify output
    display.display().await?;
    env.verify();
    
    // Clean up
    display.stop().await?;
    
    Ok(())
} 
