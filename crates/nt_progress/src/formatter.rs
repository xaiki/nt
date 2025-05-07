//! A template-based message formatting system for progress displays.
//!
//! This module provides a simple yet powerful templating system for formatting task progress
//! messages. It supports variable interpolation, conditional rendering, and basic formatting
//! operations.

use std::collections::HashMap;
use crate::errors::{ProgressError, ContextExt};

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
        let (is_positive, condition) = if tag.starts_with('?') {
            (true, &tag[1..])
        } else if tag.starts_with('!') {
            (false, &tag[1..])
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
        let format = format_parts[0].trim();
        
        match format {
            "bar" => self.format_bar(var, format_parts, context),
            "percent" => self.format_percent(var, format_parts, context),
            "ratio" => self.format_ratio(var, format_parts, context),
            "pad" | "lpad" | "rpad" => self.format_padding(var, format, format_parts, context),
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
        let progress = progress.max(0.0).min(1.0);
        
        // Get the bar width (default: 10)
        let width = if format_parts.len() > 1 {
            format_parts[1].parse::<usize>().unwrap_or(10)
        } else {
            10
        };
        
        // Get the fill character (default: '=')
        let fill_char = if format_parts.len() > 2 {
            format_parts[2].chars().next().unwrap_or('=')
        } else {
            '='
        };
        
        // Get the background character (default: ' ')
        let bg_char = if format_parts.len() > 3 {
            format_parts[3].chars().next().unwrap_or(' ')
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
        let percent = (progress.max(0.0).min(1.0) * 100.0).round() as usize;
        
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
} 