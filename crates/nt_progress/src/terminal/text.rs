//! Text manipulation utilities for terminal output
//!
//! This module provides utilities for text manipulation, such as line wrapping,
//! truncation, and other text transformations needed for terminal display.

use unicode_segmentation::UnicodeSegmentation;

/// A utility for wrapping text at specified widths
#[derive(Debug, Clone)]
pub struct TextWrapper {
    /// Maximum width for wrapped lines
    max_width: usize,
    /// Whether to break words that are longer than max_width
    break_long_words: bool,
    /// Characters to use for word boundaries
    word_separators: Vec<char>,
    /// String to use at the end of truncated lines
    truncation_marker: String,
}

impl TextWrapper {
    /// Creates a new TextWrapper with the specified max width
    ///
    /// # Parameters
    /// * `max_width` - The maximum width for wrapped lines
    ///
    /// # Returns
    /// A new TextWrapper instance
    pub fn new(max_width: usize) -> Self {
        Self {
            max_width,
            break_long_words: true,
            word_separators: vec![' ', '\t', '-', '_', ',', ';', ':', '!', '?', '.'],
            truncation_marker: "…".to_string(),
        }
    }

    /// Sets whether to break long words that exceed max_width
    ///
    /// # Parameters
    /// * `break_long_words` - If true, words longer than max_width will be broken
    ///
    /// # Returns
    /// Self for method chaining
    pub fn break_long_words(mut self, break_long_words: bool) -> Self {
        self.break_long_words = break_long_words;
        self
    }

    /// Sets the characters considered as word separators
    ///
    /// # Parameters
    /// * `separators` - Characters to consider as word separators
    ///
    /// # Returns
    /// Self for method chaining
    pub fn word_separators(mut self, separators: Vec<char>) -> Self {
        self.word_separators = separators;
        self
    }

    /// Sets the marker to use for truncated lines
    ///
    /// # Parameters
    /// * `marker` - The string to use as truncation marker
    ///
    /// # Returns
    /// Self for method chaining
    pub fn truncation_marker(mut self, marker: impl Into<String>) -> Self {
        self.truncation_marker = marker.into();
        self
    }

    /// Wraps text at the configured max_width
    ///
    /// # Parameters
    /// * `text` - The text to wrap
    ///
    /// # Returns
    /// A vector of wrapped lines
    pub fn wrap(&self, text: &str) -> Vec<String> {
        if text.is_empty() {
            return vec![String::new()];
        }

        // If the text fits, return it directly
        if self.visual_width(text) <= self.max_width {
            return vec![text.to_string()];
        }

        let mut result = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        // Split text into grapheme clusters to handle multi-byte characters correctly
        let graphemes = UnicodeSegmentation::graphemes(text, true).collect::<Vec<_>>();
        let mut i = 0;

        while i < graphemes.len() {
            // Find the next word or chunk of text
            let (word, word_width) = self.get_next_word(&graphemes[i..]);
            
            // Check if the word fits on the current line
            if current_width + word_width <= self.max_width {
                // Word fits, add it to the current line
                current_line.push_str(word.as_str());
                current_width += word_width;
                i += word.graphemes(true).count();
            } else if word_width > self.max_width && self.break_long_words {
                // Word is too long and we're allowed to break it
                let (first_part, remaining) = self.break_word(word.as_str(), self.max_width - current_width);
                
                // Add the first part to the current line
                current_line.push_str(first_part.as_str());
                result.push(current_line);
                
                // Start a new line with the remaining part
                current_line = remaining.to_string();
                current_width = self.visual_width(&current_line);
                i += word.graphemes(true).count();
            } else {
                // Word doesn't fit, start a new line
                if !current_line.is_empty() {
                    result.push(current_line);
                    current_line = String::new();
                    current_width = 0;
                }
                
                // If the word is longer than max_width and we can't break it,
                // we'll have to truncate it
                if word_width > self.max_width && !self.break_long_words {
                    let truncated = self.truncate_text(word.as_str(), self.max_width);
                    result.push(truncated);
                    i += word.graphemes(true).count();
                } else {
                    // Word will fit on a new line
                    current_line.push_str(word.as_str());
                    current_width = word_width;
                    i += word.graphemes(true).count();
                }
            }
        }

        // Add the last line if it's not empty
        if !current_line.is_empty() {
            result.push(current_line);
        }

        result
    }

