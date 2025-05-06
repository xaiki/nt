use super::test_builder::{TestBuilder, EdgeCaseType};
use anyhow::Result;

#[tokio::test]
async fn test_builder_basic_message() -> Result<()> {
    // Create a new TestBuilder with default terminal size (80x24)
    let mut builder = TestBuilder::new();
    
    // Test a simple message with Limited mode (default)
    let display = builder.test_message("Hello, world!").await?;
    display.stop().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_builder_window_mode() -> Result<()> {
    // Create a TestBuilder for Window mode with 5 lines
    let mut builder = TestBuilder::new().window_mode(5);
    
    // Test window features with multiple lines
    let display = builder.test_window_features(&[
        "First line",
        "Second line",
        "Third line",
        "Fourth line",
        "Fifth line",
    ]).await?;
    
    display.stop().await?;
    
    Ok(())
}

// Test is commented out as WindowWithTitle mode is not implemented yet
// #[tokio::test]
// async fn test_builder_window_with_title() -> Result<()> {
//     // Create a TestBuilder for WindowWithTitle mode with 5 lines (4 + title)
//     let mut builder = TestBuilder::new().window_with_title_mode(5);
//     
//     // Test window with title features
//     let display = builder.test_window_with_title_features("Window Title", &[
//         "First content line",
//         "Second content line",
//         "Third content line",
//         "Fourth content line",
//     ]).await?;
//     
//     display.stop().await?;
//     
//     Ok(())
// }

#[tokio::test]
async fn test_builder_limited_mode() -> Result<()> {
    // Create a TestBuilder for Limited mode
    let mut builder = TestBuilder::new().limited_mode();
    
    // Test limited mode features (only last message is shown)
    let display = builder.test_limited_features(&[
        "First message (will be overwritten)",
        "Second message (will be overwritten)",
        "Third message (will be displayed)",
    ]).await?;
    
    display.stop().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_builder_capturing_mode() -> Result<()> {
    // Create a TestBuilder for Capturing mode
    let mut builder = TestBuilder::new().capturing_mode();
    
    // Test capturing mode features (messages are captured but not displayed)
    let display = builder.test_capturing_features(&[
        "First captured message",
        "Second captured message",
        "Third captured message",
    ]).await?;
    
    display.stop().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_builder_edge_cases() -> Result<()> {
    // Create a TestBuilder for edge case testing
    let mut builder = TestBuilder::new();
    
    // Test various edge cases
    let display = builder.test_edge_case(EdgeCaseType::EmptyMessage).await?;
    display.stop().await?;
    
    let display = builder.test_edge_case(EdgeCaseType::LongMessage(100)).await?;
    display.stop().await?;
    
    let display = builder.test_edge_case(EdgeCaseType::SpecialCharacters).await?;
    display.stop().await?;
    
    let display = builder.test_edge_case(EdgeCaseType::UnicodeCharacters).await?;
    display.stop().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_builder_concurrent_tasks() -> Result<()> {
    // Create a TestBuilder for concurrent task testing
    let mut builder = TestBuilder::new();
    
    // Test with 5 concurrent tasks
    let display = builder.test_concurrent_tasks(5, "Task {} output").await?;
    display.stop().await?;
    
    Ok(())
}

// Test is commented out as total jobs support is not implemented yet
// #[tokio::test]
// async fn test_builder_progress_update() -> Result<()> {
//     // Create a TestBuilder for progress updating
//     let mut builder = TestBuilder::new();
//     
//     // Test progress updates with 3 jobs, 2 messages per job
//     let display = builder.test_progress_update(3, 2).await?;
//     display.stop().await?;
//     
//     Ok(())
// } 