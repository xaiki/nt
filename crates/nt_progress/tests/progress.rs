use anyhow::Result;
use nt_progress::{ProgressDisplay, ThreadMode, ProgressBarConfig, ProgressBarStyle};
use std::time::Duration;
use tokio::time::sleep;

/// Test time estimation features
///
/// This test verifies that time estimation features work correctly.
#[tokio::test]
async fn test_time_estimation() -> Result<()> {
    // Create test display with Window mode
    let display = ProgressDisplay::new().await?;
    
    // Create a task with progress tracking
    let task = display.create_task(ThreadMode::Window(10), 10).await?;
    
    // Initialize the time tracking with progress update
    task.set_progress(0).await?;
    
    // Now wait a bit to allow the timer to start measuring 
    sleep(Duration::from_millis(100)).await;
    
    // Check initial states
    let initial_eta = task.get_estimated_time_remaining().await?;
    let initial_speed = task.get_progress_speed().await?;
    let initial_elapsed = task.get_elapsed_time().await?;
    
    println!("Initial ETA: {:?}, Initial Speed: {:?}, Initial Elapsed: {:?}", 
             initial_eta, initial_speed, initial_elapsed);
    
    // Initially, there should be no ETA since we don't have enough progress data
    assert!(initial_eta.is_none(), "Initial ETA should be None");
    
    // Update progress a few times with delays to simulate work
    for i in 1..=5 {
        // Add a delay to simulate work
        sleep(Duration::from_millis(200)).await;
        
        // Update progress
        task.set_progress(i).await?;
        
        // Get time-related metrics
        let eta = task.get_estimated_time_remaining().await?;
        let speed = task.get_progress_speed().await?;
        let elapsed = task.get_elapsed_time().await?;
        
        println!(
            "Progress: {}/10, Elapsed: {:?}, ETA: {:?}, Speed: {:?}", 
            i, elapsed, eta, speed
        );
        
        // After a few updates, we should start getting estimates
        if i >= 3 {
            // We may have estimates now
            if let Some(eta_duration) = eta {
                // Ensure ETA is reasonable
                let eta_secs = eta_duration.as_secs_f64();
                assert!(eta_secs >= 0.0 && eta_secs < 10.0, 
                       "ETA should be reasonable: {:?}", eta_duration);
            }
            
            if let Some(spd) = speed {
                // Speed should be positive
                assert!(spd > 0.0, "Speed should be positive: {}", spd);
            }
        }
    }
    
    // Complete the task
    task.set_progress(10).await?;
    
    // After completion
    let final_eta = task.get_estimated_time_remaining().await?;
    let final_speed = task.get_progress_speed().await?;
    let final_elapsed = task.get_elapsed_time().await?;
    
    println!(
        "Final metrics: Elapsed: {:?}, ETA: {:?}, Speed: {:?}", 
        final_elapsed, final_eta, final_speed
    );
    
    // After completion, ETA should be None or very small
    if let Some(time) = final_eta {
        assert!(time.as_secs_f64() < 1.0, "ETA should be very small after completion");
    }
    
    // Properly clean up
    display.stop().await?;
    Ok(())
}

/// Test progress tracking with incremental updates
#[tokio::test]
async fn test_progress_tracking_increment() -> Result<()> {
    // Create test display with Window mode
    let display = ProgressDisplay::new().await?;
    
    // Create a task with progress tracking
    let task = display.create_task(ThreadMode::Window(10), 100).await?;
    
    // Update progress incrementally
    for i in 0..10 {
        let progress = i * 10;
        task.set_progress(progress).await?;
        
        // Check progress percentage
        let percentage = task.get_progress_percentage().await?;
        assert!((percentage - (progress as f64)).abs() < 0.01, 
                "Expected progress: {}, got: {}", progress, percentage);
        
        sleep(Duration::from_millis(50)).await;
    }
    
    // Properly clean up
    display.stop().await?;
    Ok(())
}

