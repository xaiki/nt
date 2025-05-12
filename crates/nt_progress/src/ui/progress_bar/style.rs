use crate::errors::ProgressError;
use crate::ui::formatter::{ProgressIndicator, CustomIndicatorType};
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
} 