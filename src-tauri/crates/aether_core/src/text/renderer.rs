//! Text rendering system
//! 
//! This module provides text rendering capabilities including
//! glyph rendering, text layout, and canvas integration.

use serde::{Deserialize, Serialize};

/// Glyph rendering information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphRenderInfo {
    /// Character
    pub character: char,
    /// Character index
    pub char_index: usize,
    /// Render position
    pub position: (f64, f64),
    /// Glyph size
    pub size: (f64, f64),
    /// Rotation angle in radians
    pub rotation: f64,
    /// Scale factor
    pub scale: f64,
    /// Color (RGBA)
    pub color: (f64, f64, f64, f64),
    /// Opacity (0.0-1.0)
    pub opacity: f64,
    /// Blur amount
    pub blur: f64,
    /// Baseline offset
    pub baseline_offset: f64,
    /// Tracking (letter spacing)
    pub tracking: f64,
}

/// Text renderer configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextRenderer {
    /// Rendering backend type
    pub backend: RenderBackend,
    /// Anti-aliasing enabled
    pub anti_aliasing: bool,
    /// Subpixel positioning
    pub subpixel_positioning: bool,
    /// Hinting mode
    pub hinting: HintingMode,
    /// Color space for rendering
    pub color_space: ColorSpace,
    /// Cache for rendered glyphs
    pub glyph_cache: GlyphCache,
    /// Performance settings
    pub performance: PerformanceSettings,
}

/// Rendering backend types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RenderBackend {
    /// CPU-based rendering
    Cpu,
    /// GPU-based rendering
    Gpu,
    /// Hybrid rendering
    Hybrid,
}

/// Font hinting modes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HintingMode {
    /// No hinting
    None,
    /// Light hinting
    Light,
    /// Normal hinting
    Normal,
    /// Full hinting
    Full,
}

/// Color space for text rendering
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorSpace {
    /// sRGB color space
    Srgb,
    /// Linear RGB color space
    LinearRgb,
    /// Display P3 color space
    DisplayP3,
    /// ACES color space
    Aces,
}

/// Glyph cache for performance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphCache {
    /// Cached glyphs
    pub glyphs: std::collections::HashMap<String, CachedGlyph>,
    /// Maximum cache size
    pub max_size: usize,
    /// Cache hit count
    pub hits: u64,
    /// Cache miss count
    pub misses: u64,
}

/// Cached glyph data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedGlyph {
    /// Glyph key
    pub key: String,
    /// Rendered glyph data
    pub data: Vec<u8>,
    /// Glyph metrics
    pub metrics: GlyphMetrics,
    /// Last access time
    pub last_access: u64,
    /// Access frequency
    pub frequency: u32,
}

/// Glyph metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphMetrics {
    /// Glyph width
    pub width: f64,
    /// Glyph height
    pub height: f64,
    /// Horizontal bearing
    pub bearing_x: f64,
    /// Vertical bearing
    pub bearing_y: f64,
    /// Advance width
    pub advance: f64,
    /// Left side bearing
    pub lsb: f64,
}

/// Performance settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Maximum batch size for rendering
    pub max_batch_size: usize,
    /// Enable parallel rendering
    pub parallel_rendering: bool,
    /// Number of render threads
    pub render_threads: usize,
    /// Enable instanced rendering
    pub instanced_rendering: bool,
    /// Texture atlas size
    pub texture_atlas_size: (u32, u32),
}

/// Individual glyph renderer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphRenderer {
    /// Font family
    pub font_family: String,
    /// Font size
    pub font_size: f64,
    /// Font weight
    pub font_weight: u16,
    /// Font style
    pub font_style: crate::text::types::FontStyle,
    /// Rendering mode
    pub render_mode: GlyphRenderMode,
    /// Quality settings
    pub quality: RenderQuality,
}