/// Test the get_elapsed_time API
#[tokio::test]
async fn test_get_elapsed_time() -> Result<()> {
    // Create test display with Window mode
    let display = ProgressDisplay::new().await?;
    
    // Create a task with progress tracking
    let task = display.create_task(ThreadMode::Window(10), 10).await?;
    
    // Initialize the time tracking by calling set_progress(0)
    task.set_progress(0).await?;
    
    // Wait a significant amount of time to ensure timer is running
    sleep(Duration::from_millis(200)).await;
    
    // Initial elapsed time should now be measurable
    let initial_elapsed = task.get_elapsed_time().await?;
    println!("Initial elapsed time after 200ms: {:?}", initial_elapsed);
    
    // Wait a bit longer
    sleep(Duration::from_millis(300)).await;
    
    // Elapsed time should have increased
    let elapsed_after_delay = task.get_elapsed_time().await?;
    println!("Elapsed time after +300ms: {:?}", elapsed_after_delay);
    
    assert!(elapsed_after_delay > initial_elapsed, 
           "Elapsed time should increase: {:?} > {:?}", 
           elapsed_after_delay, initial_elapsed);
    
    // Make some progress
    task.set_progress(5).await?;
    
    // Wait a bit more
    sleep(Duration::from_millis(300)).await;
    
    // Elapsed time should be even more
    let final_elapsed = task.get_elapsed_time().await?;
    println!("Final elapsed time after +300ms: {:?}", final_elapsed);
    
    assert!(final_elapsed > elapsed_after_delay, 
           "Elapsed time should continue increasing");
    
    // Properly clean up
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_multi_progress_bar() -> anyhow::Result<()> {
    // Create the progress display
    let display = ProgressDisplay::new().await?;
    
    // Create a window mode task to display the multi-progress bars
    let task = display.create_task(nt_progress::ThreadMode::Window(10), 1).await?;
    
    // Create a multi-progress bar group
    display.create_multi_progress_bar_group("downloads").await?;
    
    // Add several progress bars with different styles
    display.add_progress_bar(
        "downloads", 
        "file1", 
        ProgressBarConfig::new()
            .prefix("File 1")
            .width(20)
            .style(ProgressBarStyle::Standard)
    ).await?;
    
    display.add_progress_bar(
        "downloads", 
        "file2", 
        ProgressBarConfig::new()
            .prefix("File 2")
            .width(20)
            .style(ProgressBarStyle::Block)
    ).await?;
    
    display.add_progress_bar(
        "downloads", 
        "file3", 
        ProgressBarConfig::new()
            .prefix("File 3")
            .width(20)
            .style(ProgressBarStyle::Gradient)
            .show_speed(true)
            .show_eta(true)
    ).await?;
    
    // Display the multi-progress bar group on the task
    display.display_multi_progress_bar_group(task.thread_id(), "downloads").await?;
    
    // Update progress bars over time
    for i in 0..=10 {
        let progress = i * 10;
        
        // Update each progress bar with different progress levels
        display.update_multi_progress_bar("downloads", "file1", progress, 100).await?;
        display.update_multi_progress_bar("downloads", "file2", progress / 2, 100).await?;
        display.update_multi_progress_bar("downloads", "file3", progress * 2, 200).await?;
        
        // Display updated progress
        display.display_multi_progress_bar_group(task.thread_id(), "downloads").await?;
        
        // Short pause to see the progress
        sleep(Duration::from_millis(200)).await;
    }
    
    // Remove one progress bar
    display.remove_progress_bar("downloads", "file2").await?;
    display.display_multi_progress_bar_group(task.thread_id(), "downloads").await?;
    sleep(Duration::from_millis(500)).await;
    
    // Remove the entire group
    display.remove_multi_progress_bar_group("downloads").await?;
    
    // Clean up
    display.stop().await?;
    Ok(())
} 