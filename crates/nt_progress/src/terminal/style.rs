use anyhow::Result;
use crossterm::style::{Color, SetForegroundColor, SetBackgroundColor, SetAttribute, Attribute};
use crossterm::QueueableCommand;
use std::io::Write;

/// Represents a terminal style with foreground color, background color, and attributes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    /// Foreground color
    foreground: Option<Color>,
    /// Background color
    background: Option<Color>,
    /// Style attributes (bold, italic, etc.)
    attributes: Vec<Attribute>,
}

impl Style {
    /// Creates a new empty style
    pub fn new() -> Self {
        Self {
            foreground: None,
            background: None,
            attributes: Vec::new(),
        }
    }
    
    /// Creates a new style with the given foreground color
    pub fn with_foreground(foreground: Color) -> Self {
        let mut style = Self::new();
        style.foreground = Some(foreground);
        style
    }
    
    /// Creates a new style with the given background color
    pub fn with_background(background: Color) -> Self {
        let mut style = Self::new();
        style.background = Some(background);
        style
    }
    
    /// Sets the foreground color
    pub fn foreground(&mut self, color: Color) -> &mut Self {
        self.foreground = Some(color);
        self
    }
    
    /// Sets the background color
    pub fn background(&mut self, color: Color) -> &mut Self {
        self.background = Some(color);
        self
    }
    
    /// Adds an attribute to the style
    pub fn attribute(&mut self, attr: Attribute) -> &mut Self {
        self.attributes.push(attr);
        self
    }
    
    /// Adds the bold attribute
    pub fn bold(&mut self) -> &mut Self {
        self.attribute(Attribute::Bold)
    }
    
    /// Adds the italic attribute
    pub fn italic(&mut self) -> &mut Self {
        self.attribute(Attribute::Italic)
    }
    
    /// Adds the underlined attribute
    pub fn underlined(&mut self) -> &mut Self {
        self.attribute(Attribute::Underlined)
    }
    
    /// Resets all style attributes
    pub fn reset(&mut self) -> &mut Self {
        self.foreground = None;
        self.background = None;
        self.attributes.clear();
        self
    }
    
    /// Applies the style to the terminal
    pub fn apply<W: Write>(&self, w: &mut W) -> Result<()> {
        // Apply foreground color if set
        if let Some(color) = self.foreground {
            w.queue(SetForegroundColor(color))?;
        }
        
        // Apply background color if set
        if let Some(color) = self.background {
            w.queue(SetBackgroundColor(color))?;
        }
        
        // Apply attributes
        for attr in &self.attributes {
            w.queue(SetAttribute(*attr))?;
        }
        
        Ok(())
    }
    
    /// Applies the style to stdout
    pub fn apply_to_stdout(&self) -> Result<()> {
        let mut stdout = std::io::stdout();
        self.apply(&mut stdout)?;
        stdout.flush()?;
        Ok(())
    }
    
    /// Resets all terminal styles on stdout
    pub fn reset_stdout() -> Result<()> {
        let mut stdout = std::io::stdout();
        stdout.queue(SetAttribute(Attribute::Reset))?;
        stdout.flush()?;
        Ok(())
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_style_creation() {
        let style = Style::new();
        assert_eq!(style.foreground, None);
        assert_eq!(style.background, None);
        assert!(style.attributes.is_empty());
        
        let fg_style = Style::with_foreground(Color::Red);
        assert_eq!(fg_style.foreground, Some(Color::Red));
        assert_eq!(fg_style.background, None);
        
        let bg_style = Style::with_background(Color::Blue);
        assert_eq!(bg_style.foreground, None);
        assert_eq!(bg_style.background, Some(Color::Blue));
    }
    
    #[test]
    fn test_style_modification() {
        let mut style = Style::new();
        
        style.foreground(Color::Green)
             .background(Color::Black)
             .bold()
             .underlined();
        
        assert_eq!(style.foreground, Some(Color::Green));
        assert_eq!(style.background, Some(Color::Black));
        assert_eq!(style.attributes.len(), 2);
        assert!(style.attributes.contains(&Attribute::Bold));
        assert!(style.attributes.contains(&Attribute::Underlined));
        
        style.reset();
        
        assert_eq!(style.foreground, None);
        assert_eq!(style.background, None);
        assert!(style.attributes.is_empty());
    }
} 