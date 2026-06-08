use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

impl fmt::Display for TextAlignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextAlignment::Left => write!(f, "Left"),
            TextAlignment::Center => write!(f, "Center"),
            TextAlignment::Right => write!(f, "Right"),
            TextAlignment::Justify => write!(f, "Justify"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
}

impl fmt::Display for TextDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextDirection::LeftToRight => write!(f, "LTR"),
            TextDirection::RightToLeft => write!(f, "RTL"),
            TextDirection::TopToBottom => write!(f, "TTB"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {

    pub font_family: String,
    pub font_size: f64,
    pub font_weight: u16,
    pub font_style: FontStyle,
    pub color: String,
    pub background_color: Option<String>,
    pub line_height: f64,
    pub letter_spacing: f64,
    pub word_spacing: f64,
    pub text_decoration: Option<TextDecoration>,
    pub text_transform: Option<TextTransform>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontStyle {

    Normal,
    Italic,
    Oblique,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextDecoration {
    pub underline: bool,
    pub overline: bool,
    pub line_through: bool,
    pub color: Option<String>,
    pub style: TextDecorationStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextDecorationStyle {
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextTransform {

    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
    pub segments: Option<Vec<TextSegment>>,
    pub direction: TextDirection,
    pub alignment: TextAlignment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextSegment {
    pub text: String,
    pub style: Option<TextStyle>,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLayer {
    pub id: String,
    pub name: String,
    pub content: TextContent,
    pub style: TextStyle,
    pub position: (f64, f64),
    pub size: (f64, f64),
    pub rotation: f64,
    pub opacity: f64,
    pub visible: bool,
    pub locked: bool,
    pub z_index: i32,
    pub path_text: Option<crate::text::path_text::TextOnPath>,
    pub animations: Vec<crate::text::animation::CharacterAnimation>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl TextLayer {
    pub fn new(id: String, name: String, text: String) -> Self {
        Self {
            id,
            name,
            content: TextContent {
                text,
                segments: None,
                direction: TextDirection::LeftToRight,
                alignment: TextAlignment::Left,
            },
            style: TextStyle::default(),
            position: (0.0, 0.0),
            size: (100.0, 50.0),
            rotation: 0.0,
            opacity: 1.0,
            visible: true,
            locked: false,
            z_index: 0,
            path_text: None,
            animations: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn bounds(&self) -> crate::shapes::primitives::transform::BoundingBox {
        crate::shapes::primitives::transform::BoundingBox::new(
            self.position.0,
            self.position.1,
            self.position.0 + self.size.0,
            self.position.1 + self.size.1,
        )
    }

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        self.bounds().contains_point(x, y)
    }

    pub fn transform(&mut self, transform: &crate::shapes::primitives::transform::Transform) {
        let (tx, ty) = transform.transform_point(self.position.0, self.position.1);
        self.position = (tx, ty);

        self.size.0 *= transform.sx;
        self.size.1 *= transform.sy;

        self.rotation += transform.rotation.to_degrees();
    }

    pub fn transformed(&self, transform: &crate::shapes::primitives::transform::Transform) -> Self {
        let mut copy = self.clone();
        copy.transform(transform);
        copy
    }

    pub fn add_animation(&mut self, animation: crate::text::animation::CharacterAnimation) {
        self.animations.push(animation);
    }

    pub fn remove_animation(&mut self, animation_id: &str) -> bool {
        let initial_len = self.animations.len();
        self.animations.retain(|anim| anim.id != animation_id);
        self.animations.len() < initial_len
    }

    pub fn get_animation(&self, animation_id: &str) -> Option<&crate::text::animation::CharacterAnimation> {
        self.animations.iter().find(|anim| anim.id == animation_id)
    }

    pub fn get_animation_mut(&mut self, animation_id: &str) -> Option<&mut crate::text::animation::CharacterAnimation> {
        self.animations.iter_mut().find(|anim| anim.id == animation_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Text layer ID cannot be empty".to_string());
        }

        if self.content.text.is_empty() {
            return Err("Text content cannot be empty".to_string());
        }

        if self.style.font_size <= 0.0 {
            return Err("Font size must be positive".to_string());
        }

        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err("Opacity must be between 0.0 and 1.0".to_string());
        }

        if self.size.0 <= 0.0 || self.size.1 <= 0.0 {
            return Err("Layer dimensions must be positive".to_string());
        }

        Ok(())
    }


    pub fn character_count(&self) -> usize {
        self.content.text.chars().count()
    }


    pub fn word_count(&self) -> usize {
        self.content.text.split_whitespace().count()
    }


    pub fn line_count(&self) -> usize {
        self.content.text.lines().count()
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 12.0,
            font_weight: 400,
            font_style: FontStyle::Normal,
            color: "#000000".to_string(),
            background_color: None,
            line_height: 1.2,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            text_decoration: None,
            text_transform: None,
        }
    }
}

impl Default for TextContent {
    fn default() -> Self {
        Self {
            text: String::new(),
            segments: None,
            direction: TextDirection::LeftToRight,
            alignment: TextAlignment::Left,
        }
    }
}

impl Default for TextDecoration {
    fn default() -> Self {
        Self {
            underline: false,
            overline: false,
            line_through: false,
            color: None,
            style: TextDecorationStyle::Solid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_layer_creation() {
        let layer = TextLayer::new(
            "text1".to_string(),
            "Sample Text".to_string(),
            "Hello World".to_string(),
        );

        assert_eq!(layer.id, "text1");
        assert_eq!(layer.name, "Sample Text");
        assert_eq!(layer.content.text, "Hello World");
        assert!(layer.visible);
        assert_eq!(layer.opacity, 1.0);
        assert_eq!(layer.character_count(), 11);
        assert_eq!(layer.word_count(), 2);
        assert_eq!(layer.line_count(), 1);
    }

    #[test]
    fn test_text_style_default() {
        let style = TextStyle::default();

        assert_eq!(style.font_family, "Arial");
        assert_eq!(style.font_size, 12.0);
        assert_eq!(style.font_weight, 400);
        assert_eq!(style.font_style, FontStyle::Normal);
        assert_eq!(style.color, "#000000");
        assert_eq!(style.line_height, 1.2);
        assert_eq!(style.letter_spacing, 0.0);
        assert_eq!(style.word_spacing, 0.0);
    }

    #[test]
    fn test_text_alignment_display() {
        assert_eq!(format!("{}", TextAlignment::Left), "Left");
        assert_eq!(format!("{}", TextAlignment::Center), "Center");
        assert_eq!(format!("{}", TextAlignment::Right), "Right");
        assert_eq!(format!("{}", TextAlignment::Justify), "Justify");
    }

    #[test]
    fn test_text_direction_display() {
        assert_eq!(format!("{}", TextDirection::LeftToRight), "LTR");
        assert_eq!(format!("{}", TextDirection::RightToLeft), "RTL");
        assert_eq!(format!("{}", TextDirection::TopToBottom), "TTB");
    }

    #[test]
    fn test_text_layer_bounds() {
        let layer = TextLayer::new(
            "text1".to_string(),
            "Sample".to_string(),
            "Hello".to_string(),
        );

        let bounds = layer.bounds();
        assert_eq!(bounds.min_x, 0.0);
        assert_eq!(bounds.min_y, 0.0);
        assert_eq!(bounds.max_x, 100.0);
        assert_eq!(bounds.max_y, 50.0);
    }

    #[test]
    fn test_text_layer_contains_point() {
        let layer = TextLayer::new(
            "text1".to_string(),
            "Sample".to_string(),
            "Hello".to_string(),
        );

        assert!(layer.contains_point(50.0, 25.0));
        assert!(!layer.contains_point(150.0, 25.0));
        assert!(!layer.contains_point(50.0, 75.0));
    }

    #[test]
    fn test_text_layer_transform() {
        let mut layer = TextLayer::new(
            "text1".to_string(),
            "Sample".to_string(),
            "Hello".to_string(),
        );

        let transform = crate::shapes::primitives::transform::Transform::translation(10.0, 20.0);
        layer.transform(&transform);

        assert_eq!(layer.position, (10.0, 20.0));
    }

    #[test]
    fn test_text_layer_validation() {
        let valid_layer = TextLayer::new(
            "text1".to_string(),
            "Sample".to_string(),
            "Hello".to_string(),
        );

        assert!(valid_layer.validate().is_ok());

        let mut invalid_layer = TextLayer::new(
            "".to_string(),
            "Sample".to_string(),
            "Hello".to_string(),
        );
        assert!(invalid_layer.validate().is_err());

        invalid_layer.id = "text1".to_string();
        invalid_layer.content.text = "".to_string();
        assert!(invalid_layer.validate().is_err());

        invalid_layer.content.text = "Hello".to_string();
        invalid_layer.style.font_size = -1.0;
        assert!(invalid_layer.validate().is_err());

        invalid_layer.style.font_size = 12.0;
        invalid_layer.opacity = 2.0;
        assert!(invalid_layer.validate().is_err());
    }

    #[test]
    fn test_text_layer_animations() {
        let mut layer = TextLayer::new(
            "text1".to_string(),
            "Sample".to_string(),
            "Hello".to_string(),
        );

        let animation = crate::text::animation::CharacterAnimation {
            id: "anim1".to_string(),
            name: "Fade In".to_string(),
            animation_type: crate::text::animation::AnimationType::Opacity,
            keyframes: vec![],
            target_characters: vec![0, 1, 2, 3, 4],
        };

        layer.add_animation(animation.clone());
        assert_eq!(layer.animations.len(), 1);

        let retrieved = layer.get_animation("anim1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "anim1");

        let removed = layer.remove_animation("anim1");
        assert!(removed);
        assert_eq!(layer.animations.len(), 0);

        let not_removed = layer.remove_animation("nonexistent");
        assert!(!not_removed);
    }

    #[test]
    fn test_rich_text_segments() {
        let content = TextContent {
            text: "Hello World".to_string(),
            segments: Some(vec![
                TextSegment {
                    text: "Hello".to_string(),
                    style: None,
                    start: 0,
                    end: 5,
                },
                TextSegment {
                    text: " World".to_string(),
                    style: None,
                    start: 5,
                    end: 11,
                },
            ]),
            direction: TextDirection::LeftToRight,
            alignment: TextAlignment::Left,
        };

        assert_eq!(content.text, "Hello World");
        assert!(content.segments.is_some());
        assert_eq!(content.segments.as_ref().unwrap().len(), 2);
    }
}