/// Glyph rendering modes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GlyphRenderMode {
    /// Fill only
    Fill,
    /// Stroke only
    Stroke,
    /// Fill and stroke
    FillAndStroke,
    /// Outline only
    Outline,
}

/// Rendering quality levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RenderQuality {
    /// Low quality (fast)
    Low,
    /// Medium quality
    Medium,
    /// High quality (slow)
    High,
    /// Ultra quality (very slow)
    Ultra,
}

impl TextRenderer {
    /// Create new text renderer
    pub fn new() -> Self {
        Self {
            backend: RenderBackend::Cpu,
            anti_aliasing: true,
            subpixel_positioning: true,
            hinting: HintingMode::Normal,
            color_space: ColorSpace::Srgb,
            glyph_cache: GlyphCache::new(),
            performance: PerformanceSettings::default(),
        }
    }
    
    /// Create text renderer with specific backend
    pub fn with_backend(backend: RenderBackend) -> Self {
        Self {
            backend,
            anti_aliasing: true,
            subpixel_positioning: true,
            hinting: HintingMode::Normal,
            color_space: ColorSpace::Srgb,
            glyph_cache: GlyphCache::new(),
            performance: PerformanceSettings::default(),
        }
    }
    
    /// Render text layer to render infos
    pub fn render_text_layer(&mut self, text_layer: &crate::text::types::TextLayer, time: f64) -> Vec<GlyphRenderInfo> {
        let mut render_infos = Vec::new();
        
        if !text_layer.visible || text_layer.opacity == 0.0 {
            return render_infos;
        }
        
        // Get text layout
        let typography = crate::text::typography::TypographyControls::from_style(&text_layer.style);
        let layout = typography.layout_text(&text_layer.content.text, text_layer.size.0);
        
        // Apply animations
        let animated_values = self.get_animated_values(text_layer, time);
        
        // Generate render infos for each line
        for line in &layout.lines {
            for glyph_pos in &line.glyph_positions {
                let render_info = self.create_render_info(
                    glyph_pos,
                    text_layer,
                    &layout.metrics,
                    &animated_values,
                );
                render_infos.push(render_info);
            }
        }
        
        // Apply path text if present
        if let Some(path_text) = &text_layer.path_text {
            let path_glyphs = self.render_path_text(path_text, text_layer, time);
            render_infos.extend(path_glyphs);
        }
        
        render_infos
    }
    
    /// Create render info for a glyph
    fn create_render_info(
        &self,
        glyph_pos: &crate::text::typography::GlyphPosition,
        text_layer: &crate::text::types::TextLayer,
        metrics: &crate::text::typography::FontMetrics,
        animated_values: &std::collections::HashMap<crate::text::animation::AnimationType, crate::text::animation::AnimationValue>,
    ) -> GlyphRenderInfo {
        // Base position
        let mut position = (
            text_layer.position.0 + glyph_pos.x,
            text_layer.position.1 + glyph_pos.y,
        );
        
        // Apply position animation
        if let Some(crate::text::animation::AnimationValue::Vector2(dx, dy)) = 
            animated_values.get(&crate::text::animation::AnimationType::Position) {
            position.0 += dx;
            position.1 += dy;
        }
        
        // Base size
        let mut size = (glyph_pos.width, glyph_pos.height);
        
        // Apply scale animation
        if let Some(crate::text::animation::AnimationValue::Float(scale)) = 
            animated_values.get(&crate::text::animation::AnimationType::Scale) {
            size.0 *= scale;
            size.1 *= scale;
        }
        
        // Base rotation
        let mut rotation = text_layer.rotation.to_radians();
        
        // Apply rotation animation
        if let Some(crate::text::animation::AnimationValue::Float(rot)) = 
            animated_values.get(&crate::text::animation::AnimationType::Rotation) {
            rotation += rot.to_radians();
        }
        
        // Base color
        let mut color = self.parse_color(&text_layer.style.color);
        
        // Apply color animation
        if let Some(crate::text::animation::AnimationValue::Color(r, g, b, a)) = 
            animated_values.get(&crate::text::animation::AnimationType::Color) {
            color = (r, g, b, a);
        }
        
        // Base opacity
        let mut opacity = text_layer.opacity;
        
        // Apply opacity animation
        if let Some(crate::text::animation::AnimationValue::Float(op)) = 
            animated_values.get(&crate::text::animation::AnimationType::Opacity) {
            opacity *= op;
        }
        
        // Base blur
        let mut blur = 0.0;
        
        // Apply blur animation
        if let Some(crate::text::animation::AnimationValue::Blur(blur_val)) = 
            animated_values.get(&crate::text::animation::AnimationType::Blur) {
            blur = blur_val;
        }
        
        // Baseline offset
        let baseline_offset = glyph_pos.y - metrics.ascent;
        
        // Tracking animation
        let mut tracking = 0.0;
        if let Some(crate::text::animation::AnimationValue::Float(track)) = 
            animated_values.get(&crate::text::animation::AnimationType::Tracking) {
            tracking = track;
        }
        
        GlyphRenderInfo {
            character: glyph_pos.character,
            char_index: glyph_pos.char_index,
            position,
            size,
            rotation,
            scale: 1.0,
            color,
            opacity,
            blur,
            baseline_offset,
            tracking,
        }
    }
    
