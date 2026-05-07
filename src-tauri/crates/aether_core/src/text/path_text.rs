//! Text-on-path support
//! 
//! This module provides text layout along paths with support for
//! various path types, text orientation, and spacing controls.

use serde::{Deserialize, Serialize};

/// Text-on-path configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextOnPath {
    /// Path ID reference
    pub path_id: String,
    /// Text to place on path
    pub text: String,
    /// Starting position on path (0.0 to 1.0)
    pub start_position: f64,
    /// Whether text follows path direction
    pub follow_path: bool,
    /// Text orientation on path
    pub orientation: TextOrientation,
    /// Spacing between characters
    pub character_spacing: f64,
    /// Whether to kern characters
    pub kerning: bool,
    /// Text alignment on path
    pub alignment: PathTextAlignment,
    /// Whether to repeat text along path
    pub repeat: bool,
    /// Number of times to repeat (0 = fill path)
    pub repeat_count: usize,
    /// Path text style overrides
    pub style_overrides: Option<crate::text::types::TextStyle>,
}

/// Text orientation on path
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextOrientation {
    /// Text stays horizontal
    Horizontal,
    /// Text follows path tangent
    Tangent,
    /// Text is perpendicular to path
    Perpendicular,
    /// Text has custom rotation
    Custom(f64),
}

/// Text alignment on path
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathTextAlignment {
    /// Align to start of path
    Start,
    /// Align to center of path
    Center,
    /// Align to end of path
    End,
    /// Distribute evenly along path
    Distribute,
}

/// Positioned glyph on path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathGlyph {
    /// Character
    pub character: char,
    /// Original character index
    pub char_index: usize,
    /// Position on path
    pub position: (f64, f64),
    /// Rotation angle in radians
    pub rotation: f64,
    /// Scale factor
    pub scale: f64,
    /// Glyph width
    pub width: f64,
    /// Glyph height
    pub height: f64,
    /// Path parameter (0.0 to 1.0)
    pub path_parameter: f64,
    /// Tangent direction
    pub tangent: (f64, f64),
    /// Normal direction
    pub normal: (f64, f64),
}

/// Path text renderer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathTextRenderer {
    /// Path text configurations
    pub path_texts: Vec<TextOnPath>,
    /// Rendered glyphs
    pub rendered_glyphs: Vec<PathGlyph>,
    /// Path cache for performance
    pub path_cache: std::collections::HashMap<String, crate::shapes::paths::Path>,
    /// Whether rendering is up to date
    pub is_dirty: bool,
}

impl TextOnPath {
    /// Create new text-on-path configuration
    pub fn new(path_id: String, text: String) -> Self {
        Self {
            path_id,
            text,
            start_position: 0.0,
            follow_path: true,
            orientation: TextOrientation::Tangent,
            character_spacing: 0.0,
            kerning: true,
            alignment: PathTextAlignment::Start,
            repeat: false,
            repeat_count: 1,
            style_overrides: None,
        }
    }
    
    /// Set text orientation
    pub fn with_orientation(mut self, orientation: TextOrientation) -> Self {
        self.orientation = orientation;
        self
    }
    
    /// Set character spacing
    pub fn with_spacing(mut self, spacing: f64) -> Self {
        self.character_spacing = spacing;
        self
    }
    
    /// Set path alignment
    pub fn with_alignment(mut self, alignment: PathTextAlignment) -> Self {
        self.alignment = alignment;
        self
    }
    
    /// Set repeat settings
    pub fn with_repeat(mut self, repeat: bool, count: usize) -> Self {
        self.repeat = repeat;
        self.repeat_count = count;
        self
    }
    
    /// Get effective text (considering repeat)
    pub fn get_effective_text(&self) -> String {
        if !self.repeat {
            self.text.clone()
        } else if self.repeat_count == 0 {
            // Repeat to fill path (estimated)
            let estimated_chars = (100.0 / 10.0) as usize; // Rough estimate
            self.text.repeat(estimated_chars)
        } else {
            self.text.repeat(self.repeat_count)
        }
    }
    
    /// Validate text-on-path configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.path_id.is_empty() {
            return Err("Path ID cannot be empty".to_string());
        }
        
        if self.text.is_empty() {
            return Err("Text cannot be empty".to_string());
        }
        
        if self.start_position < 0.0 || self.start_position > 1.0 {
            return Err("Start position must be between 0.0 and 1.0".to_string());
        }
        
        if self.character_spacing < 0.0 {
            return Err("Character spacing cannot be negative".to_string());
        }
        
        Ok(())
    }
}

impl PathTextRenderer {
    /// Create new path text renderer
    pub fn new() -> Self {
        Self {
            path_texts: Vec::new(),
            rendered_glyphs: Vec::new(),
            path_cache: std::collections::HashMap::new(),
            is_dirty: true,
        }
    }
    
