//! Progress bar visualization components for nt_progress.
//!
//! This module provides flexible progress bar visualization components that
//! can be used with the progress tracking capabilities of nt_progress.
//! It supports various styles, customization options, and integration with
//! existing display modes.

use std::time::{Duration, Instant};
use crate::errors::ProgressError;
use crate::formatter::{ProgressIndicator, CustomIndicatorType};
use std::str::FromStr;

/// The style of the progress bar
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ProgressBarStyle {
    /// Standard ASCII progress bar (e.g. "[====    ]")
    #[default]
    Standard,
    /// Unicode block characters (e.g. "██████    ")
    Block,
    /// Braille pattern characters for a smoother appearance
    Braille,
    /// Dots pattern with partial fill characters
    Dots,
    /// ASCII with color gradient
    Gradient,
}

impl FromStr for ProgressBarStyle {
    type Err = ProgressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" => Ok(ProgressBarStyle::Standard),
            "block" => Ok(ProgressBarStyle::Block),
            "braille" => Ok(ProgressBarStyle::Braille),
            "dots" => Ok(ProgressBarStyle::Dots),
            "gradient" => Ok(ProgressBarStyle::Gradient),
            _ => Err(ProgressError::DisplayOperation(
                format!("Invalid progress bar style: {}", s)
            )),
        }
    }
}

impl From<ProgressBarStyle> for ProgressIndicator {
    fn from(style: ProgressBarStyle) -> Self {
        match style {
            ProgressBarStyle::Standard => ProgressIndicator::Bar,
            ProgressBarStyle::Block => ProgressIndicator::Block,
            ProgressBarStyle::Braille => ProgressIndicator::Custom("braille".to_string()),
            ProgressBarStyle::Dots => ProgressIndicator::Custom("dots".to_string()),
            ProgressBarStyle::Gradient => ProgressIndicator::Custom("gradient".to_string()),
        }
    }
}

impl From<ProgressBarStyle> for Option<CustomIndicatorType> {
    fn from(style: ProgressBarStyle) -> Self {
        match style {
            ProgressBarStyle::Braille => Some(CustomIndicatorType::Braille),
            ProgressBarStyle::Dots => Some(CustomIndicatorType::Dots),
            ProgressBarStyle::Gradient => Some(CustomIndicatorType::Gradient),
            _ => None,
        }
    }
}

/// Configuration for a progress bar display
#[derive(Debug, Clone)]
pub struct ProgressBarConfig {
    /// The style of the progress bar
    pub style: ProgressBarStyle,
    /// The width of the progress bar in characters
    pub width: usize,
    /// Whether to show percentage
    pub show_percentage: bool,
    /// Whether to show the fraction (e.g. "5/10")
    pub show_fraction: bool,
    /// The prefix to display before the progress bar
    pub prefix: Option<String>,
    /// The template for formatting the progress display
    pub template: Option<String>,
    /// Characters to use for the filled portion (default depends on style)
    pub fill_char: Option<char>,
    /// Characters to use for the empty portion (default depends on style)
    pub empty_char: Option<char>,
}

impl Default for ProgressBarConfig {
    fn default() -> Self {
        Self {
            style: ProgressBarStyle::default(),
            width: 20,
            show_percentage: true,
            show_fraction: true,
            prefix: None,
            template: None,
            fill_char: None,
            empty_char: None,
        }
    }
}

impl ProgressBarConfig {
    /// Create a new progress bar config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the style of the progress bar
    pub fn style(mut self, style: ProgressBarStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the width of the progress bar
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set whether to show percentage
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set whether to show the fraction
    pub fn show_fraction(mut self, show: bool) -> Self {
        self.show_fraction = show;
        self
    }

    /// Set a prefix to display before the progress bar
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set a custom template for formatting
    pub fn template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }
    
    /// Set custom characters for filled and empty portions
    pub fn chars(mut self, fill: char, empty: char) -> Self {
        self.fill_char = Some(fill);
        self.empty_char = Some(empty);
        self
    }