    /// Render text on path
    fn render_path_text(
        &self,
        path_text: &crate::text::path_text::TextOnPath,
        text_layer: &crate::text::types::TextLayer,
        time: f64,
    ) -> Vec<GlyphRenderInfo> {
        let mut render_infos = Vec::new();
        
        // This would integrate with the PathTextRenderer
        // For now, return empty as placeholder
        render_infos
    }
    
    /// Get animated values for text layer at specific time
    fn get_animated_values(
        &self,
        text_layer: &crate::text::types::TextLayer,
        time: f64,
    ) -> std::collections::HashMap<crate::text::animation::AnimationType, crate::text::animation::AnimationValue> {
        let mut values = std::collections::HashMap::new();
        
        // Create a temporary animator to evaluate animations
        let mut animator = crate::text::animation::TextAnimator::new();
        for animation in &text_layer.animations {
            animator.add_animation(animation.clone());
        }
        
        // Update animator and get values for each character
        animator.global_time = time;
        
        // For simplicity, get values for first character
        // In a real implementation, this would be more sophisticated
        if let Some(char_values) = animator.get_character_values(0) {
            values.extend(char_values);
        }
        
        values
    }
    
    /// Parse color string to RGBA tuple
    fn parse_color(&self, color_str: &str) -> (f64, f64, f64, f64) {
        // Simple hex color parsing
        if color_str.starts_with('#') {
            let hex = &color_str[1..];
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
                return (r, g, b, 1.0);
            }
        }
        
        // Default to black
        (0.0, 0.0, 0.0, 1.0)
    }
    
    /// Clear glyph cache
    pub fn clear_cache(&mut self) {
        self.glyph_cache.clear();
    }
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, u64, u64) {
        (self.glyph_cache.glyphs.len(), self.glyph_cache.hits, self.glyph_cache.misses)
    }
    
    /// Optimize cache based on usage
    pub fn optimize_cache(&mut self) {
        self.glyph_cache.optimize();
    }
    
    /// Set rendering quality
    pub fn set_quality(&mut self, quality: RenderQuality) {
        match quality {
            RenderQuality::Low => {
                self.anti_aliasing = false;
                self.subpixel_positioning = false;
                self.hinting = HintingMode::None;
            }
            RenderQuality::Medium => {
                self.anti_aliasing = true;
                self.subpixel_positioning = false;
                self.hinting = HintingMode::Light;
            }
            RenderQuality::High => {
                self.anti_aliasing = true;
                self.subpixel_positioning = true;
                self.hinting = HintingMode::Normal;
            }
            RenderQuality::Ultra => {
                self.anti_aliasing = true;
                self.subpixel_positioning = true;
                self.hinting = HintingMode::Full;
            }
        }
    }
    
    /// Validate renderer configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.performance.max_batch_size == 0 {
            return Err("Max batch size must be greater than 0".to_string());
        }
        
        if self.performance.render_threads == 0 {
            return Err("Render threads must be greater than 0".to_string());
        }
        
        if self.performance.texture_atlas_size.0 == 0 || self.performance.texture_atlas_size.1 == 0 {
            return Err("Texture atlas size must be greater than 0".to_string());
        }
        
        Ok(())
    }
}

