//! Typography controls and text layout
//! 
//! This module provides comprehensive typography controls including
//! font metrics, text layout, and rendering utilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Typography controls for text styling and layout
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographyControls {
    /// Font family
    pub font_family: String,
    /// Font size in points
    pub font_size: f64,
    /// Font weight (100-900)
    pub font_weight: u16,
    /// Font style
    pub font_style: crate::text::types::FontStyle,
    /// Line height multiplier
    pub line_height: f64,
    /// Letter spacing in points
    pub letter_spacing: f64,
    /// Word spacing in points
    pub word_spacing: f64,
    /// Paragraph spacing in points
    pub paragraph_spacing: f64,
    /// Text alignment
    pub alignment: crate::text::types::TextAlignment,
    /// Text direction
    pub direction: crate::text::types::TextDirection,
    /// Text indentation in points
    pub indentation: f64,
    /// Text justification (for justified alignment)
    pub justification: TextJustification,
    /// Hyphenation settings
    pub hyphenation: HyphenationSettings,
    /// Font features (OpenType features)
    pub font_features: HashMap<String, f64>,
}

/// Text justification options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextJustification {
    /// Whether justification is enabled
    pub enabled: bool,
    /// Minimum word spacing for justification
    pub min_word_spacing: f64,
    /// Maximum word spacing for justification
    pub max_word_spacing: f64,
    /// Minimum glyph spacing for justification
    pub min_glyph_spacing: f64,
    /// Maximum glyph spacing for justification
    pub max_glyph_spacing: f64,
}

/// Hyphenation settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyphenationSettings {
    /// Whether hyphenation is enabled
    pub enabled: bool,
    /// Hyphenation language (ISO 639-1 code)
    pub language: String,
    /// Minimum characters before hyphen
    pub min_before: usize,
    /// Minimum characters after hyphen
    pub min_after: usize,
    /// Hyphenation character
    pub hyphen_char: String,
}

/// Font metrics for text measurement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontMetrics {
    /// Font family
    pub font_family: String,
    /// Font size
    pub font_size: f64,
    /// Ascent (distance from baseline to top of highest glyph)
    pub ascent: f64,
    /// Descent (distance from baseline to bottom of lowest glyph)
    pub descent: f64,
    /// Line gap (extra space between lines)
    pub line_gap: f64,
    /// Cap height (height of capital letters)
    pub cap_height: f64,
    /// X-height (height of lowercase x)
    pub x_height: f64,
    /// Average character width
    pub avg_char_width: f64,
    /// Maximum character width
    pub max_char_width: f64,
    /// Underline position
    pub underline_position: f64,
    /// Underline thickness
    pub underline_thickness: f64,
}

/// Text layout information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLayout {
    /// Layout lines
    pub lines: Vec<TextLine>,
    /// Total width of layout
    pub width: f64,
    /// Total height of layout
    pub height: f64,
    /// Baseline positions for each line
    pub baselines: Vec<f64>,
    /// Font metrics used for layout
    pub metrics: FontMetrics,
}

/// Single line of text layout
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLine {
    /// Line text
    pub text: String,
    /// Line start index in original text
    pub start_index: usize,
    /// Line end index in original text
    pub end_index: usize,
    /// Line width
    pub width: f64,
    /// Line height
    pub height: f64,
    /// Line baseline offset
    pub baseline: f64,
    /// Glyph positions in the line
    pub glyph_positions: Vec<GlyphPosition>,
    /// Word boundaries in the line
    pub word_boundaries: Vec<usize>,
}

/// Glyph position information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphPosition {
    /// Glyph character
    pub character: char,
    /// Character index in original text
    pub char_index: usize,
    /// X position
    pub x: f64,
    /// Y position
    pub y: f64,
    /// Glyph width
    pub width: f64,
    /// Glyph height
    pub height: f64,
    /// Glyph advance (distance to next glyph)
    pub advance: f64,
    /// Glyph bearing (offset from advance)
    pub bearing: (f64, f64),
}