    /// Create a template string based on the current configuration
    pub fn build_template(&self) -> String {
        if let Some(template) = &self.template {
            return template.clone();
        }

        let mut parts = Vec::new();

        // Add prefix if any
        if let Some(prefix) = &self.prefix {
            parts.push(prefix.to_string());
        }

        // Add percentage if enabled
        if self.show_percentage {
            parts.push("{progress:percent}".to_string());
        }

        // Add the bar with appropriate style and width
        let style_name = match self.style {
            ProgressBarStyle::Standard => "bar",
            ProgressBarStyle::Block => "block",
            ProgressBarStyle::Braille => "custom:braille",
            ProgressBarStyle::Dots => "custom:dots",
            ProgressBarStyle::Gradient => "custom:gradient",
        };

        // Add custom characters if specified
        let char_params = match (self.fill_char, self.empty_char) {
            (Some(fill), Some(empty)) => format!("{}:{}:{}", style_name, fill, empty),
            _ => style_name.to_string(),
        };

        parts.push(format!("{{progress:bar:{}:{}}}", char_params, self.width));

        // Add fraction if enabled
        if self.show_fraction {
            parts.push("{completed}/{total}".to_string());
        }

        parts.join(" ")
    }
}

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
    fn test_progress_bar_style_from_str() {
        assert_eq!(ProgressBarStyle::from_str("standard").unwrap(), ProgressBarStyle::Standard);
        assert_eq!(ProgressBarStyle::from_str("BLOCK").unwrap(), ProgressBarStyle::Block);
        assert_eq!(ProgressBarStyle::from_str("braille").unwrap(), ProgressBarStyle::Braille);
        assert_eq!(ProgressBarStyle::from_str("dots").unwrap(), ProgressBarStyle::Dots);
        assert_eq!(ProgressBarStyle::from_str("gradient").unwrap(), ProgressBarStyle::Gradient);
        
        assert!(ProgressBarStyle::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_progress_bar_config_defaults() {
        let config = ProgressBarConfig::default();
        assert_eq!(config.style, ProgressBarStyle::Standard);
        assert_eq!(config.width, 20);
        assert!(config.show_percentage);
        assert!(config.show_fraction);
        assert!(config.prefix.is_none());
        assert!(config.template.is_none());
    }
    
    #[test]
    fn test_progress_bar_config_builder() {
        let config = ProgressBarConfig::new()
            .style(ProgressBarStyle::Block)
            .width(30)
            .show_percentage(false)
            .show_fraction(true)
            .prefix("Loading")
            .chars('#', ' ');
        
        assert_eq!(config.style, ProgressBarStyle::Block);
        assert_eq!(config.width, 30);
        assert!(!config.show_percentage);
        assert!(config.show_fraction);
        assert_eq!(config.prefix, Some("Loading".to_string()));
        assert_eq!(config.fill_char, Some('#'));
        assert_eq!(config.empty_char, Some(' '));
    }
    
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
    
    #[test]
    fn test_build_template() {
        // Default template
        let config = ProgressBarConfig::default();
        assert_eq!(config.build_template(), "{progress:percent} {progress:bar:bar:20} {completed}/{total}");
        
        // Without percentage
        let config = ProgressBarConfig::new().show_percentage(false);
        assert_eq!(config.build_template(), "{progress:bar:bar:20} {completed}/{total}");
        
        // Without fraction
        let config = ProgressBarConfig::new().show_fraction(false);
        assert_eq!(config.build_template(), "{progress:percent} {progress:bar:bar:20}");
        
        // With prefix
        let config = ProgressBarConfig::new().prefix("Loading");
        assert_eq!(config.build_template(), "Loading {progress:percent} {progress:bar:bar:20} {completed}/{total}");
        
        // Custom style
        let config = ProgressBarConfig::new().style(ProgressBarStyle::Block);
        assert_eq!(config.build_template(), "{progress:percent} {progress:bar:block:20} {completed}/{total}");
        
        // Custom template overrides all
        let config = ProgressBarConfig::new().template("{task}: {progress:bar:bar:10} ({progress:percent})");
        assert_eq!(config.build_template(), "{task}: {progress:bar:bar:10} ({progress:percent})");
    }
} 