impl GlyphCache {
    /// Create new glyph cache
    pub fn new() -> Self {
        Self {
            glyphs: std::collections::HashMap::new(),
            max_size: 1000,
            hits: 0,
            misses: 0,
        }
    }
    
    /// Get cached glyph
    pub fn get(&mut self, key: &str) -> Option<&CachedGlyph> {
        if let Some(glyph) = self.glyphs.get_mut(key) {
            glyph.last_access = self.get_current_time();
            glyph.frequency += 1;
            self.hits += 1;
            Some(glyph)
        } else {
            self.misses += 1;
            None
        }
    }
    
    /// Add glyph to cache
    pub fn add(&mut self, key: String, glyph: CachedGlyph) {
        // Remove oldest if cache is full
        if self.glyphs.len() >= self.max_size {
            self.remove_oldest();
        }
        
        self.glyphs.insert(key, glyph);
    }
    
    /// Remove glyph from cache
    pub fn remove(&mut self, key: &str) -> Option<CachedGlyph> {
        self.glyphs.remove(key)
    }
    
    /// Clear cache
    pub fn clear(&mut self) {
        self.glyphs.clear();
        self.hits = 0;
        self.misses = 0;
    }
    
    /// Optimize cache based on usage
    pub fn optimize(&mut self) {
        if self.glyphs.len() <= self.max_size {
            return;
        }
        
        // Sort by frequency and remove least used
        let mut glyphs: Vec<_> = self.glyphs.iter().collect();
        glyphs.sort_by(|a, b| a.1.frequency.cmp(&b.1.frequency));
        
        let remove_count = self.glyphs.len() - self.max_size;
        for (key, _) in glyphs.iter().take(remove_count) {
            self.glyphs.remove(*key);
        }
    }
    
    /// Remove oldest glyph
    fn remove_oldest(&mut self) {
        if let Some(oldest_key) = self.glyphs
            .iter()
            .min_by_key(|(_, glyph)| glyph.last_access)
            .map(|(key, _)| *key) {
            self.glyphs.remove(&oldest_key);
        }
    }
    
