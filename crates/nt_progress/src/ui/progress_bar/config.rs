use super::style::ProgressBarStyle;

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
    /// Optional spinner to display before the progress bar
    pub spinner: Option<String>,
    /// Color for the filled part of the progress bar
    pub fill_color: Option<String>,
    /// Color for the empty part of the progress bar 
    pub empty_color: Option<String>,
    /// Color for the percentage text
    pub percentage_color: Option<String>,
    /// Color for the fraction text
    pub fraction_color: Option<String>,
    /// Whether to use a spinner indicator before the progress bar
    pub use_spinner: bool,
    /// Whether to show ETA
    pub show_eta: bool,
    /// Whether to show speed
    pub show_speed: bool,
    /// Format for displaying the ETA (e.g., "ETA: {eta}")
    pub eta_format: Option<String>,
    /// Format for displaying the speed (e.g., "{speed}/s")
    pub speed_format: Option<String>,
    /// The speed unit (default is "items")
    pub speed_unit: String,
    /// Left bracket character for the progress bar
    pub left_bracket: Option<char>,
    /// Right bracket character for the progress bar
    pub right_bracket: Option<char>,
    /// Whether to use a smooth animation effect
    pub smooth_animation: bool,
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
            spinner: None,
            fill_color: None,
            empty_color: None, 
            percentage_color: None,
            fraction_color: None,
            use_spinner: false,
            show_eta: false,
            show_speed: false,
            eta_format: Some("ETA: {eta}".to_string()),
            speed_format: Some("{speed} {unit}/s".to_string()),
            speed_unit: "items".to_string(),
            left_bracket: None,
            right_bracket: None,
            smooth_animation: false,
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

    /// Set the color for the filled part of the progress bar
    pub fn fill_color(mut self, color: impl Into<String>) -> Self {
        self.fill_color = Some(color.into());
        self
    }

    /// Set the color for the empty part of the progress bar
    pub fn empty_color(mut self, color: impl Into<String>) -> Self {
        self.empty_color = Some(color.into());
        self
    }

    /// Set the color for the percentage text
    pub fn percentage_color(mut self, color: impl Into<String>) -> Self {
        self.percentage_color = Some(color.into());
        self
    }

    /// Set the color for the fraction text
    pub fn fraction_color(mut self, color: impl Into<String>) -> Self {
        self.fraction_color = Some(color.into());
        self
    }

    /// Set whether to use a spinner indicator before the progress bar
    pub fn use_spinner(mut self, use_spinner: bool) -> Self {
        self.use_spinner = use_spinner;
        self
    }

    /// Set a specific spinner type to use
    pub fn spinner(mut self, spinner: impl Into<String>) -> Self {
        self.spinner = Some(spinner.into());
        self.use_spinner = true;
        self
    }

    /// Set whether to show estimated time to completion
    pub fn show_eta(mut self, show: bool) -> Self {
        self.show_eta = show;
        self
    }

    /// Set whether to show progress speed
    pub fn show_speed(mut self, show: bool) -> Self {
        self.show_speed = show;
        self
    }

    /// Set the format for displaying ETA
    pub fn eta_format(mut self, format: impl Into<String>) -> Self {
        self.eta_format = Some(format.into());
        self.show_eta = true;
        self
    }

    /// Set the format for displaying speed
    pub fn speed_format(mut self, format: impl Into<String>) -> Self {
        self.speed_format = Some(format.into());
        self.show_speed = true;
        self
    }

    /// Set the unit for speed measurements
    pub fn speed_unit(mut self, unit: impl Into<String>) -> Self {
        self.speed_unit = unit.into();
        self
    }

    /// Set custom bracket characters for the progress bar
    pub fn brackets(mut self, left: char, right: char) -> Self {
        self.left_bracket = Some(left);
        self.right_bracket = Some(right);
        self
    }

    /// Set whether to use smooth animation effect
    pub fn smooth_animation(mut self, smooth: bool) -> Self {
        self.smooth_animation = smooth;
        self
    }

    /// Create a template string based on the current configuration
    pub fn build_template(&self) -> String {
        if let Some(template) = &self.template {
            return template.clone();
        }

        let mut parts = Vec::new();

        // Add spinner if enabled
        if self.use_spinner {
            let spinner_type = self.spinner.as_deref().unwrap_or("dots");
            parts.push(format!("{{spinner:{}}}", spinner_type));
        }

        // Add prefix if any
        if let Some(prefix) = &self.prefix {
            parts.push(prefix.clone());
        }

        // Add percentage if enabled with optional color
        if self.show_percentage {
            let percentage = if let Some(color) = &self.percentage_color {
                format!("{{progress:percent:{}}}", color)
            } else {
                "{progress:percent}".to_string()
            };
            parts.push(percentage);
        }

        // Add the bar with appropriate style and width
        let style_name = match self.style {
            ProgressBarStyle::Standard => "bar",
            ProgressBarStyle::Block => "block",
            ProgressBarStyle::Braille => "custom:braille",
            ProgressBarStyle::Dots => "custom:dots",
            ProgressBarStyle::Gradient => "custom:gradient",
        };

        // Start building the bar params
        let mut bar_params = vec![style_name.to_string()];
        
        // Add custom characters if specified
        if let Some(fill) = self.fill_char {
            bar_params.push(fill.to_string());
            if let Some(empty) = self.empty_char {
                bar_params.push(empty.to_string());
            }
        }
        
        // Add custom colors if specified
        if let Some(fill_color) = &self.fill_color {
            if bar_params.len() == 1 {
                // Need to add placeholder characters first
                match self.style {
                    ProgressBarStyle::Standard => {
                        bar_params.push("=".to_string());
                        bar_params.push(" ".to_string());
                    },
                    ProgressBarStyle::Block => {
                        bar_params.push("█".to_string());
                        bar_params.push(" ".to_string());
                    },
                    _ => {
                        // For custom styles, we use defaults
                        bar_params.push("■".to_string());
                        bar_params.push("□".to_string());
                    }
                }
            }
            bar_params.push(fill_color.clone());
            
            if let Some(empty_color) = &self.empty_color {
                bar_params.push(empty_color.clone());
            }
        }
        
        // Add brackets if specified
        if let Some(left) = self.left_bracket {
            bar_params.push(left.to_string());
            if let Some(right) = self.right_bracket {
                bar_params.push(right.to_string());
            } else {
                bar_params.push("]".to_string()); // Default right bracket
            }
        }
        
        // Add smooth animation if enabled
        if self.smooth_animation {
            bar_params.push("smooth".to_string());
        }
        
        parts.push(format!("{{progress:bar:{}:{}}}", bar_params.join(":"), self.width));

        // Add fraction if enabled with optional color
        if self.show_fraction {
            let fraction = if let Some(color) = &self.fraction_color {
                format!("{{completed:{}}}/{{total:{}}}", color, color)
            } else {
                "{completed}/{total}".to_string()
            };
            parts.push(fraction);
        }
        
        // Add ETA if enabled
        if self.show_eta {
            let eta_format = self.eta_format.as_deref().unwrap_or("ETA: {eta}");
            parts.push(eta_format.to_string());
        }
        
        // Add speed if enabled
        if self.show_speed {
            let speed_format = self.speed_format.as_deref().unwrap_or("{speed} {unit}/s");
            let formatted = speed_format
                .replace("{unit}", &self.speed_unit);
            parts.push(formatted);
        }

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
    
    #[test]
    fn test_enhanced_progress_bar_customization() {
        // Test basic customization
        let config = ProgressBarConfig::new()
            .style(ProgressBarStyle::Block)
            .width(30)
            .show_percentage(true)
            .show_fraction(true)
            .prefix("Progress: ");
            
        let template = config.build_template();
        assert!(template.contains("Progress: "));
        assert!(template.contains("{progress:percent}"));
        assert!(template.contains("{progress:bar:block:30}"));
        assert!(template.contains("{completed}/{total}"));
        
        // Test advanced customization
        let config = ProgressBarConfig::new()
            .style(ProgressBarStyle::Braille)
            .width(25)
            .fill_color("green")
            .empty_color("gray")
            .percentage_color("cyan")
            .fraction_color("yellow")
            .show_eta(true)
            .show_speed(true)
            .speed_unit("bytes")
            .brackets('[', ']')
            .smooth_animation(true);
            
        let template = config.build_template();
        
        // Check for colors
        assert!(template.contains("{progress:percent:cyan}"));
        assert!(template.contains("{completed:yellow}/{total:yellow}"));
        
        // Check for bar customization with colors and brackets
        assert!(template.contains("custom:braille:■:□:green:gray:[:]"));
        assert!(template.contains(":smooth:25"));
        
        // Check for ETA and speed
        assert!(template.contains("ETA: {eta}"));
        assert!(template.contains("bytes/s"));
    }
    
    #[test]
    fn test_color_formatter_integration() {
        // Test color integration
        let config = ProgressBarConfig::new()
            .style(ProgressBarStyle::Standard)
            .chars('#', '-')
            .fill_color("blue")
            .empty_color("white");
            
        let template = config.build_template();
        assert!(template.contains("bar:#:-:blue:white"));
        
        // Test brackets with colors
        let config = ProgressBarConfig::new()
            .style(ProgressBarStyle::Block)
            .fill_color("red")
            .empty_color("black")
            .brackets('{', '}');
            
        let template = config.build_template();
        assert!(template.contains("block:█: :red:black:{:}"));
    }
    
    #[test]
    fn test_eta_and_speed_formatting() {
        // Test ETA formatting
        let config = ProgressBarConfig::new()
            .show_eta(true)
            .eta_format("Remaining: {eta}");
            
        let template = config.build_template();
        assert!(template.contains("Remaining: {eta}"));
        
        // Test speed formatting
        let config = ProgressBarConfig::new()
            .show_speed(true)
            .speed_format("Rate: {speed} {unit}/sec")
            .speed_unit("MB");
            
        let template = config.build_template();
        assert!(template.contains("Rate: {speed} MB/sec"));
    }
    
    #[test]
    fn test_smooth_animation() {
        // Test smooth animation flag
        let config = ProgressBarConfig::new()
            .style(ProgressBarStyle::Block)
            .smooth_animation(true);
            
        let template = config.build_template();
        assert!(template.contains(":smooth:"));
    }
} 