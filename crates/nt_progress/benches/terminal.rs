use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use nt_progress::ProgressDisplay;
use nt_progress::modes::ThreadMode;
use std::io::{stdout, Write};
use tokio::runtime::Runtime;
use std::time::Duration;

fn bench_terminal_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("terminal_operations");
    
    // Benchmark window mode with different sizes
    for size in [1, 5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::new("window_mode", size), size, |b, &size| {
            b.to_async(Runtime::new().unwrap()).iter(|| async {
                let mut display = ProgressDisplay::new().await;
                let handle = display.spawn_with_mode(ThreadMode::Window(size), || "bench-task").await.unwrap();
                
                for i in 0..100 {
                    writeln!(stdout(), "Line {}", i).unwrap();
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                
                display.display().await.unwrap();
                display.stop().await.unwrap();
            });
        });
    }
    
    // Benchmark concurrent operations
    group.bench_function("concurrent_operations", |b| {
        b.to_async(Runtime::new().unwrap()).iter(|| async {
            let mut display = ProgressDisplay::new().await;
            let num_threads = 5;
            let mut handles = vec![];
            
            for i in 0..num_threads {
                let mut display = display.clone();
                handles.push(tokio::spawn(async move {
                    display.spawn_with_mode(ThreadMode::Window(5), move || format!("thread-{}", i)).await.unwrap();
                    for j in 0..20 {
                        writeln!(stdout(), "Thread {}: Line {}", i, j).unwrap();
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                }));
            }
            
            for handle in handles {
                handle.await.unwrap();
            }
            
            display.display().await.unwrap();
            display.stop().await.unwrap();
        });
    });
    
    // Benchmark special character handling
    group.bench_function("special_characters", |b| {
        b.to_async(Runtime::new().unwrap()).iter(|| async {
            let mut display = ProgressDisplay::new().await;
            let handle = display.spawn_with_mode(ThreadMode::Window(5), || "special-chars").await.unwrap();
            
            let special_chars = [
                "Test with \n newlines \t tabs \r returns",
                "Test with unicode: 你好世界",
                "Test with emoji: 🚀 ✨",
            ];
            
            for text in special_chars.iter() {
                writeln!(stdout(), "{}", text).unwrap();
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            
            display.display().await.unwrap();
            display.stop().await.unwrap();
        });
    });
    
    // Benchmark long line handling
    group.bench_function("long_lines", |b| {
        b.to_async(Runtime::new().unwrap()).iter(|| async {
            let mut display = ProgressDisplay::new().await;
            let handle = display.spawn_with_mode(ThreadMode::Window(5), || "long-lines").await.unwrap();
            
            let long_line = "x".repeat(1000);
            writeln!(stdout(), "{}", long_line).unwrap();
            
            display.display().await.unwrap();
            display.stop().await.unwrap();
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_terminal_operations);
criterion_main!(benches); 