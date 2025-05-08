//! A template-based message formatting system for progress displays.
//!
//! This module provides a simple yet powerful templating system for formatting task progress
//! messages. It supports variable interpolation, conditional rendering, and basic formatting
//! operations.

use std::collections::HashMap;
use crate::errors::{ProgressError, ContextExt};
use crate::terminal::Color;
use std::str::FromStr;

/// Template variable types that can be interpolated into templates
#[derive(Debug, Clone)]
pub enum TemplateVar {
    /// A string value
    String(String),
    /// A numeric value
    Number(f64),
    /// A boolean value
    Boolean(bool),
}

impl TemplateVar {
    /// Convert the variable to a string representation
    pub fn as_string(&self) -> String {
        match self {
            TemplateVar::String(s) => s.clone(),
            TemplateVar::Number(n) => n.to_string(),
            TemplateVar::Boolean(b) => b.to_string(),
        }
    }
    
    /// Check if the variable has a truthy value
    pub fn is_truthy(&self) -> bool {
        match self {
            TemplateVar::String(s) => !s.is_empty(),
            TemplateVar::Number(n) => *n != 0.0,
            TemplateVar::Boolean(b) => *b,
        }
    }
}

impl From<String> for TemplateVar {
    fn from(s: String) -> Self {
        TemplateVar::String(s)
    }
}

impl From<&str> for TemplateVar {
    fn from(s: &str) -> Self {
        TemplateVar::String(s.to_string())
    }
}

impl From<f64> for TemplateVar {
    fn from(n: f64) -> Self {
        TemplateVar::Number(n)
    }
}

impl From<usize> for TemplateVar {
    fn from(n: usize) -> Self {
        TemplateVar::Number(n as f64)
    }
}

impl From<bool> for TemplateVar {
    fn from(b: bool) -> Self {
        TemplateVar::Boolean(b)
    }
}

/// A container for template variables
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    vars: HashMap<String, TemplateVar>,
}

impl TemplateContext {
    /// Create a new empty template context
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
    
    /// Set a variable in the context
    pub fn set<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<TemplateVar>,
    {
        self.vars.insert(key.into(), value.into());
        self
    }
    
    /// Get a variable from the context
    pub fn get(&self, key: &str) -> Option<&TemplateVar> {
        self.vars.get(key)
    }
    
    /// Check if the context has a variable
    pub fn has(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }
}

/// A template for formatting task progress messages
#[derive(Debug, Clone)]
pub struct ProgressTemplate {
    template: String,
}

