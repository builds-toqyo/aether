use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographyControls {

    pub font_family: String,

    pub font_size: f64,

    pub font_weight: u16,

    pub font_style: crate::text::types::FontStyle,

    pub line_height: f64,

    pub letter_spacing: f64,

    pub word_spacing: f64,

    pub paragraph_spacing: f64,

    pub alignment: crate::text::types::TextAlignment,

    pub direction: crate::text::types::TextDirection,

    pub indentation: f64,

    pub justification: TextJustification,

    pub hyphenation: HyphenationSettings,

    pub font_features: HashMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TextJustification {
    pub enabled: bool,
    pub min_word_spacing: f64,
    pub max_word_spacing: f64,
    pub min_glyph_spacing: f64,
    pub max_glyph_spacing: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HyphenationSettings {

    pub enabled: bool,

    pub language: String,

    pub min_before: usize,

    pub min_after: usize,

    pub hyphen_char: String,
}


#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FontMetrics {
    pub font_family: String,
    pub font_size: f64,
    pub ascent: f64,
    pub descent: f64,
    pub line_gap: f64,
    pub cap_height: f64,
    pub x_height: f64,
    pub avg_char_width: f64,
    pub max_char_width: f64,
    pub underline_position: f64,
    pub underline_thickness: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLayout {

    pub lines: Vec<TextLine>,

    pub width: f64,

    pub height: f64,

    pub baselines: Vec<f64>,

    pub metrics: FontMetrics,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLine {

    pub text: String,

    pub start_index: usize,

    pub end_index: usize,

    pub width: f64,

    pub height: f64,

    pub baseline: f64,

    pub glyph_positions: Vec<GlyphPosition>,

    pub word_boundaries: Vec<usize>,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphPosition {

    pub character: char,

    pub char_index: usize,

    pub x: f64,

    pub y: f64,

    pub width: f64,

    pub height: f64,

    pub advance: f64,

    pub bearing: (f64, f64),
}

impl TypographyControls {

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


    pub fn get_font_metrics(&self) -> FontMetrics {
        // TODO: Implement font metrics calculation
        FontMetrics::default()
    }

    pub fn estimate_word_width(&self, word: &str, _metrics: &FontMetrics) -> f64 {
        word.len() as f64 * self.font_size * 0.5
    }

    pub fn layout_text(&self, _text: &str, _width: f64) -> Vec<(f64, f64, f64, f64)> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
