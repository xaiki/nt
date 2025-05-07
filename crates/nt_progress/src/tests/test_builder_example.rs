use crate::tests::test_builder::TestBuilder;
use crate::tests::common::with_timeout;
use anyhow::Error;
use crate::ProgressDisplay;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProgressDisplay;
    use anyhow::{Result, Error};

    #[tokio::test]
    async fn test_builder_basic_message() -> Result<(), Error> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let mut builder = TestBuilder::new();

        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            builder.test_basic_message(&display).await.map_err(anyhow::Error::from)?;
            Ok::<(), Error>(())
        }, 15).await?;

        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_builder_concurrent_tasks() -> Result<(), Error> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let mut builder = TestBuilder::new();

        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            builder.test_concurrent_tasks_with_display(&display, 5).await.map_err(anyhow::Error::from)?;
            Ok::<(), Error>(())
        }, 15).await?;

        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_builder_edge_case_empty_message() -> Result<(), Error> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let mut builder = TestBuilder::new();

        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            builder.test_edge_case_empty_message(&display).await.map_err(anyhow::Error::from)?;
            Ok::<(), Error>(())
        }, 15).await?;

        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_builder_edge_case_long_message() -> Result<(), Error> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let mut builder = TestBuilder::new();

        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            builder.test_edge_case_long_message(&display).await.map_err(anyhow::Error::from)?;
            Ok::<(), Error>(())
        }, 15).await?;

        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_builder_edge_case_special_characters() -> Result<(), Error> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let mut builder = TestBuilder::new();

        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            builder.test_edge_case_special_characters(&display).await.map_err(anyhow::Error::from)?;
            Ok::<(), Error>(())
        }, 15).await?;

        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_builder_edge_case_unicode_characters() -> Result<(), Error> {
        // Create display OUTSIDE timeout
        let display = ProgressDisplay::new().await?;
        let mut builder = TestBuilder::new();

        // Run test logic INSIDE timeout
        let _ = with_timeout(async {
            builder.test_edge_case_unicode_characters(&display).await.map_err(anyhow::Error::from)?;
            Ok::<(), Error>(())
        }, 15).await?;

        // Clean up OUTSIDE timeout
        display.stop().await?;
        Ok(())
    }
}

#[tokio::test]
async fn test_builder_window_mode() -> Result<(), Error> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut builder = TestBuilder::new().window_mode(5);

    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test window features with multiple lines
        builder.test_window_features(&[
            "First line",
            "Second line",
            "Third line",
            "Fourth line",
            "Fifth line",
        ]).await?;
        Ok::<(), Error>(())
    }, 60).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

// Test is commented out as WindowWithTitle mode is not implemented yet
// #[tokio::test]
// async fn test_builder_window_with_title() -> Result<(), Error> {
//     with_timeout(async {
//         // Create a TestBuilder for WindowWithTitle mode with 5 lines (4 + title)
//         let mut builder = TestBuilder::new().window_with_title_mode(5);
//         
//         // Test window with title features
//         let display = builder.test_window_with_title_features("Window Title", &[
//             "First content line",
//             "Second content line",
//             "Third content line",
//             "Fourth content line",
//         ]).await?;
//         
//         display.stop().await?;
//         
//         Ok(())
//     }, 60).await?
// }

#[tokio::test]
async fn test_builder_limited_mode() -> Result<(), Error> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut builder = TestBuilder::new().limited_mode();

    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test limited mode features (only last message is shown)
        builder.test_limited_features(&[
            "First message (will be overwritten)",
            "Second message (will be overwritten)",
            "Third message (will be displayed)",
        ]).await?;
        Ok::<(), Error>(())
    }, 60).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_builder_capturing_mode() -> Result<(), Error> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut builder = TestBuilder::new().capturing_mode();

    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test capturing mode features (messages are captured but not displayed)
        builder.test_capturing_features(&[
            "First captured message",
            "Second captured message",
            "Third captured message",
        ]).await?;
        Ok::<(), Error>(())
    }, 60).await?;

    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

// Test is commented out as total jobs support is not implemented yet
// #[tokio::test]
// async fn test_builder_progress_update() -> Result<(), Error> {
//     // Create display OUTSIDE timeout
//     let display = ProgressDisplay::new().await?;
//     let mut builder = TestBuilder::new();
//
//     // Run test logic INSIDE timeout
//     let _ = with_timeout(async {
//         // Test progress updates with 3 jobs, 2 messages per job
//         builder.test_progress_update(3, 2).await?;
//         Ok::<(), Error>(())
//     }, 60).await?;
//
//     // Clean up OUTSIDE timeout
//     display.stop().await?;
//     Ok(())
// }

#[tokio::test]
async fn test_builder_terminal_size_customization() -> Result<(), anyhow::Error> {
    // Create a TestBuilder with a custom terminal size
    let mut builder = TestBuilder::with_size(60, 20);
    
    // Test that the terminal size is correctly set
    assert_eq!(builder.terminal_size(), (60, 20));
    
    // Test resizing the terminal
    builder = builder.resize(80, 25);
    assert_eq!(builder.terminal_size(), (80, 25));
    
    // Create a display and test with it
    let _display = builder.build_display().await;
    
    // Run the terminal size detection test
    let size_detected = builder.test_terminal_size_detection(80, 25).await?;
    assert!(size_detected, "Terminal size detection failed");
    
    // Test resize handling
    let message = "Test message for resize";
    let display = builder.test_resize_handling(80, 25, 100, 30, message).await?;
    
    // Clean up
    display.stop().await?;
    Ok(())
} 