impl ProgressTemplate {
    /// Create a new template from a string
    ///
    /// # Parameters
    /// * `template` - The template string
    ///
    /// # Returns
    /// A new ProgressTemplate
    ///
    /// # Template Syntax
    /// Templates use a simple syntax for variable interpolation and conditionals:
    ///
    /// - `{var}` - Interpolate the value of `var`
    /// - `{var:format}` - Interpolate `var` with the specified format
    /// - `{?condition}content{/}` - Include `content` only if `condition` is truthy
    /// - `{!condition}content{/}` - Include `content` only if `condition` is falsy
    ///
    /// # Available Formats
    ///
    /// - `{var:bar}` - Render `var` as a progress bar
    /// - `{var:percent}` - Render `var` as a percentage (e.g., "50%")
    /// - `{var:ratio}` - Render `var` as a ratio (e.g., "5/10")
    /// - `{var:pad:N}` - Pad `var` to length N with spaces
    /// - `{var:lpad:N}` - Left-pad `var` to length N with spaces
    /// - `{var:rpad:N}` - Right-pad `var` to length N with spaces
    /// - `{var:color:name}` - Apply color to `var` (supported colors: black, red, green, yellow, blue, magenta, cyan, white, reset)
    ///
    /// # Examples
    ///
    /// ```
    /// use nt_progress::formatter::{ProgressTemplate, TemplateContext};
    ///
    /// let template = ProgressTemplate::new("Progress: {progress:bar} {progress:percent} ({completed}/{total})");
    /// let mut ctx = TemplateContext::new();
    /// ctx.set("progress", 0.5)
    ///    .set("completed", 5)
    ///    .set("total", 10);
    ///
    /// let output = template.render(&ctx).unwrap();
    /// // Output: "Progress: [=====     ] 50% (5/10)"
    /// ```
    /// 
    /// Using color formatting:
    /// 
    /// ```
    /// use nt_progress::formatter::{ProgressTemplate, TemplateContext};
    /// 
    /// let template = ProgressTemplate::new("Status: {status:color:green}");
    /// let mut ctx = TemplateContext::new();
    /// ctx.set("status", "Success");
    /// 
    /// let output = template.render(&ctx).unwrap();
    /// // Output will show "Status: Success" with "Success" in green
    /// ```
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }
    
    /// Render the template with the given context
    ///
    /// # Parameters
    /// * `context` - The context containing variables for interpolation
    ///
    /// # Returns
    /// A Result containing the rendered string or an error
    pub fn render(&self, context: &TemplateContext) -> Result<String, ProgressError> {
        // Create a buffer for building the result
        let mut result = String::with_capacity(self.template.len() * 2);
        
        // Parse the template and render it
        let chars: Vec<char> = self.template.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            if chars[i] == '{' {
                if i + 1 < chars.len() && chars[i + 1] == '{' {
                    // Escaped opening brace {{ -> {
                    result.push('{');
                    i += 2;
                    continue;
                }
                
                // Look for the closing brace
                let mut j = i + 1;
                while j < chars.len() && chars[j] != '}' {
                    j += 1;
                }
                
                if j < chars.len() {
                    // Found a complete tag
                    let tag = chars[i + 1..j].iter().collect::<String>();
                    
                    if let Some(rendered) = self.render_tag(&tag, context)
                        .with_context("rendering template tag", "ProgressTemplate")? {
                        result.push_str(&rendered);
                    }
                    
                    i = j + 1;
                } else {
                    // Unclosed tag, treat as literal
                    result.push('{');
                    i += 1;
                }
            } else if chars[i] == '}' && i + 1 < chars.len() && chars[i + 1] == '}' {
                // Escaped closing brace }} -> }
                result.push('}');
                i += 2;
            } else {
                // Normal character
                result.push(chars[i]);
                i += 1;
            }
        }
        
        Ok(result)
    }
    
    // Process a single template tag
    fn render_tag(&self, tag: &str, context: &TemplateContext) -> Result<Option<String>, ProgressError> {
        // Check for conditional tag
        if tag.starts_with('?') || tag.starts_with('!') {
            return self.render_conditional_tag(tag, context);
        }
        
        // Regular variable tag
        let parts: Vec<&str> = tag.split(':').collect();
        let var_name = parts[0].trim();
        
        // Get the variable from the context
        if let Some(var) = context.get(var_name) {
            // Check if we have a format specifier
            if parts.len() > 1 {
                self.apply_format(var, &parts[1..], context)
            } else {
                // No format, just convert to string
                Ok(Some(var.as_string()))
            }
        } else {
            // Variable not found, render as empty string
            Ok(Some(String::new()))
        }
    }
    
    // Process a conditional tag {?condition}content{/} or {!condition}content{/}
    fn render_conditional_tag(&self, tag: &str, context: &TemplateContext) -> Result<Option<String>, ProgressError> {
        let (is_positive, condition) = if let Some(stripped) = tag.strip_prefix('?') {
            (true, stripped)
        } else if let Some(stripped) = tag.strip_prefix('!') {
            (false, stripped)
        } else {
            return Ok(None);
        };

        // Get the condition variable
        let condition_value = match context.get(condition) {
            Some(var) => var.is_truthy(),
            None => false,
        };

        // Return Some("") if the condition matches, None if it doesn't
        if condition_value == is_positive {
            Ok(Some(String::new()))
        } else {
            Ok(None)
        }
    }
    
    // Apply a format to a variable
    fn apply_format(
        &self,
        var: &TemplateVar,
        format_parts: &[&str],
        context: &TemplateContext,
    ) -> Result<Option<String>, ProgressError> {
        if format_parts.is_empty() {
            return Ok(Some(var.as_string()));
        }

        let format = format_parts[0];

        match format {
            "bar" => self.format_bar(var, format_parts, context),
            "percent" => self.format_percent(var, format_parts, context),
            "ratio" => self.format_ratio(var, format_parts, context),
            "pad" | "lpad" | "rpad" => self.format_padding(var, format, format_parts, context),
            "color" => self.format_color(var, format_parts, context),
            _ => Ok(Some(var.as_string())),
        }
    }
    
    // Format a variable as a progress bar
    fn format_bar(
        &self,
        var: &TemplateVar,
        format_parts: &[&str],
        _context: &TemplateContext,
    ) -> Result<Option<String>, ProgressError> {
        // Extract the progress value (should be a number between 0 and 1)
        let progress = match var {
            TemplateVar::Number(n) => *n,
            _ => {
                return Err(ProgressError::DisplayOperation(
                    "Progress bar format requires a number".to_string(),
                ))
            }
        };
        
        // Clamp to 0..1 range
        let progress = progress.clamp(0.0, 1.0);
        
        // Determine indicator type
        let indicator_type = if format_parts.len() > 1 {
            // If the format includes an indicator type, use it
            match ProgressIndicator::from_str(format_parts[1]) {
                Ok(indicator) => indicator,
                Err(_) => ProgressIndicator::Bar
            }
        } else {
            // Default to Bar
            ProgressIndicator::Bar
        };
        
        match indicator_type {
            ProgressIndicator::Bar => self.format_bar_indicator(progress, format_parts),
            ProgressIndicator::Block => self.format_block_indicator(progress, format_parts),
            ProgressIndicator::Spinner => self.format_spinner_indicator(progress, format_parts),
            ProgressIndicator::Numeric => self.format_numeric_indicator(progress, format_parts),
            ProgressIndicator::Custom(name) => self.format_custom_indicator(name, progress, format_parts),
        }
    }
    
    /// Format a traditional bar indicator "[====    ]"
    fn format_bar_indicator(
        &self,
        progress: f64,
        format_parts: &[&str],
    ) -> Result<Option<String>, ProgressError> {
        // Get the bar width (default: 10)
        let width = if format_parts.len() > 2 {
            format_parts[2].parse::<usize>().unwrap_or(10)
        } else {
            10
        };
        
        // Get the fill character (default: '=')
        let fill_char = if format_parts.len() > 3 {
            format_parts[3].chars().next().unwrap_or('=')
        } else {
            '='
        };
        
        // Get the background character (default: ' ')
        let bg_char = if format_parts.len() > 4 {
            format_parts[4].chars().next().unwrap_or(' ')
        } else {
            ' '
        };
        
        // Calculate filled portion
        let filled = (width as f64 * progress).round() as usize;
        
        // Build the bar
        let mut result = String::with_capacity(width + 2);
        result.push('[');
        
        for i in 0..width {
            if i < filled {
                result.push(fill_char);
            } else {
                result.push(bg_char);
            }
        }
        
        result.push(']');
        
        Ok(Some(result))
    }
    
    /// Format a block-based indicator using Unicode block characters
    fn format_block_indicator(
        &self,
        progress: f64,
        format_parts: &[&str],
    ) -> Result<Option<String>, ProgressError> {
        // Get the bar width (default: 10)
        let width = if format_parts.len() > 2 {
            format_parts[2].parse::<usize>().unwrap_or(10)
        } else {
            10
        };
        
        // Get the block characters (default: "█▓▒░ ")
        let block_chars = if format_parts.len() > 3 {
            format_parts[3]
        } else {
            ProgressIndicator::default_block_chars()
        };
        
        // Calculate filled portion
        let filled = (width as f64 * progress).round() as usize;
        let blocks = block_chars.chars().collect::<Vec<_>>();
        
        // Handle invalid block chars
        if blocks.is_empty() {
            return Err(ProgressError::DisplayOperation(
                "Block indicator requires at least one character".to_string(),
            ));
        }
        
        // Get fill character (first) and background character (last or space)
        let fill_char = blocks[0];
        let bg_char = if blocks.len() > 1 {
            *blocks.last().unwrap()
        } else {
            ' ' // Default to space if only one character provided
        };
        
        // Build the bar
        let mut result = String::with_capacity(width + 2);
        result.push('[');
        
        for i in 0..width {
            if i < filled {
                result.push(fill_char); // Filled block
            } else {
                result.push(bg_char); // Background character
            }
        }
        
        result.push(']');
        
        Ok(Some(result))
    }
    
    /// Format a spinner indicator that changes based on progress
    fn format_spinner_indicator(
        &self,
        progress: f64,
        format_parts: &[&str],
    ) -> Result<Option<String>, ProgressError> {
        // Get spinner frames
        let frames = if format_parts.len() > 2 {
            format_parts[2].chars().collect::<Vec<_>>()
        } else {
            ProgressIndicator::default_spinner_frames()
                .join("")
                .chars()
                .collect::<Vec<_>>()
        };
        
        // Handle invalid frames
        if frames.is_empty() {
            return Err(ProgressError::DisplayOperation(
                "Spinner indicator requires at least one frame".to_string(),
            ));
        }
        
        // Calculate current frame based on progress
        let frame_index = (progress * frames.len() as f64).floor() as usize % frames.len();
        let frame = frames[frame_index];
        
        Ok(Some(frame.to_string()))
    }
    
    /// Format a simple numeric indicator showing only the percentage
    fn format_numeric_indicator(
        &self,
        progress: f64,
        format_parts: &[&str],
    ) -> Result<Option<String>, ProgressError> {
        // Calculate percentage
        let percent = (progress * 100.0).round() as usize;
        
        // Check if we should include the percent sign
        let include_sign = format_parts.len() <= 2 || format_parts[2] != "false";
        
        if include_sign {
            Ok(Some(format!("{}%", percent)))
        } else {
            Ok(Some(format!("{}", percent)))
        }
    }
    
    // Format a variable as a percentage
    fn format_percent(
        &self,
        var: &TemplateVar,
        _format_parts: &[&str],
        _context: &TemplateContext,
    ) -> Result<Option<String>, ProgressError> {
        // Extract the progress value (should be a number between 0 and 1)
        let progress = match var {
            TemplateVar::Number(n) => *n,
            _ => {
                return Err(ProgressError::DisplayOperation(
                    "Percentage format requires a number".to_string(),
                ))
            }
        };
        
        // Clamp to 0..1 range and convert to percentage
        let percent = (progress.clamp(0.0, 1.0) * 100.0).round() as usize;
        
        Ok(Some(format!("{}%", percent)))
    }
    
    // Format a variable as a ratio (numerator/denominator)
    fn format_ratio(
        &self,
        var: &TemplateVar,
        format_parts: &[&str],
        context: &TemplateContext,
    ) -> Result<Option<String>, ProgressError> {
        // Extract the numerator value
        let numerator = match var {
            TemplateVar::Number(n) => *n as usize,
            _ => {
                return Err(ProgressError::DisplayOperation(
                    "Ratio format requires a number".to_string(),
                ))
            }
        };
        
        // Get the denominator from format or context
        let denominator = if format_parts.len() > 1 {
            // Try to parse directly
            if let Ok(n) = format_parts[1].parse::<usize>() {
                n
            } else {
                // Try to get from context
                match context.get(format_parts[1]) {
                    Some(TemplateVar::Number(n)) => *n as usize,
                    _ => 100, // Default
                }
            }
        } else {
            100 // Default
        };
        
        Ok(Some(format!("{}/{}", numerator, denominator)))
    }
    
    // Format a variable with padding
    fn format_padding(
        &self,
        var: &TemplateVar,
        format: &str,
        format_parts: &[&str],
        _context: &TemplateContext,
    ) -> Result<Option<String>, ProgressError> {
        let text = var.as_string();
        
        // Get the padding width (default: text length)
        let width = if format_parts.len() > 1 {
            format_parts[1].parse::<usize>().unwrap_or(text.len())
        } else {
            text.len()
        };
        
        match format {
            "lpad" => Ok(Some(format!("{:>width$}", text, width = width))),
            "rpad" => Ok(Some(format!("{:<width$}", text, width = width))),
            _ => Ok(Some(format!("{:^width$}", text, width = width))), // Center padding
        }
    }
    
    /// Format a variable with color
    fn format_color(
        &self,
        var: &TemplateVar,
        format_parts: &[&str],
        _context: &TemplateContext,
    ) -> Result<Option<String>, ProgressError> {
        // Extract the text value
        let text = var.as_string();
        
        // Get the color from format
        if format_parts.len() < 2 {
            return Err(ProgressError::DisplayOperation(
                "Color format requires a color name".to_string(),
            ));
        }
        
        let color_name = format_parts[1].trim();
        let color = match ColorName::from_str(color_name) {
            Some(c) => c,
            None => {
                return Err(ProgressError::DisplayOperation(
                    format!("Unknown color: {}", color_name),
                ));
            }
        };
        
        // Convert to crossterm Color and then to ANSI code
        let crossterm_color = color.to_color();
        let color_code = match crossterm_color {
            Color::Black => "\x1B[30m",
            Color::Red => "\x1B[31m",
            Color::Green => "\x1B[32m",
            Color::Yellow => "\x1B[33m", 
            Color::Blue => "\x1B[34m",
            Color::Magenta => "\x1B[35m",
            Color::Cyan => "\x1B[36m",
            Color::White => "\x1B[37m",
            Color::Reset => "\x1B[0m",
            _ => "\x1B[0m", // Default to reset for other colors
        };
        
        // Apply color to text and reset after
        let colored_text = format!("{}{}\x1B[0m", color_code, text);
        
        Ok(Some(colored_text))
    }
    
    /// Format a custom indicator defined by the user
    fn format_custom_indicator(
        &self,
        name: String,
        progress: f64,
        format_parts: &[&str],
    ) -> Result<Option<String>, ProgressError> {
        // Parse the indicator type
        let indicator_type = name.parse::<CustomIndicatorType>().map_err(|_| {
            ProgressError::DisplayOperation(format!(
                "Unknown custom indicator: {}. Valid options are: {}",
                name,
                CustomIndicatorType::variants().join(", ")
            ))
        })?;
        
        // Format based on the indicator type
        match indicator_type {
            CustomIndicatorType::Dots => self.format_dots_indicator(progress, format_parts),
            CustomIndicatorType::Braille => self.format_braille_indicator(progress, format_parts),
            CustomIndicatorType::Gradient => self.format_gradient_indicator(progress, format_parts),
        }
    }
    
    /// Format a dots indicator that uses Unicode dots for a more compact representation
    /// Example: "⣿⣿⣿⣿⣷⣀⣀⣀"
    fn format_dots_indicator(
        &self,
        progress: f64,
        format_parts: &[&str],
    ) -> Result<Option<String>, ProgressError> {
        // Get the bar width (default: 10)
        let width = if format_parts.len() > 3 {
            format_parts[3].parse::<usize>().unwrap_or(10)
        } else {
            10
        };
        
        // Braille dots pattern (8 levels of fill)
        let dots = "⠀⡀⣀⣄⣤⣦⣶⣾⣿";
        let dots_chars: Vec<char> = dots.chars().collect();
        
        // Calculate filled portion
        let filled = (width as f64 * progress).round() as usize;
        let partial_fill = ((width as f64 * progress * 8.0) as usize) % 8;
        
        // Build the dots bar
        let mut result = String::with_capacity(width);
        
        for i in 0..width {
            if i < filled {
                // Fully filled
                result.push(dots_chars[dots_chars.len() - 1]);
            } else if i == filled && partial_fill > 0 {
                // Partially filled character
                result.push(dots_chars[partial_fill]);
            } else {
                // Empty
                result.push(dots_chars[0]);
            }
        }
        
        Ok(Some(result))
    }
    
    /// Format a braille-based indicator that uses Unicode braille patterns
    /// Example: "⣿⣿⣿⣿⣿⠿⠄⠄⠄"
    fn format_braille_indicator(
        &self,
        progress: f64,
        format_parts: &[&str],
    ) -> Result<Option<String>, ProgressError> {
        // Get the bar width (default: 10)
        let width = if format_parts.len() > 3 {
            format_parts[3].parse::<usize>().unwrap_or(10)
        } else {
            10
        };
        
        // Braille patterns (full, partial, empty)
        let full = "⣿";
        let partial = "⠿⠷⠯⠟⠻⠛⠙⠉";
        let empty = "⠄";
        
        let partial_chars: Vec<char> = partial.chars().collect();
        
        // Calculate filled portion
        let filled = (width as f64 * progress).round() as usize;
        let partial_fill = if filled < width {
            ((width as f64 * progress * partial_chars.len() as f64) as usize) % partial_chars.len()
        } else {
            0
        };
        
        // Build the braille bar
        let mut result = String::with_capacity(width);
        
        for i in 0..width {
            if i < filled {
                result.push_str(full);
            } else if i == filled && partial_fill > 0 {
                result.push(partial_chars[partial_fill]);
            } else {
                result.push_str(empty);
            }
        }
        
        Ok(Some(result))
    }
    
    /// Format a gradient indicator that uses a color gradient for a more visually appealing progress bar
    /// Example: "[====    ]" with color gradient
    fn format_gradient_indicator(
        &self,
        progress: f64,
        format_parts: &[&str],
    ) -> Result<Option<String>, ProgressError> {
        use crossterm::style::{Color, Stylize};
        
        // Get the bar width (default: 10)
        let width = if format_parts.len() > 3 {
            format_parts[3].parse::<usize>().unwrap_or(10)
        } else {
            10
        };
        
        // Get colors for gradient (default: red to green)
        let start_color = if format_parts.len() > 4 {
            match format_parts[4] {
                "red" => Color::Red,
                "green" => Color::Green,
                "blue" => Color::Blue,
                "yellow" => Color::Yellow,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                _ => Color::Red,
            }
        } else {
            Color::Red
        };
        
        let end_color = if format_parts.len() > 5 {
            match format_parts[5] {
                "red" => Color::Red,
                "green" => Color::Green,
                "blue" => Color::Blue,
                "yellow" => Color::Yellow,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                _ => Color::Green,
            }
        } else {
            Color::Green
        };
        
        // Calculate filled portion
        let filled = (width as f64 * progress).round() as usize;
        
        // Build the bar with gradient
        let mut result = String::new();
        result.push('[');
        
        for i in 0..width {
            let char = if i < filled { '=' } else { ' ' };
            
            if i < filled {
                // Calculate a blend of the colors based on position within filled area
                let color_pos = i as f64 / filled.max(1) as f64;
                
                // Choose a color from the gradient
                let color = match (start_color, end_color) {
                    (Color::Red, Color::Green) => {
                        if color_pos < 0.5 {
                            Color::Red
                        } else {
                            Color::Green
                        }
                    },
                    (Color::Blue, Color::Cyan) => {
                        if color_pos < 0.5 {
                            Color::Blue
                        } else {
                            Color::Cyan
                        }
                    },
                    _ => {
                        if color_pos < 0.5 {
                            start_color
                        } else {
                            end_color
                        }
                    },
                };
                
                result.push_str(&char.to_string().with(color).to_string());
            } else {
                result.push(char);
            }
        }
        
        result.push(']');
        
        Ok(Some(result))
    }
}

