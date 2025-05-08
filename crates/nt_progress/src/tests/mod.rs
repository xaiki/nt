pub mod common;
pub mod display;
pub mod terminal;
pub mod limited;
pub mod window;
pub mod window_with_title;
pub mod capturing;
pub mod test_builder;
pub mod test_builder_example;
pub mod error_handling;
pub mod io;
mod passthrough;
pub mod custom_writer;
pub mod io_factory;

/// Tests for progress tracking functionality
#[cfg(test)]
pub mod progress {
    use crate::ProgressDisplay;
    use crate::modes::ThreadMode;
    use crate::terminal::TestEnv;
    use crate::tests::common::with_timeout;
    use anyhow::Result;
    use tokio::time::sleep;
    use std::time::Duration;

    /// Test basic progress tracking functionality
    #[tokio::test]
    async fn test_progress_tracking_basic() -> Result<()> {
        // Create a progress display
        let display = ProgressDisplay::new().await?;
        let mut env = TestEnv::new();
        
        // Run test logic inside timeout
        let _ = with_timeout(async {
            // Create a task with progress tracking
            let mut task = display.create_task(ThreadMode::Window(5), 10).await?;
            
            // Set the progress format
            task.set_progress_format("{current}/{total} ({percent}%)").await?;
            
            // Update progress and check the percentage
            for i in 0..=10 {
                let expected_percentage = (i as f64) * 10.0;
                task.set_progress(i).await?;
                let percentage = task.get_progress_percentage().await?;
                
                // Should be within 0.1% due to floating-point calculation
                assert!((percentage - expected_percentage).abs() < 0.1,
                    "Expected progress: {}, got: {}", expected_percentage, percentage);
                
                // Capture the current progress
                let message = format!("Progress: {}/{} ({}%)", i, 10, percentage as usize);
                task.capture_stdout(message.clone()).await?;
                env.writeln(&message);
                
                sleep(Duration::from_millis(50)).await;
            }
            
            // Verify final progress
            let final_percentage = task.get_progress_percentage().await?;
            assert_eq!(final_percentage, 100.0);
            
            Ok::<(), anyhow::Error>(())
        }, 5).await?;
        
        // Verify output
        display.display().await?;
        env.verify();
        
        // Clean up
        display.stop().await?;
        Ok(())
    }
    
    /// Test progress tracking with increment method
    #[tokio::test]
    async fn test_progress_tracking_increment() -> Result<()> {
        // Create a progress display
        let display = ProgressDisplay::new().await?;
        let mut env = TestEnv::new();
        
        // Run test logic inside timeout
        let _ = with_timeout(async {
            // Create a task with progress tracking
            let mut task = display.create_task(ThreadMode::Window(5), 10).await?;
            
            // Update progress by incrementing and check the percentage
            for i in 0..=10 {
                let expected_percentage = (i as f64) * 10.0;
                if i > 0 {
                    let percentage = task.update_progress().await?;
                    
                    // Should be within 0.1% due to floating-point calculation
                    assert!((percentage - expected_percentage).abs() < 0.1,
                        "Expected progress after increment: {}, got: {}", expected_percentage, percentage);
                }
                
                // Capture the current progress
                let percentage = task.get_progress_percentage().await?;
                let message = format!("Progress (incremental): {}/{} ({}%)", i, 10, percentage as usize);
                task.capture_stdout(message.clone()).await?;
                env.writeln(&message);
                
                sleep(Duration::from_millis(50)).await;
            }
            
            // Verify final progress
            let final_percentage = task.get_progress_percentage().await?;
            assert_eq!(final_percentage, 100.0);
            
            Ok::<(), anyhow::Error>(())
        }, 5).await?;
        
        // Verify output
        display.display().await?;
        env.verify();
        
        // Clean up
        display.stop().await?;
        Ok(())
    }
    
    /// Test progress bar display
    #[tokio::test]
    async fn test_progress_bar_display() -> Result<()> {
        // Create a progress display
        let display = ProgressDisplay::new().await?;
        let mut env = TestEnv::new();
        
        // Run test logic inside timeout
        let _ = with_timeout(async {
            // Create a task for the progress bar
            let task = display.create_task(ThreadMode::Window(5), 10).await?;
            let thread_id = task.thread_id();
            
            // Update progress bar and check the display
            for i in 0..=10 {
                // Update the progress bar
                display.progress_manager().update_progress_bar(thread_id, i, 10, "Processing").await?;
                
                // Add expected output to test env
                let progress_percent = (i * 100) / 10;
                let bar_width = 50;
                let filled = (progress_percent * bar_width) / 100;
                // Use standard ASCII characters instead of Unicode blocks
                let bar = "#".repeat(filled) + &"-".repeat(bar_width - filled);
                let expected = format!("{:<12} {}%|{}| {}/{}", "Processing", progress_percent, bar, i, 10);
                env.writeln(&expected);
                
                sleep(Duration::from_millis(50)).await;
            }
            
            Ok::<(), anyhow::Error>(())
        }, 5).await?;
        
        // Verify output
        display.display().await?;
        env.verify();
        
        // Clean up
        display.stop().await?;
        Ok(())
    }
} 