    /// Add text-on-path configuration
    pub fn add_text_on_path(&mut self, text_on_path: TextOnPath) {
        text_on_path.validate().unwrap();
        self.path_texts.push(text_on_path);
        self.is_dirty = true;
    }
    
    /// Remove text-on-path by path ID
    pub fn remove_text_on_path(&mut self, path_id: &str) -> bool {
        let initial_len = self.path_texts.len();
        self.path_texts.retain(|tp| tp.path_id != path_id);
        let removed = self.path_texts.len() < initial_len;
        if removed {
            self.is_dirty = true;
        }
        removed
    }
    
    /// Get text-on-path by path ID
    pub fn get_text_on_path(&self, path_id: &str) -> Option<&TextOnPath> {
        self.path_texts.iter().find(|tp| tp.path_id == path_id)
    }
    
    /// Update path cache
    pub fn update_path_cache(&mut self, path_id: &str, path: crate::shapes::paths::Path) {
        self.path_cache.insert(path_id.to_string(), path);
        self.is_dirty = true;
    }
    
    /// Render all text-on-path configurations
    pub fn render(&mut self) -> Result<(), String> {
        if !self.is_dirty {
            return Ok(());
        }
        
        self.rendered_glyphs.clear();
        
        for text_on_path in &self.path_texts {
            if let Some(path) = self.path_cache.get(&text_on_path.path_id) {
                let glyphs = self.render_text_on_path(text_on_path, path)?;
                self.rendered_glyphs.extend(glyphs);
            } else {
                return Err(format!("Path not found in cache: {}", text_on_path.path_id));
            }
        }
        
        self.is_dirty = false;
        Ok(())
    }
    
    /// Render single text-on-path configuration
    fn render_text_on_path(&self, text_on_path: &TextOnPath, path: &crate::shapes::paths::Path) -> Result<Vec<PathGlyph>, String> {
        let mut glyphs = Vec::new();
        let effective_text = text_on_path.get_effective_text();
        let path_length = path.length();
        
        if path_length == 0.0 {
            return Ok(glyphs);
        }
        
        // Calculate starting position based on alignment
        let start_param = match text_on_path.alignment {
            PathTextAlignment::Start => text_on_path.start_position,
            PathTextAlignment::Center => text_on_path.start_position - 0.5,
            PathTextAlignment::End => text_on_path.start_position - 1.0,
            PathTextAlignment::Distribute => text_on_path.start_position,
        }.clamp(0.0, 1.0);
        
        // Calculate character spacing
        let char_count = effective_text.chars().count();
        let total_char_spacing = if char_count > 1 {
            text_on_path.character_spacing * (char_count - 1) as f64
        } else {
            0.0
        };
        
        let available_length = path_length * (1.0 - start_param);
        let char_width = if char_count > 0 {
            (available_length - total_char_spacing) / char_count as f64
        } else {
            0.0
        };
        
        let mut current_param = start_param;
        
        for (char_index, character) in effective_text.chars().enumerate() {
            // Get position and tangent at current parameter
            let (position, tangent, normal) = self.get_path_geometry(path, current_param);
            
            // Calculate rotation based on orientation
            let rotation = match text_on_path.orientation {
                TextOrientation::Horizontal => 0.0,
                TextOrientation::Tangent => tangent.1.atan2(tangent.0),
                TextOrientation::Perpendicular => (tangent.1.atan2(tangent.0) + std::f64::consts::PI / 2.0),
                TextOrientation::Custom(angle) => angle,
            };
            
            // Estimate glyph dimensions
            let glyph_width = char_width;
            let glyph_height = text_on_path.style_overrides
                .as_ref()
                .map(|style| style.font_size)
                .unwrap_or(12.0);
            
            // Create path glyph
            let glyph = PathGlyph {
                character,
                char_index,
                position,
                rotation,
                scale: 1.0,
                width: glyph_width,
                height: glyph_height,
                path_parameter: current_param,
                tangent,
                normal,
            };
            
            glyphs.push(glyph);
            
            // Advance to next character position
            let advance_distance = glyph_width + text_on_path.character_spacing;
            let advance_param = advance_distance / path_length;
            current_param += advance_param;
            
            // Stop if we've reached the end of the path
            if current_param >= 1.0 {
                break;
            }
        }
        
        Ok(glyphs)
    }
    