/// Supported color names for formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorName {
    /// Black color
    Black,
    /// Red color
    Red,
    /// Green color
    Green,
    /// Yellow color
    Yellow,
    /// Blue color
    Blue,
    /// Magenta color
    Magenta,
    /// Cyan color
    Cyan,
    /// White color
    White,
    /// Reset to default color
    Reset,
}

impl ColorName {
    /// Convert to crossterm Color
    fn to_color(self) -> Color {
        match self {
            ColorName::Black => Color::Black,
            ColorName::Red => Color::Red,
            ColorName::Green => Color::Green,
            ColorName::Yellow => Color::Yellow,
            ColorName::Blue => Color::Blue,
            ColorName::Magenta => Color::Magenta,
            ColorName::Cyan => Color::Cyan,
            ColorName::White => Color::White,
            ColorName::Reset => Color::Reset,
        }
    }
    
    /// Parse from string
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "black" => Some(ColorName::Black),
            "red" => Some(ColorName::Red),
            "green" => Some(ColorName::Green),
            "yellow" => Some(ColorName::Yellow),
            "blue" => Some(ColorName::Blue),
            "magenta" => Some(ColorName::Magenta),
            "cyan" => Some(ColorName::Cyan),
            "white" => Some(ColorName::White),
            "reset" => Some(ColorName::Reset),
            _ => None,
        }
    }
}