    /// Gets the next word or chunk from graphemes
    ///
    /// # Parameters
    /// * `graphemes` - The grapheme clusters to process
    ///
    /// # Returns
    /// A tuple containing the word and its visual width
    fn get_next_word<'a>(&self, graphemes: &[&'a str]) -> (String, usize) {
        if graphemes.is_empty() {
            return (String::new(), 0);
        }

        // Check if the first character is a separator
        let first_char = graphemes[0].chars().next().unwrap_or(' ');
        if self.word_separators.contains(&first_char) {
            return (graphemes[0].to_string(), self.visual_width(graphemes[0]));
        }

        // Find the next word boundary
        let mut end = 1;
        while end < graphemes.len() {
            let ch = graphemes[end].chars().next().unwrap_or(' ');
            if self.word_separators.contains(&ch) {
                break;
            }
            end += 1;
        }

        // Combine the graphemes into a word
        let word = graphemes[..end].join("");
        let width = self.visual_width(&word);

        (word, width)
    }

    /// Breaks a word at the specified width
    ///
    /// # Parameters
    /// * `word` - The word to break
    /// * `available_width` - The available width for the first part
    ///
    /// # Returns
    /// A tuple containing the first part and the remaining part
    fn break_word(&self, word: &str, available_width: usize) -> (String, String) {
        let graphemes = UnicodeSegmentation::graphemes(word, true).collect::<Vec<_>>();
        let mut first_part = String::new();
        let mut current_width = 0;
        let mut i = 0;

        // Add graphemes to the first part until we reach the available width
        while i < graphemes.len() && current_width < available_width {
            let grapheme_width = self.visual_width(graphemes[i]);
            if current_width + grapheme_width <= available_width {
                first_part.push_str(graphemes[i]);
                current_width += grapheme_width;
                i += 1;
            } else {
                break;
            }
        }

        // Combine the remaining graphemes into the second part
        let remaining = graphemes[i..].join("");

        (first_part, remaining)
    }

    /// Truncates text to the specified width, adding a truncation marker
    ///
    /// # Parameters
    /// * `text` - The text to truncate
    /// * `width` - The maximum width for the truncated text
    ///
    /// # Returns
    /// The truncated text with the truncation marker
    fn truncate_text(&self, text: &str, width: usize) -> String {
        if self.visual_width(text) <= width {
            return text.to_string();
        }

        let marker_width = self.visual_width(&self.truncation_marker);
        let available_width = width.saturating_sub(marker_width);
        
        let graphemes = UnicodeSegmentation::graphemes(text, true).collect::<Vec<_>>();
        let mut result = String::new();
        let mut current_width = 0;
        
        // Add graphemes until we reach the available width
        for grapheme in graphemes {
            let grapheme_width = self.visual_width(grapheme);
            if current_width + grapheme_width <= available_width {
                result.push_str(grapheme);
                current_width += grapheme_width;
            } else {
                break;
            }
        }
        
        // Add truncation marker
        result.push_str(&self.truncation_marker);
        
        result
    }

    /// Calculates the visual width of a string, accounting for wide characters
    ///
    /// # Parameters
    /// * `text` - The text to measure
    ///
    /// # Returns
    /// The visual width of the text
    fn visual_width(&self, text: &str) -> usize {
        let mut width = 0;
        for grapheme in UnicodeSegmentation::graphemes(text, true) {
            // Count wide characters (like CJK characters) as 2 columns
            let ch = grapheme.chars().next().unwrap_or(' ');
            if is_wide_char(ch) {
                width += 2;
            } else {
                width += 1;
            }
        }
        width
    }
}

/// Checks if a character is visually wide (occupies 2 columns)
///
/// # Parameters
/// * `ch` - The character to check
///
/// # Returns
/// true if the character is visually wide, false otherwise
fn is_wide_char(ch: char) -> bool {
    // East Asian Wide (W) and East Asian Full-width (F) characters
    // This is a simplified check that covers common cases
    matches!(ch,
        '\u{1100}'..='\u{11FF}' |   // Hangul Jamo
        '\u{2E80}'..='\u{9FFF}' |   // CJK Unified Ideographs
        '\u{AC00}'..='\u{D7AF}' |   // Hangul Syllables
        '\u{F900}'..='\u{FAFF}' |   // CJK Compatibility Ideographs
        '\u{FE10}'..='\u{FE19}' |   // Vertical Forms
        '\u{FE30}'..='\u{FE6F}' |   // CJK Compatibility Forms
        '\u{FF00}'..='\u{FF60}' |   // Fullwidth ASCII Variants
        '\u{FFE0}'..='\u{FFE6}'     // Fullwidth Symbol Variants
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_wrapping() {
        let wrapper = TextWrapper::new(20);
        let input = "This is a test of the line wrapping functionality";
        let wrapped = wrapper.wrap(input);
        
        assert_eq!(wrapped, vec![
            "This is a test of ",
            "the line wrapping ",
            "functionality"
        ]);
    }

    #[test]
    fn test_long_words() {
        let wrapper = TextWrapper::new(10);
        let input = "Supercalifragilisticexpialidocious";
        
        // With break_long_words = true (default)
        let wrapped = wrapper.wrap(input);
        assert_eq!(wrapped.len(), 2);
        
        // With break_long_words = false
        let wrapper = wrapper.break_long_words(false);
        let wrapped = wrapper.wrap(input);
        assert_eq!(wrapped.len(), 1);
        assert!(wrapped[0].ends_with("…"));
    }

    #[test]
    fn test_unicode_wrapping() {
        let wrapper = TextWrapper::new(10);
        let input = "こんにちは世界";  // "Hello World" in Japanese
        let wrapped = wrapper.wrap(input);
        
        // Each character takes 2 columns, so we should get 2 lines
        assert_eq!(wrapped.len(), 2);
    }

    #[test]
    fn test_mixed_text() {
        let wrapper = TextWrapper::new(20);
        let input = "Hello 你好 Bonjour こんにちは";
        let wrapped = wrapper.wrap(input);
        
        // The mixed text contains several wide characters
        assert!(wrapped.len() >= 2);
    }

    #[test]
    fn test_empty_text() {
        let wrapper = TextWrapper::new(10);
        let wrapped = wrapper.wrap("");
        
        assert_eq!(wrapped, vec![String::new()]);
    }

    #[test]
    fn test_visual_width() {
        let wrapper = TextWrapper::new(10);
        
        // ASCII text
        assert_eq!(wrapper.visual_width("Hello"), 5);
        
        // Text with wide characters
        assert_eq!(wrapper.visual_width("你好"), 4);
        assert_eq!(wrapper.visual_width("こんにちは"), 10);
        
        // Mixed text
        assert_eq!(wrapper.visual_width("Hi 你好"), 7);
    }
} 