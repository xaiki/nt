use crate::ProgressDisplay;
use crate::modes::ThreadMode;
use crate::terminal::TestEnv;

#[tokio::test]
async fn test_terminal_basic() {
    let mut env = TestEnv::new(80, 24);
    let display = ProgressDisplay::new().await;
    
    // Test basic terminal operations
    display.spawn_with_mode(ThreadMode::Limited, || "basic-test").await.unwrap();
    env.writeln("Test line 1");
    env.writeln("Test line 2");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_terminal_resize() {
    let mut env = TestEnv::new(80, 24);
    let display = ProgressDisplay::new().await;
    
    // Test terminal resize handling
    let thread = display.spawn(|| "resize-test").await.unwrap();
    
    display.terminal.set_size(40, 12).await.expect("Failed to set terminal size");
    
    env.writeln("Initial size");
    env.writeln("After resize");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_terminal_clear() {
    let mut env = TestEnv::new(80, 24);
    let display = ProgressDisplay::new().await;
    
    // Test terminal clear
    display.spawn_with_mode(ThreadMode::Limited, || "clear-test").await.unwrap();
    env.writeln("Line 1");
    env.writeln("Line 2");
    env.clear();
    env.writeln("After clear");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_terminal_cursor() {
    let mut env = TestEnv::new(80, 24);
    let display = ProgressDisplay::new().await;
    
    // Test cursor movement
    display.spawn_with_mode(ThreadMode::Limited, || "cursor-test").await.unwrap();
    env.writeln("Line 1");
    env.move_to(10, 5);
    env.writeln("At position");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
}

#[tokio::test]
async fn test_terminal_colors() {
    let mut env = TestEnv::new(80, 24);
    let display = ProgressDisplay::new().await;
    
    // Test color handling
    display.spawn_with_mode(ThreadMode::Limited, || "color-test").await.unwrap();
    env.set_color(crossterm::style::Color::Red);
    env.writeln("Red text");
    env.reset_styles();
    env.writeln("Normal text");
    
    display.display().await.unwrap();
    display.stop().await.unwrap();
    env.verify();
} 