    /// Get path geometry at specific parameter
    fn get_path_geometry(&self, path: &crate::shapes::paths::Path, parameter: f64) -> ((f64, f64), (f64, f64), (f64, f64)) {
        // Sample points along path to find position at parameter
        let samples = path.sample_points(100);
        
        if samples.is_empty() {
            return ((0.0, 0.0), (1.0, 0.0), (0.0, 1.0));
        }
        
        let index = (parameter * (samples.len() - 1) as f64).round() as usize;
        let index = index.min(samples.len() - 1);
        
        let position = samples[index];
        
        // Calculate tangent (direction to next point)
        let next_index = (index + 1).min(samples.len() - 1);
        let next_position = samples[next_index];
        let tangent = (
            next_position.0 - position.0,
            next_position.1 - position.1,
        );
        
        // Calculate normal (perpendicular to tangent)
        let normal = (-tangent.1, tangent.0);
        
        (position, tangent, normal)
    }
    
    /// Get rendered glyphs
    pub fn get_rendered_glyphs(&self) -> &[PathGlyph] {
        &self.rendered_glyphs
    }
    
    /// Get glyphs for specific path
    pub fn get_glyphs_for_path(&self, path_id: &str) -> Vec<&PathGlyph> {
        self.rendered_glyphs
            .iter()
            .filter(|glyph| {
                // This would need to track which glyph belongs to which path
                // For now, return all glyphs
                true
            })
            .collect()
    }
    
    /// Clear all text-on-path configurations
    pub fn clear(&mut self) {
        self.path_texts.clear();
        self.rendered_glyphs.clear();
        self.is_dirty = true;
    }
    
    /// Mark as dirty (needs re-rendering)
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }
    
    /// Check if rendering is up to date
    pub fn is_up_to_date(&self) -> bool {
        !self.is_dirty
    }
    
    /// Get total glyph count
    pub fn glyph_count(&self) -> usize {
        self.rendered_glyphs.len()
    }
    
    /// Get bounds of rendered text
    pub fn get_bounds(&self) -> Option<crate::shapes::primitives::transform::BoundingBox> {
        if self.rendered_glyphs.is_empty() {
            return None;
        }
        
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        
        for glyph in &self.rendered_glyphs {
            let half_width = glyph.width / 2.0;
            let half_height = glyph.height / 2.0;
            
            min_x = min_x.min(glyph.position.0 - half_width);
            min_y = min_y.min(glyph.position.1 - half_height);
            max_x = max_x.max(glyph.position.0 + half_width);
            max_y = max_y.max(glyph.position.1 + half_height);
        }
        
        Some(crate::shapes::primitives::transform::BoundingBox::new(min_x, min_y, max_x, max_y))
    }
    
    /// Validate all text-on-path configurations
    pub fn validate(&self) -> Result<(), String> {
        for (i, text_on_path) in self.path_texts.iter().enumerate() {
            text_on_path.validate().map_err(|e| format!("Text on path {}: {}", i, e))?;
        }
        Ok(())
    }
}

