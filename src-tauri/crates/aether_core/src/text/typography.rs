use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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
        Self::compute_font_metrics(&self.font_family, self.font_size, self.font_weight)
    }

    fn compute_font_metrics(font_family: &str, font_size: f64, font_weight: u16) -> FontMetrics {
        if let Some(metrics) = Self::try_load_system_font_metrics(font_family, font_size, font_weight) {
            return metrics;
        }
        Self::heuristic_font_metrics(font_family, font_size)
    }

    fn try_load_system_font_metrics(font_family: &str, font_size: f64, font_weight: u16) -> Option<FontMetrics> {
        let paths = Self::system_font_paths(font_family, font_weight);
        for path in paths {
            if let Ok(bytes) = fs::read(&path) {
                let settings = fontdue::FontSettings {
                    collection_index: 0,
                    scale: font_size as f32,
                    load_substitutions: false,
                };
                if let Ok(font) = fontdue::Font::from_bytes(bytes, settings) {
                    let hm = font.horizontal_line_metrics(font_size as f32)?;

                    // Sample common glyphs for width estimates
                    let sample_chars = ['M', 'x', 'H', 'a', 'n', 'o', '0', ' '];
                    let mut widths = Vec::new();
                    let mut max_width = 0.0f32;
                    for ch in &sample_chars {
                        if let Some(glyph) = font.chars().get(ch) {
                            let metrics = font.metrics_indexed(glyph.get(), font_size as f32);
                            widths.push(metrics.advance_width);
                            max_width = max_width.max(metrics.advance_width);
                        }
                    }
                    let avg_width = if !widths.is_empty() {
                        widths.iter().sum::<f32>() / widths.len() as f32
                    } else {
                        font_size as f32 * 0.5
                    };

                    return Some(FontMetrics {
                        font_family: font_family.to_string(),
                        font_size,
                        ascent: hm.ascent as f64,
                        descent: hm.descent as f64,
                        line_gap: hm.line_gap as f64,
                        cap_height: font_size * 0.7,
                        x_height: font_size * 0.5,
                        avg_char_width: avg_width as f64,
                        max_char_width: max_width as f64,
                        underline_position: font_size * 0.1,
                        underline_thickness: font_size * 0.05,
                    });
                }
            }
        }
        None
    }

    fn system_font_paths(font_family: &str, font_weight: u16) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let weight_suffix = match font_weight {
            100..=300 => "Light",
            400 => "Regular",
            500 => "Medium",
            600 => "SemiBold",
            700 => "Bold",
            800..=900 => "Black",
            _ => "Regular",
        };

        // macOS
        let mac_dirs = ["/System/Library/Fonts", "/Library/Fonts", "/System/Library/Fonts/Supplemental"];
        for dir in &mac_dirs {
            paths.push(PathBuf::from(format!("{}/{}.ttf", dir, font_family)));
            paths.push(PathBuf::from(format!("{}/{}-{}.ttf", dir, font_family, weight_suffix)));
            paths.push(PathBuf::from(format!("{}/{}.ttc", dir, font_family)));
            paths.push(PathBuf::from(format!("{}/{}.otf", dir, font_family)));
        }

        // Linux
        let linux_dirs = ["/usr/share/fonts/truetype", "/usr/local/share/fonts"];
        for dir in &linux_dirs {
            paths.push(PathBuf::from(format!("{}/{}/{}.ttf", dir, font_family, font_family)));
            paths.push(PathBuf::from(format!("{}/{}-{}.ttf", dir, font_family, weight_suffix)));
            paths.push(PathBuf::from(format!("{}/{}.ttf", dir, font_family)));
        }

        // Windows
        let win_dir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
        paths.push(PathBuf::from(format!("{}\\Fonts\\{}.ttf", win_dir, font_family)));
        paths.push(PathBuf::from(format!("{}\\Fonts\\{}-{}.ttf", win_dir, font_family, weight_suffix)));

        paths
    }

    fn heuristic_font_metrics(font_family: &str, font_size: f64) -> FontMetrics {
        FontMetrics {
            font_family: font_family.to_string(),
            font_size,
            ascent: font_size * 0.8,
            descent: font_size * 0.2,
            line_gap: font_size * 0.1,
            cap_height: font_size * 0.7,
            x_height: font_size * 0.5,
            avg_char_width: font_size * 0.5,
            max_char_width: font_size * 0.7,
            underline_position: font_size * 0.1,
            underline_thickness: font_size * 0.05,
        }
    }

    pub fn estimate_word_width(&self, word: &str, _metrics: &FontMetrics) -> f64 {
        word.len() as f64 * self.font_size * 0.5
    }

    pub fn layout_text(&self, _text: &str, _width: f64) -> TextLayout {
        TextLayout {
            lines: Vec::new(),
            width: 0.0,
            height: 0.0,
            baselines: Vec::new(),
            metrics: FontMetrics::default(),
        }
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