impl TypographyControls {
    /// Create new typography controls
    pub fn new() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 12.0,
            font_weight: 400,
            font_style: crate::text::types::FontStyle::Normal,
            line_height: 1.2,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            paragraph_spacing: 0.0,
            alignment: crate::text::types::TextAlignment::Left,
            direction: crate::text::types::TextDirection::LeftToRight,
            indentation: 0.0,
            justification: TextJustification::default(),
            hyphenation: HyphenationSettings::default(),
            font_features: HashMap::new(),
        }
    }
    
    /// Create typography controls from text style
    pub fn from_style(style: &crate::text::types::TextStyle) -> Self {
        Self {
            font_family: style.font_family.clone(),
            font_size: style.font_size,
            font_weight: style.font_weight,
            font_style: style.font_style,
            line_height: style.line_height,
            letter_spacing: style.letter_spacing,
            word_spacing: style.word_spacing,
            paragraph_spacing: 0.0,
            alignment: crate::text::types::TextAlignment::Left,
            direction: crate::text::types::TextDirection::LeftToRight,
            indentation: 0.0,
            justification: TextJustification::default(),
            hyphenation: HyphenationSettings::default(),
            font_features: HashMap::new(),
        }
    }
    
    /// Get font metrics for the current typography settings
    pub fn get_font_metrics(&self) -> FontMetrics {
        // In a real implementation, this would query the font system
        // For now, we'll return reasonable defaults
        let scale_factor = self.font_size / 12.0; // Scale from 12pt base
        
        FontMetrics {
            font_family: self.font_family.clone(),
            font_size: self.font_size,
            ascent: 9.6 * scale_factor,
            descent: -2.4 * scale_factor,
            line_gap: 2.0 * scale_factor,
            cap_height: 7.2 * scale_factor,
            x_height: 4.8 * scale_factor,
            avg_char_width: 6.0 * scale_factor,
            max_char_width: 8.0 * scale_factor,
            underline_position: -1.2 * scale_factor,
            underline_thickness: 0.6 * scale_factor,
        }
    }
    
    /// Layout text with current typography settings
    pub fn layout_text(&self, text: &str, max_width: f64) -> TextLayout {
        let metrics = self.get_font_metrics();
        let line_height = self.font_size * self.line_height;
        
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;
        let mut char_index = 0;
        let mut line_start = 0;
        
        for (line_num, line_text) in text.lines().enumerate() {
            if line_num > 0 {
                // Add paragraph spacing
                if !lines.is_empty() {
                    lines.last_mut().unwrap().height += self.paragraph_spacing;
                }
            }
            
            let words: Vec<&str> = line_text.split_whitespace().collect();
            let mut line_words = Vec::new();
            let mut line_width = 0.0;
            
            for (word_index, word) in words.iter().enumerate() {
                let word_width = self.estimate_word_width(word, &metrics);
                let spacing = if word_index > 0 { self.word_spacing } else { 0.0 };
                let total_width = line_width + word_width + spacing;
                
                if total_width > max_width && !line_words.is_empty() {
                    // Finish current line
                    let line = self.create_line(
                        &line_words.join(" "),
                        line_start,
                        char_index,
                        line_width,
                        line_height,
                        &metrics,
                    );
                    lines.push(line);
                    
                    // Start new line
                    line_words = vec![*word];
                    line_width = word_width;
                    line_start = char_index;
                } else {
                    line_words.push(*word);
                    line_width = total_width;
                }
                
                char_index += word.len() + 1; // +1 for space
            }
            
            // Add remaining words
            if !line_words.is_empty() {
                let line = self.create_line(
                    &line_words.join(" "),
                    line_start,
                    char_index - 1,
                    line_width,
                    line_height,
                    &metrics,
                );
                lines.push(line);
            }
        }
        
        let total_height = lines.iter().map(|line| line.height).sum();
        let max_width = lines.iter().map(|line| line.width).fold(0.0, f64::max);
        let baselines: Vec<f64> = lines.iter()
            .scan(0.0, |acc, line| {
                let baseline = *acc + line.baseline;
                *acc += line.height;
                Some(baseline)
            })
            .collect();
        
        TextLayout {
            lines,
            width: max_width,
            height: total_height,
            baselines,
            metrics,
        }
    }
    
    /// Create a text line with glyph positions
    fn create_line(
        &self,
        text: &str,
        start_index: usize,
        end_index: usize,
        width: f64,
        height: f64,
        metrics: &FontMetrics,
    ) -> TextLine {
        let mut glyph_positions = Vec::new();
        let mut x = 0.0;
        let y = metrics.ascent;
        
        for (char_index, ch) in text.char_indices() {
            let char_width = self.estimate_char_width(ch, metrics);
            let advance = char_width + self.letter_spacing;
            
            glyph_positions.push(GlyphPosition {
                character: ch,
                char_index: start_index + char_index,
                x,
                y,
                width: char_width,
                height: metrics.ascent - metrics.descent,
                advance,
                bearing: (0.0, 0.0),
            });
            
            x += advance;
        }
        
        // Find word boundaries
        let mut word_boundaries = Vec::new();
        for (char_index, ch) in text.char_indices() {
            if ch.is_whitespace() {
                word_boundaries.push(start_index + char_index);
            }
        }
        
        TextLine {
            text: text.to_string(),
            start_index,
            end_index,
            width,
            height,
            baseline: y,
            glyph_positions,
            word_boundaries,
        }
    }
    
    /// Estimate word width
    fn estimate_word_width(&self, word: &str, metrics: &FontMetrics) -> f64 {
        word.chars()
            .map(|ch| self.estimate_char_width(ch, metrics))
            .sum::<f64>() + 
        (word.len().saturating_sub(1) as f64 * self.letter_spacing)
    }
    
    /// Estimate character width
    fn estimate_char_width(&self, ch: char, metrics: &FontMetrics) -> f64 {
        // Simple character width estimation
        match ch {
            'i' | 'j' | 'l' | 't' | 'I' => metrics.avg_char_width * 0.4,
            'm' | 'w' | 'W' => metrics.avg_char_width * 1.5,
            ' ' => self.word_spacing,
            _ => metrics.avg_char_width,
        }
    }
    
    /// Apply text transformation
    pub fn apply_text_transform(&self, text: &str, transform: crate::text::types::TextTransform) -> String {
        match transform {
            crate::text::types::TextTransform::Uppercase => text.to_uppercase(),
            crate::text::types::TextTransform::Lowercase => text.to_lowercase(),
            crate::text::types::TextTransform::Capitalize => {
                text.split_whitespace()
                    .map(|word| {
                        let mut chars: Vec<char> = word.chars().collect();
                        if let Some(first_char) = chars.get_mut(0) {
                            *first_char = first_char.to_uppercase().collect::<Vec<char>>()[0];
                        }
                        chars.into_iter().collect()
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            }
        }
    }
    
    /// Get CSS-style font string
    pub fn get_font_string(&self) -> String {
        let style = match self.font_style {
            crate::text::types::FontStyle::Normal => "",
            crate::text::types::FontStyle::Italic => "italic ",
            crate::text::types::FontStyle::Oblique => "oblique ",
        };
        
        let weight = self.font_weight.to_string();
        
        format!("{}{} {}px {}", style, weight, self.font_size, self.font_family)
    }
}

impl Default for TypographyControls {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TextJustification {
    fn default() -> Self {
        Self {
            enabled: false,
            min_word_spacing: 0.8,
            max_word_spacing: 1.5,
            min_glyph_spacing: 0.95,
            max_glyph_spacing: 1.05,
        }
    }
}

impl Default for HyphenationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            language: "en".to_string(),
            min_before: 2,
            min_after: 2,
            hyphen_char: "-".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_typography_controls_creation() {
        let controls = TypographyControls::new();
        
        assert_eq!(controls.font_family, "Arial");
        assert_eq!(controls.font_size, 12.0);
        assert_eq!(controls.font_weight, 400);
        assert_eq!(controls.line_height, 1.2);
        assert_eq!(controls.letter_spacing, 0.0);
        assert_eq!(controls.word_spacing, 0.0);
        assert_eq!(controls.alignment, crate::text::types::TextAlignment::Left);
        assert_eq!(controls.direction, crate::text::types::TextDirection::LeftToRight);
    }
    
    #[test]
    fn test_typography_from_style() {
        let style = crate::text::types::TextStyle {
            font_family: "Helvetica".to_string(),
            font_size: 16.0,
            font_weight: 700,
            font_style: crate::text::types::FontStyle::Italic,
            color: "#FF0000".to_string(),
            background_color: None,
            line_height: 1.5,
            letter_spacing: 0.5,
            word_spacing: 1.0,
            text_decoration: None,
            text_transform: None,
        };
        
        let controls = TypographyControls::from_style(&style);
        
        assert_eq!(controls.font_family, "Helvetica");
        assert_eq!(controls.font_size, 16.0);
        assert_eq!(controls.font_weight, 700);
        assert_eq!(controls.font_style, crate::text::types::FontStyle::Italic);
        assert_eq!(controls.line_height, 1.5);
        assert_eq!(controls.letter_spacing, 0.5);
        assert_eq!(controls.word_spacing, 1.0);
    }
    
    #[test]
    fn test_font_metrics() {
        let controls = TypographyControls::new();
        let metrics = controls.get_font_metrics();
        
        assert_eq!(metrics.font_family, "Arial");
        assert_eq!(metrics.font_size, 12.0);
        assert!(metrics.ascent > 0.0);
        assert!(metrics.descent < 0.0);
        assert!(metrics.line_gap > 0.0);
        assert!(metrics.cap_height > 0.0);
        assert!(metrics.x_height > 0.0);
        assert!(metrics.avg_char_width > 0.0);
        assert!(metrics.max_char_width > 0.0);
    }
    
    #[test]
    fn test_text_layout() {
        let controls = TypographyControls::new();
        let layout = controls.layout_text("Hello World", 200.0);
        
        assert_eq!(layout.lines.len(), 1);
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
        assert_eq!(layout.baselines.len(), 1);
        
        let line = &layout.lines[0];
        assert_eq!(line.text, "Hello World");
        assert!(line.width > 0.0);
        assert!(line.height > 0.0);
        assert_eq!(line.glyph_positions.len(), 11); // "Hello World" has 11 chars
    }
    
    #[test]
    fn test_text_layout_wrapping() {
        let mut controls = TypographyControls::new();
        controls.font_size = 20.0;
        
        let layout = controls.layout_text("Hello World This is a long text", 100.0);
        
        assert!(layout.lines.len() > 1); // Should wrap
        assert!(layout.width <= 100.0); // Should not exceed max width
    }
    
    #[test]
    fn test_text_transform() {
        let controls = TypographyControls::new();
        
        let uppercase = controls.apply_text_transform("hello", crate::text::types::TextTransform::Uppercase);
        assert_eq!(uppercase, "HELLO");
        
        let lowercase = controls.apply_text_transform("HELLO", crate::text::types::TextTransform::Lowercase);
        assert_eq!(lowercase, "hello");
        
        let capitalize = controls.apply_text_transform("hello world", crate::text::types::TextTransform::Capitalize);
        assert_eq!(capitalize, "Hello World");
    }
    
    #[test]
    fn test_font_string() {
        let controls = TypographyControls::new();
        let font_string = controls.get_font_string();
        
        assert!(font_string.contains("Arial"));
        assert!(font_string.contains("400"));
        assert!(font_string.contains("12px"));
    }
    
    #[test]
    fn test_char_width_estimation() {
        let controls = TypographyControls::new();
        let metrics = controls.get_font_metrics();
        
        let narrow_char = controls.estimate_char_width('i', &metrics);
        let wide_char = controls.estimate_char_width('m', &metrics);
        let normal_char = controls.estimate_char_width('a', &metrics);
        
        assert!(narrow_char < normal_char);
        assert!(wide_char > normal_char);
    }
    
    #[test]
    fn test_word_width_estimation() {
        let controls = TypographyControls::new();
        let metrics = controls.get_font_metrics();
        
        let narrow_word = controls.estimate_word_width("it", &metrics);
        let wide_word = controls.estimate_word_width("wide", &metrics);
        
        assert!(narrow_word < wide_word);
    }
    
    #[test]
    fn test_default_justification() {
        let justification = TextJustification::default();
        
        assert!(!justification.enabled);
        assert_eq!(justification.min_word_spacing, 0.8);
        assert_eq!(justification.max_word_spacing, 1.5);
        assert_eq!(justification.min_glyph_spacing, 0.95);
        assert_eq!(justification.max_glyph_spacing, 1.05);
    }
    
    #[test]
    fn test_default_hyphenation() {
        let hyphenation = HyphenationSettings::default();
        
        assert!(!hyphenation.enabled);
        assert_eq!(hyphenation.language, "en");
        assert_eq!(hyphenation.min_before, 2);
        assert_eq!(hyphenation.min_after, 2);
        assert_eq!(hyphenation.hyphen_char, "-");
    }
}