/// Built-in template presets for common progress displays
pub enum TemplatePreset {
    /// Simple progress bar: "[====    ] 50% (5/10)"
    SimpleProgress,
    /// Task status: "Running task: <message>"
    TaskStatus,
    /// Job progress: "Completed 5/10 jobs (50%)"
    JobProgress,
    /// Download progress: "Downloading file.txt [====    ] 10.5 MB / 20 MB (50%)"
    DownloadProgress,
}

impl TemplatePreset {
    /// Get the template string for this preset
    pub fn template_string(&self) -> &'static str {
        match self {
            TemplatePreset::SimpleProgress => "{progress:bar:10} {progress:percent} ({completed}/{total})",
            TemplatePreset::TaskStatus => "Running task: {message}",
            TemplatePreset::JobProgress => "Completed {completed}/{total} jobs ({progress:percent})",
            TemplatePreset::DownloadProgress => "Downloading {filename} {progress:bar:10} {bytes_done} / {bytes_total} ({progress:percent})",
        }
    }
    
    /// Create a ProgressTemplate from this preset
    pub fn create_template(&self) -> ProgressTemplate {
        ProgressTemplate::new(self.template_string())
    }
}

/// Supported progress indicator types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressIndicator {
    /// Traditional progress bar using characters
    /// Example: "[====    ]"
    Bar,
    
    /// Block-based progress bar using block characters
    /// Example: "[██████    ]" or "█▓▒░ "
    Block,
    
    /// Animated spinner that rotates through frames
    /// Example: One of "-\|/" based on progress
    Spinner,
    
    /// Simple numeric display (no visual indicator)
    /// Example: "50%"
    Numeric,
    
    /// Custom indicator defined by the user
    /// This allows for user-defined progress indicators with 
    /// custom rendering logic
    /// Used with {progress:bar:custom:name} in templates
    Custom(String),
}