    /// Get current time (simplified)
    fn get_current_time(&self) -> u64 {
        // In a real implementation, this would use actual system time
        0
    }
    
    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl GlyphRenderer {
    /// Create new glyph renderer
    pub fn new() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 12.0,
            font_weight: 400,
            font_style: crate::text::types::FontStyle::Normal,
            render_mode: GlyphRenderMode::Fill,
            quality: RenderQuality::Medium,
        }
    }
    
    /// Create glyph renderer with font settings
    pub fn with_font(font_family: String, font_size: f64) -> Self {
        Self {
            font_family,
            font_size,
            font_weight: 400,
            font_style: crate::text::types::FontStyle::Normal,
            render_mode: GlyphRenderMode::Fill,
            quality: RenderQuality::Medium,
        }
    }
    
    /// Set render mode
    pub fn with_render_mode(mut self, mode: GlyphRenderMode) -> Self {
        self.render_mode = mode;
        self
    }
    
    /// Set quality
    pub fn with_quality(mut self, quality: RenderQuality) -> Self {
        self.quality = quality;
        self
    }
    
    /// Create glyph key for caching
    pub fn create_glyph_key(&self, character: char) -> String {
        format!(
            "{}_{:.2}_{}_{}_{:?}_{:?}",
            self.font_family,
            self.font_size,
            self.font_weight,
            self.font_style as u8,
            self.render_mode,
            self.quality
        )
    }
    
    /// Validate glyph renderer
    pub fn validate(&self) -> Result<(), String> {
        if self.font_family.is_empty() {
            return Err("Font family cannot be empty".to_string());
        }
        
        if self.font_size <= 0.0 {
            return Err("Font size must be positive".to_string());
        }
        
        if self.font_weight < 100 || self.font_weight > 900 {
            return Err("Font weight must be between 100 and 900".to_string());
        }
        
        Ok(())
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            parallel_rendering: true,
            render_threads: num_cpus::get(),
            instanced_rendering: true,
            texture_atlas_size: (2048, 2048),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_renderer_creation() {
        let renderer = TextRenderer::new();
        
        assert_eq!(renderer.backend, RenderBackend::Cpu);
        assert!(renderer.anti_aliasing);
        assert!(renderer.subpixel_positioning);
        assert_eq!(renderer.hinting, HintingMode::Normal);
        assert_eq!(renderer.color_space, ColorSpace::Srgb);
    }
    
    #[test]
    fn test_text_renderer_with_backend() {
        let renderer = TextRenderer::with_backend(RenderBackend::Gpu);
        
        assert_eq!(renderer.backend, RenderBackend::Gpu);
        assert!(renderer.anti_aliasing);
        assert!(renderer.subpixel_positioning);
    }
    
    #[test]
    fn test_glyph_cache() {
        let mut cache = GlyphCache::new();
        
        assert_eq!(cache.glyphs.len(), 0);
        assert_eq!(cache.hits, 0);
        assert_eq!(cache.misses, 0);
        assert_eq!(cache.hit_rate(), 0.0);
        
        // Test miss
        assert!(cache.get("test").is_none());
        assert_eq!(cache.misses, 1);
        
        // Test add and hit
        let glyph = CachedGlyph {
            key: "test".to_string(),
            data: vec![1, 2, 3],
            metrics: GlyphMetrics {
                width: 10.0,
                height: 12.0,
                bearing_x: 0.0,
                bearing_y: 10.0,
                advance: 10.0,
                lsb: 0.0,
            },
            last_access: 0,
            frequency: 0,
        };
        
        cache.add("test".to_string(), glyph);
        
        let cached = cache.get("test");
        assert!(cached.is_some());
        assert_eq!(cache.hits, 1);
        assert!(cache.hit_rate() > 0.0);
        
        // Test remove
        let removed = cache.remove("test");
        assert!(removed.is_some());
        assert!(cache.get("test").is_none());
    }
    
    #[test]
    fn test_glyph_renderer_creation() {
        let renderer = GlyphRenderer::new();
        
        assert_eq!(renderer.font_family, "Arial");
        assert_eq!(renderer.font_size, 12.0);
        assert_eq!(renderer.font_weight, 400);
        assert_eq!(renderer.font_style, crate::text::types::FontStyle::Normal);
        assert_eq!(renderer.render_mode, GlyphRenderMode::Fill);
        assert_eq!(renderer.quality, RenderQuality::Medium);
    }
    
    #[test]
    fn test_glyph_renderer_with_font() {
        let renderer = GlyphRenderer::with_font("Helvetica".to_string(), 16.0);
        
        assert_eq!(renderer.font_family, "Helvetica");
        assert_eq!(renderer.font_size, 16.0);
    }
    
    #[test]
    fn test_glyph_key_creation() {
        let renderer = GlyphRenderer::with_font("Arial".to_string(), 12.0);
        let key = renderer.create_glyph_key('A');
        
        assert!(key.contains("Arial"));
        assert!(key.contains("12.00"));
        assert!(key.contains("400"));
    }
    
    #[test]
    fn test_color_parsing() {
        let renderer = TextRenderer::new();
        
        let color = renderer.parse_color("#FF0000");
        assert_eq!(color, (1.0, 0.0, 0.0, 1.0));
        
        let color = renderer.parse_color("#00FF00");
        assert_eq!(color, (0.0, 1.0, 0.0, 1.0));
        
        let color = renderer.parse_color("#0000FF");
        assert_eq!(color, (0.0, 0.0, 1.0, 1.0));
        
        let color = renderer.parse_color("invalid");
        assert_eq!(color, (0.0, 0.0, 0.0, 1.0));
    }
    
    #[test]
    fn test_quality_settings() {
        let mut renderer = TextRenderer::new();
        
        renderer.set_quality(RenderQuality::Low);
        assert!(!renderer.anti_aliasing);
        assert!(!renderer.subpixel_positioning);
        assert_eq!(renderer.hinting, HintingMode::None);
        
        renderer.set_quality(RenderQuality::High);
        assert!(renderer.anti_aliasing);
        assert!(renderer.subpixel_positioning);
        assert_eq!(renderer.hinting, HintingMode::Normal);
        
        renderer.set_quality(RenderQuality::Ultra);
        assert!(renderer.anti_aliasing);
        assert!(renderer.subpixel_positioning);
        assert_eq!(renderer.hinting, HintingMode::Full);
    }
    
    #[test]
    fn test_text_renderer_validation() {
        let renderer = TextRenderer::new();
        assert!(renderer.validate().is_ok());
        
        let mut invalid_renderer = renderer.clone();
        invalid_renderer.performance.max_batch_size = 0;
        assert!(invalid_renderer.validate().is_err());
        
        invalid_renderer = renderer.clone();
        invalid_renderer.performance.render_threads = 0;
        assert!(invalid_renderer.validate().is_err());
        
        invalid_renderer = renderer.clone();
        invalid_renderer.performance.texture_atlas_size = (0, 2048);
        assert!(invalid_renderer.validate().is_err());
    }
    
    #[test]
    fn test_glyph_renderer_validation() {
        let renderer = GlyphRenderer::new();
        assert!(renderer.validate().is_ok());
        
        let mut invalid_renderer = renderer.clone();
        invalid_renderer.font_family = "".to_string();
        assert!(invalid_renderer.validate().is_err());
        
        invalid_renderer = renderer.clone();
        invalid_renderer.font_size = -1.0;
        assert!(invalid_renderer.validate().is_err());
        
        invalid_renderer = renderer.clone();
        invalid_renderer.font_weight = 50; // Below 100
        assert!(invalid_renderer.validate().is_err());
        
        invalid_renderer.font_weight = 1000; // Above 900
        assert!(invalid_renderer.validate().is_err());
    }
    
    #[test]
    fn test_render_text_layer_basic() {
        let mut renderer = TextRenderer::new();
        
        let text_layer = crate::text::types::TextLayer::new(
            "test".to_string(),
            "Test".to_string(),
            "Hello".to_string(),
        );
        
        let render_infos = renderer.render_text_layer(&text_layer, 0.0);
        
        assert!(!render_infos.is_empty());
        
        // Check that each render info has the expected character
        for (i, render_info) in render_infos.iter().enumerate() {
            assert_eq!(render_info.char_index, i);
            assert!(render_info.opacity > 0.0);
        }
    }
    
    #[test]
    fn test_render_text_layer_hidden() {
        let mut renderer = TextRenderer::new();
        
        let mut text_layer = crate::text::types::TextLayer::new(
            "test".to_string(),
            "Test".to_string(),
            "Hello".to_string(),
        );
        
        text_layer.visible = false;
        let render_infos = renderer.render_text_layer(&text_layer, 0.0);
        assert!(render_infos.is_empty());
        
        text_layer.visible = true;
        text_layer.opacity = 0.0;
        let render_infos = renderer.render_text_layer(&text_layer, 0.0);
        assert!(render_infos.is_empty());
    }
}
