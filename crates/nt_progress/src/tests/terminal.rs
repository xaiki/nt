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

#[tokio::test]
async fn test_terminal_basic() {
    with_timeout(async {
        let mut env = TestEnv::new();
        let display = ProgressDisplay::new().await.unwrap();
        
        // Test basic terminal operations
        display.spawn_with_mode(ThreadMode::Limited, || "basic-test".to_string()).await.unwrap();
        env.writeln("Test line 1");
        env.writeln("Test line 2");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 60).await.unwrap();
}

#[tokio::test]
async fn test_terminal_resize() {
    with_timeout(async {
        let mut env = TestEnv::new();
        let display = ProgressDisplay::new().await.unwrap();
        
        // Test terminal resize handling
        let _thread = display.spawn(|_| async move { Ok(()) }).await.unwrap();
        
        display.terminal.set_size(40, 12).await.expect("Failed to set terminal size");
        
        env.writeln("Initial size");
        env.writeln("After resize");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 60).await.unwrap();
}

#[tokio::test]
async fn test_terminal_clear() {
    with_timeout(async {
        let mut env = TestEnv::new();
        let display = ProgressDisplay::new().await.unwrap();
        
        // Test terminal clear
        display.spawn_with_mode(ThreadMode::Limited, || "clear-test".to_string()).await.unwrap();
        env.writeln("Line 1");
        env.writeln("Line 2");
        env.clear();
        env.writeln("After clear");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 60).await.unwrap();
}

#[tokio::test]
async fn test_terminal_cursor() {
    with_timeout(async {
        let mut env = TestEnv::new();
        let display = ProgressDisplay::new().await.unwrap();
        
        // Test cursor movement
        display.spawn_with_mode(ThreadMode::Limited, || "cursor-test".to_string()).await.unwrap();
        env.writeln("Line 1");
        env.move_to(10, 5);
        env.writeln("At position");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 60).await.unwrap();
}

#[tokio::test]
async fn test_terminal_colors() {
    with_timeout(async {
        let mut env = TestEnv::new();
        let display = ProgressDisplay::new().await.unwrap();
        
        // Test color handling
        display.spawn_with_mode(ThreadMode::Limited, || "color-test".to_string()).await.unwrap();
        env.set_color(crossterm::style::Color::Red);
        env.writeln("Red text");
        env.reset_styles();
        env.writeln("Normal text");
        
        display.display().await.unwrap();
        display.stop().await.unwrap();
        env.verify();
    }, 60).await.unwrap();
}

#[tokio::test]
async fn test_terminal_event_handling() {
    with_timeout(async {
        let mut env = TestEnv::new();
        let display = ProgressDisplay::new().await.unwrap();
        
        // Test keyboard event handling
        let event_manager = display.terminal.event_manager();
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
            }).await.unwrap();
        }
        
        // Start the event loop
        event_manager.start_event_loop().await.unwrap();
        
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
        
        event_manager.emit_event(TerminalEvent::KeyPress(key_a)).await.unwrap();
        event_manager.emit_event(TerminalEvent::KeyPress(key_b)).await.unwrap();
        
        // Wait a bit for the events to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Stop the event loop
        event_manager.stop_event_loop().await.unwrap();
        
        // Verify events were received
        assert_eq!(event_count.load(Ordering::SeqCst), 2);
        assert_eq!(key_press_count.load(Ordering::SeqCst), 2);
        
        display.stop().await.unwrap();
        env.verify();
    }, 60).await.unwrap();
}

#[tokio::test]
async fn test_terminal_capability_detection() {
    with_timeout(async {
        let mut env = TestEnv::new();
        let display = ProgressDisplay::new().await.unwrap();
        
        // Test terminal capabilities
        let terminal = &display.terminal;
        
        // Check basic capabilities
        assert!(terminal.supports_keyboard_input());
        
        // Test size detection
        let (width, height) = terminal.size().await;
        assert_eq!(width, 80);
        assert_eq!(height, 24);
        
        // Test style support
        assert!(terminal.supports_color());
        assert!(terminal.supports_cursor_movement());
        
        display.stop().await.unwrap();
        env.verify();
    }, 60).await.unwrap();
}

#[tokio::test]
async fn test_terminal_style_management() {
    with_timeout(async {
        let mut env = TestEnv::new();
        let display = ProgressDisplay::new().await.unwrap();
        
        // Test style application
        let mut style = Style::new();
        style.foreground(Color::Red)
             .bold()
             .italic();
        
        // Apply style
        env.apply_style(&style);
        env.writeln("Styled text");
        
        // Reset style
        env.reset_styles();
        env.writeln("Normal text");
        
        display.stop().await.unwrap();
        env.verify();
    }, 60).await.unwrap();
} 
