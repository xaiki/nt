use std::time::{Duration, Instant};
use super::config::ProgressBarConfig;

/// A progress bar that tracks progress and allows for customized display
#[derive(Debug, Clone)]
pub struct ProgressBar {
    /// The current progress value (0.0 to 1.0)
    progress: f64,
    /// The configuration for this progress bar
    config: ProgressBarConfig,
    /// The time when the progress bar was created
    start_time: Instant,
    /// The time of the last update
    last_update: Instant,
    /// Optional estimated time to completion
    eta: Option<Duration>,
    /// Optional speed measurement (units per second)
    speed: Option<f64>,
    /// The current value (numerator)
    current: usize,
    /// The total value (denominator)
    total: usize,
}

impl ProgressBar {
    /// Create a new progress bar with the specified configuration
    pub fn new(config: ProgressBarConfig) -> Self {
        let now = Instant::now();
        Self {
            progress: 0.0,
            config,
            start_time: now,
            last_update: now,
            eta: None,
            speed: None,
            current: 0,
            total: 100,
        }
    }

    /// Create a new progress bar with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ProgressBarConfig::default())
    }

    /// Update the progress bar with a new progress value between 0.0 and 1.0
    pub fn update(&mut self, progress: f64) -> &mut Self {
        let now = Instant::now();
        let progress = progress.clamp(0.0, 1.0);
        
        // Only update timing calculations if progress has increased
        if progress > self.progress {
            // We don't need elapsed here, so we'll drop it
            let delta_progress = progress - self.progress;
            let delta_time = now.duration_since(self.last_update);
            
            // Update speed (units per second) if we have a time delta
            if !delta_time.is_zero() {
                let speed = delta_progress / delta_time.as_secs_f64();
                self.speed = Some(speed);
                
                // Estimate time to completion if we have a positive speed
                if speed > 0.0 {
                    let remaining_progress = 1.0 - progress;
                    let remaining_seconds = remaining_progress / speed;
                    self.eta = Some(Duration::from_secs_f64(remaining_seconds));
                }
            }
            
            self.last_update = now;
        }
        
        self.progress = progress;
        self
    }

    /// Update the progress bar with current and total values
    pub fn update_with_values(&mut self, current: usize, total: usize) -> &mut Self {
        self.current = current;
        self.total = total.max(1); // Prevent division by zero
        let progress = (current as f64) / (total as f64);
        self.update(progress)
    }

    /// Get the current progress as a value between 0.0 and 1.0
    pub fn progress(&self) -> f64 {
        self.progress
    }

    /// Get the progress as a percentage value between 0 and 100
    pub fn percentage(&self) -> usize {
        (self.progress * 100.0) as usize
    }

    /// Get the elapsed time since the progress bar was created
    pub fn elapsed(&self) -> Duration {
        Instant::now().duration_since(self.start_time)
    }

    /// Get the estimated time to completion, if available
    pub fn eta(&self) -> Option<Duration> {
        self.eta
    }

    /// Get the current speed in units per second, if available
    pub fn speed(&self) -> Option<f64> {
        self.speed
    }
    
    /// Get the template for this progress bar
    pub fn template(&self) -> String {
        self.config.build_template()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_progress_bar_update() {
        let config = ProgressBarConfig::default();
        let mut bar = ProgressBar::new(config);
        
        assert_eq!(bar.progress(), 0.0);
        assert_eq!(bar.percentage(), 0);
        
        bar.update(0.5);
        assert_eq!(bar.progress(), 0.5);
        assert_eq!(bar.percentage(), 50);
        
        // Test clamping
        bar.update(1.5);
        assert_eq!(bar.progress(), 1.0);
        assert_eq!(bar.percentage(), 100);
        
        bar.update(-0.5);
        assert_eq!(bar.progress(), 0.0);
        assert_eq!(bar.percentage(), 0);
    }
    
    #[test]
    fn test_progress_bar_update_with_values() {
        let mut bar = ProgressBar::with_defaults();
        
        bar.update_with_values(5, 10);
        assert_eq!(bar.progress(), 0.5);
        assert_eq!(bar.percentage(), 50);
        assert_eq!(bar.current, 5);
        assert_eq!(bar.total, 10);
        
        // Test with zero total (should use 1 to prevent division by zero)
        bar.update_with_values(5, 0);
        assert_eq!(bar.total, 1);
    }
} 