impl Default for PathTextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::paths::PathBuilder;
    
    #[test]
    fn test_text_on_path_creation() {
        let text_on_path = TextOnPath::new(
            "path1".to_string(),
            "Hello World".to_string(),
        );
        
        assert_eq!(text_on_path.path_id, "path1");
        assert_eq!(text_on_path.text, "Hello World");
        assert_eq!(text_on_path.start_position, 0.0);
        assert!(text_on_path.follow_path);
        assert_eq!(text_on_path.orientation, TextOrientation::Tangent);
        assert_eq!(text_on_path.character_spacing, 0.0);
        assert!(text_on_path.kerning);
        assert_eq!(text_on_path.alignment, PathTextAlignment::Start);
        assert!(!text_on_path.repeat);
        assert_eq!(text_on_path.repeat_count, 1);
    }
    
    #[test]
    fn test_text_on_path_builder() {
        let text_on_path = TextOnPath::new("path1".to_string(), "Hello".to_string())
            .with_orientation(TextOrientation::Horizontal)
            .with_spacing(2.0)
            .with_alignment(PathTextAlignment::Center)
            .with_repeat(true, 3);
        
        assert_eq!(text_on_path.orientation, TextOrientation::Horizontal);
        assert_eq!(text_on_path.character_spacing, 2.0);
        assert_eq!(text_on_path.alignment, PathTextAlignment::Center);
        assert!(text_on_path.repeat);
        assert_eq!(text_on_path.repeat_count, 3);
    }
    
    #[test]
    fn test_effective_text() {
        let text_on_path = TextOnPath::new("path1".to_string(), "Hello".to_string());
        
        assert_eq!(text_on_path.get_effective_text(), "Hello");
        
        let repeat_text = text_on_path.with_repeat(true, 3);
        assert_eq!(repeat_text.get_effective_text(), "HelloHelloHello");
        
        let fill_text = text_on_path.with_repeat(true, 0);
        let fill_effective = fill_text.get_effective_text();
        assert!(fill_effective.len() > "Hello".len());
    }
    
    #[test]
    fn test_text_on_path_validation() {
        let text_on_path = TextOnPath::new("path1".to_string(), "Hello".to_string());
        assert!(text_on_path.validate().is_ok());
        
        let mut invalid = text_on_path.clone();
        invalid.path_id = "".to_string();
        assert!(invalid.validate().is_err());
        
        invalid.path_id = "path1".to_string();
        invalid.text = "".to_string();
        assert!(invalid.validate().is_err());
        
        invalid.text = "Hello".to_string();
        invalid.start_position = -1.0;
        assert!(invalid.validate().is_err());
        
        invalid.start_position = 2.0;
        assert!(invalid.validate().is_err());
    }
    
    #[test]
    fn test_path_text_renderer() {
        let mut renderer = PathTextRenderer::new();
        
        assert!(renderer.path_texts.is_empty());
        assert!(renderer.rendered_glyphs.is_empty());
        assert!(renderer.is_dirty);
        
        let text_on_path = TextOnPath::new("path1".to_string(), "Hello".to_string());
        renderer.add_text_on_path(text_on_path);
        
        assert_eq!(renderer.path_texts.len(), 1);
        assert!(renderer.is_dirty);
        
        assert!(renderer.remove_text_on_path("path1"));
        assert_eq!(renderer.path_texts.len(), 0);
        assert!(!renderer.remove_text_on_path("nonexistent"));
    }
    
    #[test]
    fn test_path_cache() {
        let mut renderer = PathTextRenderer::new();
        
        let path = PathBuilder::new()
            .move_to(0.0, 0.0)
            .line_to(100.0, 0.0)
            .line_to(100.0, 100.0)
            .line_to(0.0, 100.0)
            .close()
            .build();
        
        renderer.update_path_cache("path1".to_string(), path.clone());
        
        assert!(renderer.path_cache.contains_key("path1"));
        assert_eq!(renderer.path_cache.len(), 1);
    }
    
    #[test]
    fn test_render_with_path() {
        let mut renderer = PathTextRenderer::new();
        
        // Create a simple path
        let path = PathBuilder::new()
            .move_to(0.0, 50.0)
            .line_to(200.0, 50.0)
            .build();
        
        renderer.update_path_cache("path1".to_string(), path);
        
        let text_on_path = TextOnPath::new("path1".to_string(), "Hello".to_string())
            .with_orientation(TextOrientation::Tangent);
        
        renderer.add_text_on_path(text_on_path);
        
        let result = renderer.render();
        assert!(result.is_ok());
        assert!(!renderer.is_dirty);
        assert!(renderer.glyph_count() > 0);
    }
    
    #[test]
    fn test_path_geometry() {
        let renderer = PathTextRenderer::new();
        
        // Create a simple horizontal line
        let path = PathBuilder::new()
            .move_to(0.0, 50.0)
            .line_to(100.0, 50.0)
            .build();
        
        let (position, tangent, normal) = renderer.get_path_geometry(&path, 0.5);
        
        // Position should be around the middle
        assert!((position.0 - 50.0).abs() < 10.0);
        assert_eq!(position.1, 50.0);
        
        // Tangent should point right
        assert!(tangent.0 > 0.0);
        assert!((tangent.1).abs() < f64::EPSILON);
        
        // Normal should point up
        assert!((normal.0).abs() < f64::EPSILON);
        assert!(normal.1 > 0.0);
    }
    
    #[test]
    fn test_get_bounds() {
        let mut renderer = PathTextRenderer::new();
        
        // Initially no bounds
        assert!(renderer.get_bounds().is_none());
        
        // Create a simple path and text
        let path = PathBuilder::new()
            .move_to(0.0, 50.0)
            .line_to(100.0, 50.0)
            .build();
        
        renderer.update_path_cache("path1".to_string(), path);
        
        let text_on_path = TextOnPath::new("path1".to_string(), "Hello".to_string());
        renderer.add_text_on_path(text_on_path);
        
        let _ = renderer.render();
        
        // Should have bounds now
        let bounds = renderer.get_bounds();
        assert!(bounds.is_some());
        
        let bounds = bounds.unwrap();
        assert!(bounds.width() > 0.0);
        assert!(bounds.height() > 0.0);
    }
    
    #[test]
    fn test_clear() {
        let mut renderer = PathTextRenderer::new();
        
        let text_on_path = TextOnPath::new("path1".to_string(), "Hello".to_string());
        renderer.add_text_on_path(text_on_path);
        
        assert_eq!(renderer.path_texts.len(), 1);
        
        renderer.clear();
        
        assert_eq!(renderer.path_texts.len(), 0);
        assert_eq!(renderer.rendered_glyphs.len(), 0);
        assert!(renderer.is_dirty);
    }
}
