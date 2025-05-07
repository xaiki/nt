use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::terminal::TestEnv;
use crate::tests::common::with_timeout;
use crossterm::style::Color;
use crate::terminal::Style;
use crate::terminal::TerminalEvent;
use crate::terminal::KeyData;
use crossterm::event::{KeyCode, KeyModifiers};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use anyhow::Result;

#[tokio::test]
async fn test_terminal_basic() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Limited, || "basic-test".to_string()).await?;
        task.capture_stdout("Test line 1".to_string()).await?;
        task.capture_stdout("Test line 2".to_string()).await?;
        
        env.writeln("Test line 1");
        env.writeln("Test line 2");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_terminal_resize() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn(|_| async move { Ok(()) }).await?;
        
        display.renderer.terminal().set_size(40, 12).await?;
        
        task.capture_stdout("Initial size".to_string()).await?;
        task.capture_stdout("After resize".to_string()).await?;
        
        env.writeln("Initial size");
        env.writeln("After resize");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_terminal_clear() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Limited, || "clear-test".to_string()).await?;
        task.capture_stdout("Line 1".to_string()).await?;
        task.capture_stdout("Line 2".to_string()).await?;
        task.capture_stdout("After clear".to_string()).await?;
        
        env.writeln("Line 1");
        env.writeln("Line 2");
        env.clear();
        env.writeln("After clear");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_terminal_cursor() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Limited, || "cursor-test".to_string()).await?;
        task.capture_stdout("Line 1".to_string()).await?;
        task.capture_stdout("At position".to_string()).await?;
        
        env.writeln("Line 1");
        env.move_to(10, 5);
        env.writeln("At position");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_terminal_colors() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Limited, || "color-test".to_string()).await?;
        
        env.set_color(crossterm::style::Color::Red);
        task.capture_stdout("Red text".to_string()).await?;
        env.writeln("Red text");
        
        env.reset_styles();
        task.capture_stdout("Normal text".to_string()).await?;
        env.writeln("Normal text");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_terminal_event_handling() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test keyboard event handling
        let event_manager = display.renderer.terminal().event_manager();
        let event_count = Arc::new(AtomicUsize::new(0));
        let key_press_count = Arc::new(AtomicUsize::new(0));
        
        // Register a handler to count events
        {
            let event_count = Arc::clone(&event_count);
            let key_press_count = Arc::clone(&key_press_count);
            event_manager.register_handler(move |event| {
                let event_count = Arc::clone(&event_count);
                let key_press_count = Arc::clone(&key_press_count);
                async move {
                    event_count.fetch_add(1, Ordering::SeqCst);
                    if let TerminalEvent::KeyPress(_) = event {
                        key_press_count.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            }).await?;
        }
        
        // Start the event loop
        event_manager.start_event_loop().await?;
        
        // Simulate key press
        let key_a = KeyData {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
            char: Some('a'),
            is_release: false,
        };
        let key_b = KeyData {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::empty(),
            char: Some('b'),
            is_release: false,
        };
        
        event_manager.emit_event(TerminalEvent::KeyPress(key_a)).await?;
        event_manager.emit_event(TerminalEvent::KeyPress(key_b)).await?;
        
        // Wait a bit for the events to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Stop the event loop
        event_manager.stop_event_loop().await?;
        
        // Verify events were received
        assert_eq!(event_count.load(Ordering::SeqCst), 2);
        assert_eq!(key_press_count.load(Ordering::SeqCst), 2);
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_terminal_capability_detection() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        // Test terminal capabilities
        let terminal = display.renderer.terminal();
        
        // Check basic capabilities
        assert!(terminal.supports_keyboard_input());
        
        // Test size detection
        let (width, height) = terminal.size().await;
        assert_eq!(width, 80);
        assert_eq!(height, 24);
        
        // Test style support
        assert!(terminal.supports_color());
        assert!(terminal.supports_cursor_movement());
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_terminal_style_management() -> Result<()> {
    // Create display OUTSIDE timeout
    let display = ProgressDisplay::new().await?;
    let mut env = TestEnv::new();
    
    // Run test logic INSIDE timeout
    let _ = with_timeout(async {
        let mut task = display.spawn_with_mode(ThreadMode::Limited, || "style-test".to_string()).await?;
        
        // Test style application
        let mut style = Style::new();
        style.foreground(Color::Red)
             .bold()
             .italic();
        
        // Apply style
        env.apply_style(&style);
        task.capture_stdout("Styled text".to_string()).await?;
        env.writeln("Styled text");
        
        // Reset style
        env.reset_styles();
        task.capture_stdout("Normal text".to_string()).await?;
        env.writeln("Normal text");
        
        display.display().await?;
        env.verify();
        Ok::<(), anyhow::Error>(())
    }, 15).await?;
    
    // Clean up OUTSIDE timeout
    display.stop().await?;
    Ok(())
} 
