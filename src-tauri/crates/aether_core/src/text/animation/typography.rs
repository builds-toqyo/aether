use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographyControls {
    pub font_family: String,
    pub font_size: f64,
    pub font_weight: FontWeight,
    pub line_height: f64,
    pub letter_spacing: f64,
    pub word_spacing: f64,
    pub text_align: TextAlign,
    pub text_transform: TextTransform,
    pub color: (f64, f64, f64, f64),
    pub baseline_shift: f64,
    pub kerning: bool,
    pub ligatures: bool,
    pub small_caps: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub overline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    Custom(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

impl Default for TypographyControls {
    fn default() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 16.0,
            font_weight: FontWeight::Normal,
            line_height: 1.2,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            text_align: TextAlign::Left,
            text_transform: TextTransform::None,
            color: (0.0, 0.0, 0.0, 1.0),
            baseline_shift: 0.0,
            kerning: true,
            ligatures: true,
            small_caps: false,
            underline: false,
            strikethrough: false,
            overline: false,
        }
    }
}

impl TypographyControls {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_font_family(mut self, font_family: String) -> Self {
        self.font_family = font_family;
        self
    }

    pub fn with_font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn with_font_weight(mut self, font_weight: FontWeight) -> Self {
        self.font_weight = font_weight;
        self
    }

    pub fn with_color(mut self, r: f64, g: f64, b: f64, a: f64) -> Self {
        self.color = (r, g, b, a);
        self
    }

    pub fn with_text_align(mut self, text_align: TextAlign) -> Self {
        self.text_align = text_align;
        self
    }

    pub fn get_font_weight_value(&self) -> u16 {
        match self.font_weight {
            FontWeight::Thin => 100,
            FontWeight::ExtraLight => 200,
            FontWeight::Light => 300,
            FontWeight::Normal => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
            FontWeight::Black => 900,
            FontWeight::Custom(value) => value,
        }
    }

    pub fn set_font_weight_value(&mut self, value: u16) {
        self.font_weight = match value {
            100 => FontWeight::Thin,
            200 => FontWeight::ExtraLight,
            300 => FontWeight::Light,
            400 => FontWeight::Normal,
            500 => FontWeight::Medium,
            600 => FontWeight::SemiBold,
            700 => FontWeight::Bold,
            800 => FontWeight::ExtraBold,
            900 => FontWeight::Black,
            _ => FontWeight::Custom(value),
        };
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.font_family.is_empty() {
            return Err("Font family cannot be empty".to_string());
        }

        if self.font_size <= 0.0 {
            return Err("Font size must be positive".to_string());
        }

        if self.line_height <= 0.0 {
            return Err("Line height must be positive".to_string());
        }

        if self.color.0 < 0.0 || self.color.0 > 1.0 ||
           self.color.1 < 0.0 || self.color.1 > 1.0 ||
           self.color.2 < 0.0 || self.color.2 > 1.0 ||
           self.color.3 < 0.0 || self.color.3 > 1.0 {
            return Err("Color values must be between 0.0 and 1.0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typography_controls_default() {
        let typography = TypographyControls::default();
        assert_eq!(typography.font_family, "Arial");
        assert_eq!(typography.font_size, 16.0);
        assert_eq!(typography.font_weight, FontWeight::Normal);
        assert_eq!(typography.line_height, 1.2);
        assert_eq!(typography.color, (0.0, 0.0, 0.0, 1.0));
        assert!(typography.kerning);
        assert!(typography.ligatures);
        assert!(!typography.underline);
    }

    #[test]
    fn test_typography_controls_builder() {
        let typography = TypographyControls::new()
            .with_font_family("Helvetica".to_string())
            .with_font_size(24.0)
            .with_font_weight(FontWeight::Bold)
            .with_color(1.0, 0.0, 0.0, 1.0)
            .with_text_align(TextAlign::Center);

        assert_eq!(typography.font_family, "Helvetica");
        assert_eq!(typography.font_size, 24.0);
        assert_eq!(typography.font_weight, FontWeight::Bold);
        assert_eq!(typography.color, (1.0, 0.0, 0.0, 1.0));
        assert_eq!(typography.text_align, TextAlign::Center);
    }

    #[test]
    fn test_font_weight_conversion() {
        let mut typography = TypographyControls::default();


        typography.font_weight = FontWeight::Thin;
        assert_eq!(typography.get_font_weight_value(), 100);

        typography.font_weight = FontWeight::Normal;
        assert_eq!(typography.get_font_weight_value(), 400);

        typography.font_weight = FontWeight::Bold;
        assert_eq!(typography.get_font_weight_value(), 700);

        typography.font_weight = FontWeight::Custom(750);
        assert_eq!(typography.get_font_weight_value(), 750);


        typography.set_font_weight_value(300);
        assert_eq!(typography.font_weight, FontWeight::Light);

        typography.set_font_weight_value(600);
        assert_eq!(typography.font_weight, FontWeight::SemiBold);

        typography.set_font_weight_value(950);
        assert_eq!(typography.font_weight, FontWeight::Custom(950));
    }

    #[test]
    fn test_typography_validation() {
        let valid_typography = TypographyControls::default();
        assert!(valid_typography.validate().is_ok());

        let mut invalid_typography = TypographyControls::default();
        invalid_typography.font_family = "".to_string();
        assert!(invalid_typography.validate().is_err());

        invalid_typography.font_family = "Arial".to_string();
        invalid_typography.font_size = -1.0;
        assert!(invalid_typography.validate().is_err());

        invalid_typography.font_size = 16.0;
        invalid_typography.line_height = 0.0;
        assert!(invalid_typography.validate().is_err());

        invalid_typography.line_height = 1.2;
        invalid_typography.color = (2.0, 0.0, 0.0, 1.0);
        assert!(invalid_typography.validate().is_err());
    }

    #[test]
    fn test_font_weight_variants() {
        let weights = vec![
            FontWeight::Thin,
            FontWeight::ExtraLight,
            FontWeight::Light,
            FontWeight::Normal,
            FontWeight::Medium,
            FontWeight::SemiBold,
            FontWeight::Bold,
            FontWeight::ExtraBold,
            FontWeight::Black,
            FontWeight::Custom(750),
        ];

        for weight in weights {
            let mut typography = TypographyControls::default();
            typography.font_weight = weight;
            assert_eq!(typography.font_weight, weight);
        }
    }

    #[test]
    fn test_text_transform_variants() {
        let transforms = vec![
            TextTransform::None,
            TextTransform::Uppercase,
            TextTransform::Lowercase,
            TextTransform::Capitalize,
        ];

        for transform in transforms {
            let mut typography = TypographyControls::default();
            typography.text_transform = transform;
            assert_eq!(typography.text_transform, transform);
        }
    }

    #[test]
    fn test_text_align_variants() {
        let alignments = vec![
            TextAlign::Left,
            TextAlign::Center,
            TextAlign::Right,
            TextAlign::Justify,
        ];

        for align in alignments {
            let mut typography = TypographyControls::default();
            typography.text_align = align;
            assert_eq!(typography.text_align, align);
        }
    }
}