impl ProgressIndicator {
    /// Get the default frames for a spinner indicator
    ///
    /// # Returns
    /// A vector of strings representing the spinner frames
    pub fn default_spinner_frames() -> Vec<&'static str> {
        vec!["-", "\\", "|", "/"]
    }
    
    /// Get the default characters for a block indicator
    ///
    /// # Returns
    /// A string containing the block characters from full to empty
    pub fn default_block_chars() -> &'static str {
        "█▓▒░ "
    }
}

/// Error returned when parsing a string to ProgressIndicator fails
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressIndicatorParseError;

impl std::fmt::Display for ProgressIndicatorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid progress indicator name")
    }
}

impl std::error::Error for ProgressIndicatorParseError {}

impl FromStr for ProgressIndicator {
    type Err = ProgressIndicatorParseError;
    
    /// Parse a string into a ProgressIndicator
    ///
    /// # Parameters
    /// * `s` - The indicator name as a string
    ///
    /// # Returns
    /// Ok(ProgressIndicator) if the name is valid, Err otherwise
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        let base_type = parts[0].to_lowercase();
        
        match base_type.as_str() {
            "bar" => Ok(ProgressIndicator::Bar),
            "block" => Ok(ProgressIndicator::Block),
            "spinner" => Ok(ProgressIndicator::Spinner),
            "numeric" => Ok(ProgressIndicator::Numeric),
            "custom" => {
                if parts.len() > 1 {
                    Ok(ProgressIndicator::Custom(parts[1].to_string()))
                } else {
                    Err(ProgressIndicatorParseError)
                }
            }
            _ => Err(ProgressIndicatorParseError),
        }
    }
}

