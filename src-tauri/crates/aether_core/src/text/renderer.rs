use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphRenderInfo {

    pub character: char,
    pub char_index: usize,
    pub position: (f64, f64),
    pub size: (f64, f64),
    pub rotation: f64,
    pub scale: f64,
    pub color: (f64, f64, f64, f64),
    pub opacity: f64,
    pub blur: f64,
    pub baseline_offset: f64,
    pub tracking: f64,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextRenderer {
    pub backend: RenderBackend,
    pub anti_aliasing: bool,
    pub subpixel_positioning: bool,
    pub hinting: HintingMode,
    pub color_space: ColorSpace,
    pub glyph_cache: GlyphCache,
    pub performance: PerformanceSettings,
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RenderBackend {
    Cpu,
    Gpu,
    Hybrid,
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HintingMode {
    None,
    Light,
    Normal,
    Full,
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorSpace {
    Srgb,
    LinearRgb,
    DisplayP3,
    Aces,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphCache {

    pub glyphs: std::collections::HashMap<String, CachedGlyph>,

    pub max_size: usize,

    pub hits: u64,

    pub misses: u64,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedGlyph {

    pub key: String,

    pub data: Vec<u8>,

    pub metrics: GlyphMetrics,

    pub last_access: u64,

    pub frequency: u32,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphMetrics {
    pub width: f64,
    pub height: f64,
    pub bearing_x: f64,
    pub bearing_y: f64,
    pub advance: f64,
    pub lsb: f64,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub max_batch_size: usize,
    pub parallel_rendering: bool,
    pub render_threads: usize,
    pub instanced_rendering: bool,
    pub texture_atlas_size: (u32, u32),
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphRenderer {
    pub font_family: String,
    pub font_size: f64,
    pub font_weight: u16,
    pub font_style: crate::text::types::FontStyle,
    pub render_mode: GlyphRenderMode,
    pub quality: RenderQuality,
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GlyphRenderMode {
    Fill,
    Stroke,
    FillAndStroke,
    Outline,
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RenderQuality {
    Low,
    Medium,
    High,
    Ultra,
}

impl TextRenderer {
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

    pub fn render_text_layer(&mut self, text_layer: &crate::text::types::TextLayer, time: f64) -> Vec<GlyphRenderInfo> {
        let mut render_infos = Vec::new();

        if !text_layer.visible || text_layer.opacity == 0.0 {
            return render_infos;
        }

        let typography = crate::text::typography::TypographyControls::from_style(&text_layer.style);
        let layout = typography.layout_text(&text_layer.content.text, text_layer.size.0);

        let animated_values = self.get_animated_values(text_layer, time);

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

        if let Some(path_text) = &text_layer.path_text {
            let path_glyphs = self.render_path_text(path_text, text_layer, time);
            render_infos.extend(path_glyphs);
        }

        render_infos
    }


    fn create_render_info(
        &self,
        glyph_pos: &crate::text::typography::GlyphPosition,
        text_layer: &crate::text::types::TextLayer,
        metrics: &crate::text::typography::FontMetrics,
        animated_values: &std::collections::HashMap<crate::text::animation::AnimationType, crate::text::animation::AnimationValue>,
    ) -> GlyphRenderInfo {

        let mut position = (
            text_layer.position.0 + glyph_pos.x,
            text_layer.position.1 + glyph_pos.y,
        );

        if let Some(crate::text::animation::AnimationValue::Vector2(dx, dy)) =
            animated_values.get(&crate::text::animation::AnimationType::Position) {
            position.0 += dx;
            position.1 += dy;
        }

        let mut size = (glyph_pos.width, glyph_pos.height);

        if let Some(crate::text::animation::AnimationValue::Float(scale)) =
            animated_values.get(&crate::text::animation::AnimationType::Scale) {
            size.0 *= scale;
            size.1 *= scale;
        }

        let mut rotation = text_layer.rotation.to_radians();

        if let Some(crate::text::animation::AnimationValue::Float(rot)) =
            animated_values.get(&crate::text::animation::AnimationType::Rotation) {
            rotation += rot.to_radians();
        }

        let mut color = self.parse_color(&text_layer.style.color);

        if let Some(crate::text::animation::AnimationValue::Color(r, g, b, a)) =
            animated_values.get(&crate::text::animation::AnimationType::Color) {
            color = (*r, *g, *b, *a);
        }

        let mut opacity = text_layer.opacity;

        if let Some(crate::text::animation::AnimationValue::Float(op)) =
            animated_values.get(&crate::text::animation::AnimationType::Opacity) {
            opacity *= *op;
        }

        let mut blur = 0.0;

        if let Some(crate::text::animation::AnimationValue::Float(blur_val)) =
            animated_values.get(&crate::text::animation::AnimationType::Blur) {
            blur = *blur_val;
        }

        let baseline_offset = glyph_pos.y - metrics.ascent;

        let mut tracking = 0.0;
        if let Some(crate::text::animation::AnimationValue::Float(track)) =
            animated_values.get(&crate::text::animation::AnimationType::Tracking) {
            tracking = *track;
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

    fn render_path_text(
        &self,
        path_text: &crate::text::path_text::TextOnPath,
        text_layer: &crate::text::types::TextLayer,
        time: f64,
    ) -> Vec<GlyphRenderInfo> {
        let mut render_infos = Vec::new();

        render_infos
    }

    fn get_animated_values(
        &self,
        text_layer: &crate::text::types::TextLayer,
        time: f64,
    ) -> std::collections::HashMap<crate::text::animation::AnimationType, crate::text::animation::AnimationValue> {
        let mut values = std::collections::HashMap::new();

        let mut animator = crate::text::animation::TextAnimator::new();
        for animation in &text_layer.animations {
            animator.add_animation(animation.clone());
        }

        animator.global_time = time;

        let char_values = animator.get_character_values(0);
        values.extend(char_values);

        values
    }

    fn parse_color(&self, color_str: &str) -> (f64, f64, f64, f64) {

        if color_str.starts_with('#') {
            let hex = &color_str[1..];
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
                return (r, g, b, 1.0);
            }
        }

        (0.0, 0.0, 0.0, 1.0)
    }

    pub fn clear_cache(&mut self) {
        self.glyph_cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, u64, u64) {
        (self.glyph_cache.glyphs.len(), self.glyph_cache.hits, self.glyph_cache.misses)
    }

    pub fn optimize_cache(&mut self) {
        self.glyph_cache.optimize();
    }

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
    pub fn new() -> Self {
        Self {
            glyphs: std::collections::HashMap::new(),
            max_size: 1000,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&CachedGlyph> {
        let current_time = self.get_current_time();
        if let Some(glyph) = self.glyphs.get_mut(key) {
            glyph.last_access = current_time;
            glyph.frequency += 1;
            self.hits += 1;
            Some(glyph)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn add(&mut self, key: String, glyph: CachedGlyph) {

        if self.glyphs.len() >= self.max_size {
            self.remove_oldest();
        }
        self.glyphs.insert(key, glyph);
    }

    pub fn remove(&mut self, key: &str) -> Option<CachedGlyph> {
        self.glyphs.remove(key)
    }

    pub fn clear(&mut self) {
        self.glyphs.clear();
        self.hits = 0;
        self.misses = 0;
    }

    pub fn optimize(&mut self) {
        if self.glyphs.len() <= self.max_size {
            return;
        }

        let mut glyphs: Vec<_> = self.glyphs.iter().collect();
        glyphs.sort_by(|a, b| a.1.frequency.cmp(&b.1.frequency));

        let remove_count = self.glyphs.len() - self.max_size;
        let keys_to_remove: Vec<String> = glyphs.iter()
            .take(remove_count)
            .map(|(key, _)| (*key).clone())
            .collect();
        for key in keys_to_remove {
            self.glyphs.remove(&key);
        }
    }

    fn remove_oldest(&mut self) {
        if let Some(oldest_key) = self.glyphs
            .iter()
            .min_by_key(|(_, glyph)| glyph.last_access)
            .map(|(key, _)| key.clone()) {
            self.glyphs.remove(&oldest_key);
        }
    }

    fn get_current_time(&self) -> u64 {

        0
    }

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

    pub fn with_render_mode(mut self, mode: GlyphRenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    pub fn with_quality(mut self, quality: RenderQuality) -> Self {
        self.quality = quality;
        self
    }

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


        assert!(cache.get("test").is_none());
        assert_eq!(cache.misses, 1);


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
        invalid_renderer.font_weight = 50;
        assert!(invalid_renderer.validate().is_err());

        invalid_renderer.font_weight = 1000;
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