/// Custom indicator type for formatting progress
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomIndicatorType {
    /// Dots indicator that uses Unicode dots
    Dots,
    /// Braille-based indicator that uses Unicode braille patterns
    Braille,
    /// Gradient indicator that uses color gradients
    Gradient,
}

/// Error that can occur when parsing a custom indicator type from a string
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomIndicatorTypeError;

impl std::fmt::Display for CustomIndicatorTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown custom indicator type")
    }
}

impl std::error::Error for CustomIndicatorTypeError {}

impl std::str::FromStr for CustomIndicatorType {
    type Err = CustomIndicatorTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dots" => Ok(CustomIndicatorType::Dots),
            "braille" => Ok(CustomIndicatorType::Braille),
            "gradient" => Ok(CustomIndicatorType::Gradient),
            _ => Err(CustomIndicatorTypeError),
        }
    }
}

impl CustomIndicatorType {
    /// Get a list of all available custom indicator types
    pub fn variants() -> &'static [&'static str] {
        &["dots", "braille", "gradient"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_template_var_conversion() {
        assert!(matches!(TemplateVar::from("hello"), TemplateVar::String(_)));
        assert!(matches!(TemplateVar::from(42usize), TemplateVar::Number(_)));
        assert!(matches!(TemplateVar::from(true), TemplateVar::Boolean(_)));
    }
    
    #[test]
    fn test_template_context() {
        let mut ctx = TemplateContext::new();
        ctx.set("str", "hello")
           .set("num", 42)
           .set("bool", true);
        
        assert!(ctx.has("str"));
        assert!(ctx.has("num"));
        assert!(ctx.has("bool"));
        assert!(!ctx.has("missing"));
        
        assert!(matches!(ctx.get("str"), Some(TemplateVar::String(_))));
        assert!(matches!(ctx.get("num"), Some(TemplateVar::Number(_))));
        assert!(matches!(ctx.get("bool"), Some(TemplateVar::Boolean(_))));
    }
    
    #[test]
    fn test_simple_template() {
        let template = ProgressTemplate::new("Hello, {name}!");
        let mut ctx = TemplateContext::new();
        ctx.set("name", "world");
        
        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "Hello, world!");
    }
    
    #[test]
    fn test_progress_bar() {
        let template = ProgressTemplate::new("{progress:bar:10}");
        let mut ctx = TemplateContext::new();
        ctx.set("progress", 0.5);
        
        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "[=====     ]");
    }
    
    #[test]
    fn test_percentage_format() {
        let template = ProgressTemplate::new("{progress:percent}");
        let mut ctx = TemplateContext::new();
        ctx.set("progress", 0.75);
        
        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "75%");
    }
    
    #[test]
    fn test_ratio_format() {
        let template = ProgressTemplate::new("{completed:ratio:total}");
        let mut ctx = TemplateContext::new();
        ctx.set("completed", 7)
           .set("total", 10);
        
        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "7/10");
    }
    
    #[test]
    fn test_padding_formats() {
        let template = ProgressTemplate::new("'{text:lpad:10}' '{text:rpad:10}' '{text:pad:10}'");
        let mut ctx = TemplateContext::new();
        ctx.set("text", "test");
        
        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "'      test' 'test      ' '   test   '");
    }
    
    #[test]
    fn test_template_preset() {
        let template = TemplatePreset::SimpleProgress.create_template();
        let mut ctx = TemplateContext::new();
        ctx.set("progress", 0.5)
           .set("completed", 5)
           .set("total", 10);
        
        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "[=====     ] 50% (5/10)");
    }
    
    #[test]
    fn test_color_format() {
        let template = ProgressTemplate::new("Hello, {name:color:red}!");
        let mut ctx = TemplateContext::new();
        ctx.set("name", "World");
        
        let result = template.render(&ctx).unwrap();
        // The result should contain ANSI color codes
        assert!(result.contains("\x1B[31m"), "Result should contain red color code");
        assert!(result.contains("\x1B[0m"), "Result should contain reset code");
        assert!(result.contains("World"), "Result should contain the variable value");
    }
    
    #[test]
    fn test_color_name_parsing() {
        assert_eq!(ColorName::from_str("red"), Some(ColorName::Red));
        assert_eq!(ColorName::from_str("RED"), Some(ColorName::Red));
        assert_eq!(ColorName::from_str("Red"), Some(ColorName::Red));
        assert_eq!(ColorName::from_str("unknown"), None);
        
        // Test color to crossterm Color conversion
        assert_eq!(ColorName::Red.to_color(), Color::Red);
        assert_eq!(ColorName::Green.to_color(), Color::Green);
        assert_eq!(ColorName::Reset.to_color(), Color::Reset);
    }
    
    #[test]
    fn test_invalid_color_format() {
        let template = ProgressTemplate::new("Hello, {name:color}!");
        let mut ctx = TemplateContext::new();
        ctx.set("name", "World");
        
        // Should error without a color name
        assert!(template.render(&ctx).is_err());
        
        let template = ProgressTemplate::new("Hello, {name:color:invalid}!");
        assert!(template.render(&ctx).is_err());
    }
    
    #[test]
    fn test_progress_indicator_types() {
        let template = ProgressTemplate::new("Default: {p:bar} Block: {p:bar:block} Spinner: {p:bar:spinner} Numeric: {p:bar:numeric}");
        let mut ctx = TemplateContext::new();
        ctx.set("p", 0.5);
        
        let output = template.render(&ctx).unwrap();
        
        // Check each indicator type is present
        assert!(output.contains("Default: [=====     ]"));
        assert!(output.contains("Block: [█████     ]"));
        assert!(output.contains("Spinner: "));  // One of the spinner frames will be present
        assert!(output.contains("Numeric: 50%"));
    }
    
    #[test]
    fn test_block_indicator_custom_chars() {
        let template = ProgressTemplate::new("{p:bar:block:10:#}");
        let mut ctx = TemplateContext::new();
        ctx.set("p", 0.5);
        
        let output = template.render(&ctx).unwrap();
        assert_eq!(output, "[#####     ]");
    }
    
    #[test]
    fn test_spinner_indicator_custom_frames() {
        let template = ProgressTemplate::new("{p:bar:spinner:abcd}");
        let mut ctx = TemplateContext::new();
        
        // Test with different progress values to cycle through frames
        ctx.set("p", 0.0);
        let output1 = template.render(&ctx).unwrap();
        
        ctx.set("p", 0.25);
        let output2 = template.render(&ctx).unwrap();
        
        ctx.set("p", 0.5);
        let output3 = template.render(&ctx).unwrap();
        
        ctx.set("p", 0.75);
        let output4 = template.render(&ctx).unwrap();
        
        // Each output should be one of the frame characters
        assert!(["a", "b", "c", "d"].contains(&output1.as_str()));
        assert!(["a", "b", "c", "d"].contains(&output2.as_str()));
        assert!(["a", "b", "c", "d"].contains(&output3.as_str()));
        assert!(["a", "b", "c", "d"].contains(&output4.as_str()));
        
        // At least two different frames should be used
        let outputs = vec![output1, output2, output3, output4];
        let unique_outputs = outputs.iter().collect::<std::collections::HashSet<_>>();
        assert!(unique_outputs.len() > 1);
    }
    
    #[test]
    fn test_numeric_indicator_options() {
        let template = ProgressTemplate::new("With sign: {p:bar:numeric} Without sign: {p:bar:numeric:false}");
        let mut ctx = TemplateContext::new();
        ctx.set("p", 0.75);
        
        let output = template.render(&ctx).unwrap();
        assert_eq!(output, "With sign: 75% Without sign: 75");
    }
    
    #[test]
    fn test_progress_indicator_parsing() {
        use std::str::FromStr;
        
        assert_eq!(ProgressIndicator::from_str("bar"), Ok(ProgressIndicator::Bar));
        assert_eq!(ProgressIndicator::from_str("BAR"), Ok(ProgressIndicator::Bar));
        assert_eq!(ProgressIndicator::from_str("Bar"), Ok(ProgressIndicator::Bar));
        assert_eq!(ProgressIndicator::from_str("block"), Ok(ProgressIndicator::Block));
        assert_eq!(ProgressIndicator::from_str("spinner"), Ok(ProgressIndicator::Spinner));
        assert_eq!(ProgressIndicator::from_str("numeric"), Ok(ProgressIndicator::Numeric));
        assert_eq!(ProgressIndicator::from_str("custom:dots"), Ok(ProgressIndicator::Custom("dots".to_string())));
        assert_eq!(ProgressIndicator::from_str("custom:braille"), Ok(ProgressIndicator::Custom("braille".to_string())));
        assert_eq!(ProgressIndicator::from_str("custom:gradient"), Ok(ProgressIndicator::Custom("gradient".to_string())));
        assert!(ProgressIndicator::from_str("unknown").is_err());
        assert!(ProgressIndicator::from_str("custom").is_err());
    }
    
    #[test]
    fn test_custom_indicators() {
        // Test dots indicator
        let template = ProgressTemplate::new("{p:bar:custom:dots}");
        let mut ctx = TemplateContext::new();
        ctx.set("p", 0.5);
        
        let result = template.render(&ctx).unwrap();
        assert!(!result.is_empty(), "Dots indicator should produce non-empty output");
        
        // Test braille indicator
        let template = ProgressTemplate::new("{p:bar:custom:braille}");
        let mut ctx = TemplateContext::new();
        ctx.set("p", 0.5);
        
        let result = template.render(&ctx).unwrap();
        assert!(!result.is_empty(), "Braille indicator should produce non-empty output");
        
        // Test gradient indicator
        let template = ProgressTemplate::new("{p:bar:custom:gradient}");
        let mut ctx = TemplateContext::new();
        ctx.set("p", 0.5);
        
        let result = template.render(&ctx).unwrap();
        assert!(result.contains("["), "Result should contain bar brackets");
        assert!(result.contains("]"), "Result should contain bar brackets");
    }
} 