use crate::text::renderer::{GlyphRenderInfo, TextRenderer, CachedGlyph};
use crate::text::types::TextLayer;
use aether_types::ParameterValue;
use uuid::Uuid;

pub struct TextCompositor {
    pub blend_mode: BlendMode,
    pub anti_aliasing: bool,
    pub subpixel_rendering: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
    Subtract,
}

impl TextCompositor {
    pub fn new() -> Self {
        Self {
            blend_mode: BlendMode::Normal,
            anti_aliasing: true,
            subpixel_rendering: false,
        }
    }

    pub fn composite_text_layer(
        &self,
        text_layer: &TextLayer,
        renderer: &mut TextRenderer,
        base_image: Option<(Uuid, Vec<u8>, usize, usize)>,
        time: f64,
    ) -> ParameterValue {
        if !text_layer.visible || text_layer.opacity == 0.0 {
            return base_image.map(|(id, _, _, _)| ParameterValue::Image(id))
                .unwrap_or(ParameterValue::None);
        }

        let render_infos = renderer.render_text_layer(text_layer, time);

        if render_infos.is_empty() {
            return base_image.map(|(id, _, _, _)| ParameterValue::Image(id))
                .unwrap_or(ParameterValue::None);
        }

        if let Some((_base_id, base_data, width, height)) = base_image {
            let _composited = self.composite_glyphs_onto_image(
                &render_infos,
                renderer,
                &base_data,
                width,
                height,
                text_layer.opacity,
            );

            let new_id = Uuid::new_v4();
            ParameterValue::Image(new_id)
        } else {
            let _text_only = self.render_text_only(&render_infos, renderer, text_layer.opacity);
            let new_id = Uuid::new_v4();
            ParameterValue::Image(new_id)
        }
    }

    fn composite_glyphs_onto_image(
        &self,
        render_infos: &[GlyphRenderInfo],
        renderer: &TextRenderer,
        base_data: &[u8],
        width: usize,
        height: usize,
        layer_opacity: f64,
    ) -> Vec<u8> {
        let mut result = base_data.to_vec();

        for render_info in render_infos {
            let glyph_key = self.create_glyph_key(render_info);
            if let Some(cached_glyph) = renderer.glyph_cache.glyphs.get(&glyph_key) {
                self.blend_glyph_onto_image(
                    &mut result,
                    cached_glyph,
                    render_info,
                    width,
                    height,
                    layer_opacity,
                );
            }
        }

        result
    }

    fn blend_glyph_onto_image(
        &self,
        image_data: &mut [u8],
        glyph: &CachedGlyph,
        render_info: &GlyphRenderInfo,
        image_width: usize,
        image_height: usize,
        layer_opacity: f64,
    ) {
        let glyph_x = render_info.position.0 as i32;
        let glyph_y = render_info.position.1 as i32;
        let glyph_width = glyph.metrics.width as usize;
        let glyph_height = glyph.metrics.height as usize;

        let opacity = (render_info.opacity * layer_opacity) as f32;

        for y in 0..glyph_height {
            for x in 0..glyph_width {
                let img_x = (glyph_x + x as i32) as usize;
                let img_y = (glyph_y + y as i32) as usize;

                if img_x >= image_width || img_y >= image_height {
                    continue;
                }

                let pixel_idx = (img_y * image_width + img_x) * 4;
                let glyph_idx = (y * glyph_width + x) * 4;

                if glyph_idx + 3 >= glyph.data.len() || pixel_idx + 3 >= image_data.len() {
                    continue;
                }

                let glyph_r = glyph.data[glyph_idx] as f32;
                let glyph_g = glyph.data[glyph_idx + 1] as f32;
                let glyph_b = glyph.data[glyph_idx + 2] as f32;
                let glyph_a = glyph.data[glyph_idx + 3] as f32 / 255.0 * opacity;

                let img_r = image_data[pixel_idx] as f32;
                let img_g = image_data[pixel_idx + 1] as f32;
                let img_b = image_data[pixel_idx + 2] as f32;
                let img_a = image_data[pixel_idx + 3] as f32 / 255.0;

                let (final_r, final_g, final_b, final_a) = match self.blend_mode {
                    BlendMode::Normal => {
                        let alpha = glyph_a;
                        let inv_alpha = 1.0 - alpha;
                        (
                            glyph_r * alpha + img_r * inv_alpha,
                            glyph_g * alpha + img_g * inv_alpha,
                            glyph_b * alpha + img_b * inv_alpha,
                            glyph_a + img_a * inv_alpha,
                        )
                    }
                    BlendMode::Multiply => {
                        let alpha = glyph_a;
                        let inv_alpha = 1.0 - alpha;
                        let r = (glyph_r / 255.0) * (img_r / 255.0) * 255.0;
                        let g = (glyph_g / 255.0) * (img_g / 255.0) * 255.0;
                        let b = (glyph_b / 255.0) * (img_b / 255.0) * 255.0;
                        (
                            r * alpha + img_r * inv_alpha,
                            g * alpha + img_g * inv_alpha,
                            b * alpha + img_b * inv_alpha,
                            glyph_a + img_a * inv_alpha,
                        )
                    }
                    BlendMode::Screen => {
                        let alpha = glyph_a;
                        let inv_alpha = 1.0 - alpha;
                        let r = (1.0 - (1.0 - glyph_r / 255.0) * (1.0 - img_r / 255.0)) * 255.0;
                        let g = (1.0 - (1.0 - glyph_g / 255.0) * (1.0 - img_g / 255.0)) * 255.0;
                        let b = (1.0 - (1.0 - glyph_b / 255.0) * (1.0 - img_b / 255.0)) * 255.0;
                        (
                            r * alpha + img_r * inv_alpha,
                            g * alpha + img_g * inv_alpha,
                            b * alpha + img_b * inv_alpha,
                            glyph_a + img_a * inv_alpha,
                        )
                    }
                    BlendMode::Overlay => {
                        let alpha = glyph_a;
                        let inv_alpha = 1.0 - alpha;
                        let overlay = |base: f32, blend: f32| -> f32 {
                            if base < 0.5 {
                                2.0 * base * blend
                            } else {
                                1.0 - 2.0 * (1.0 - base) * (1.0 - blend)
                            }
                        };
                        let r = overlay(img_r / 255.0, glyph_r / 255.0) * 255.0;
                        let g = overlay(img_g / 255.0, glyph_g / 255.0) * 255.0;
                        let b = overlay(img_b / 255.0, glyph_b / 255.0) * 255.0;
                        (
                            r * alpha + img_r * inv_alpha,
                            g * alpha + img_g * inv_alpha,
                            b * alpha + img_b * inv_alpha,
                            glyph_a + img_a * inv_alpha,
                        )
                    }
                    BlendMode::Add => {
                        let alpha = glyph_a;
                        let inv_alpha = 1.0 - alpha;
                        let r = (glyph_r + img_r).min(255.0);
                        let g = (glyph_g + img_g).min(255.0);
                        let b = (glyph_b + img_b).min(255.0);
                        (
                            r * alpha + img_r * inv_alpha,
                            g * alpha + img_g * inv_alpha,
                            b * alpha + img_b * inv_alpha,
                            glyph_a + img_a * inv_alpha,
                        )
                    }
                    BlendMode::Subtract => {
                        let alpha = glyph_a;
                        let inv_alpha = 1.0 - alpha;
                        let r = (img_r - glyph_r).max(0.0);
                        let g = (img_g - glyph_g).max(0.0);
                        let b = (img_b - glyph_b).max(0.0);
                        (
                            r * alpha + img_r * inv_alpha,
                            g * alpha + img_g * inv_alpha,
                            b * alpha + img_b * inv_alpha,
                            glyph_a + img_a * inv_alpha,
                        )
                    }
                };

                image_data[pixel_idx] = final_r.min(255.0).max(0.0) as u8;
                image_data[pixel_idx + 1] = final_g.min(255.0).max(0.0) as u8;
                image_data[pixel_idx + 2] = final_b.min(255.0).max(0.0) as u8;
                image_data[pixel_idx + 3] = (final_a.min(1.0).max(0.0) * 255.0) as u8;
            }
        }
    }

    fn render_text_only(
        &self,
        render_infos: &[GlyphRenderInfo],
        renderer: &TextRenderer,
        layer_opacity: f64,
    ) -> Vec<u8> {
        let mut max_x = 0;
        let mut max_y = 0;

        for render_info in render_infos {
            let x = (render_info.position.0 + render_info.size.0) as usize;
            let y = (render_info.position.1 + render_info.size.1) as usize;
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }

        let width = max_x.max(1920);
        let height = max_y.max(1080);
        let mut result = vec![0u8; width * height * 4];

        for render_info in render_infos {
            let glyph_key = self.create_glyph_key(render_info);
            if let Some(cached_glyph) = renderer.glyph_cache.glyphs.get(&glyph_key) {
                self.blend_glyph_onto_image(
                    &mut result,
                    cached_glyph,
                    render_info,
                    width,
                    height,
                    layer_opacity,
                );
            }
        }

        result
    }

    fn create_glyph_key(&self, render_info: &GlyphRenderInfo) -> String {
        format!(
            "char_{}_pos_{:.0}_{:.0}_size_{:.0}_{:.0}",
            render_info.character,
            render_info.position.0,
            render_info.position.1,
            render_info.size.0,
            render_info.size.1
        )
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    pub fn set_anti_aliasing(&mut self, enabled: bool) {
        self.anti_aliasing = enabled;
    }

    pub fn set_subpixel_rendering(&mut self, enabled: bool) {
        self.subpixel_rendering = enabled;
    }
}

impl Default for TextCompositor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_compositor_creation() {
        let compositor = TextCompositor::new();
        assert_eq!(compositor.blend_mode, BlendMode::Normal);
        assert!(compositor.anti_aliasing);
        assert!(!compositor.subpixel_rendering);
    }

    #[test]
    fn test_blend_mode_setting() {
        let mut compositor = TextCompositor::new();
        compositor.set_blend_mode(BlendMode::Multiply);
        assert_eq!(compositor.blend_mode, BlendMode::Multiply);
    }

    #[test]
    fn test_anti_aliasing_setting() {
        let mut compositor = TextCompositor::new();
        compositor.set_anti_aliasing(false);
        assert!(!compositor.anti_aliasing);
    }

    #[test]
    fn test_glyph_key_creation() {
        let compositor = TextCompositor::new();
        let render_info = GlyphRenderInfo {
            character: 'A',
            char_index: 0,
            position: (100.0, 200.0),
            size: (12.0, 24.0),
            rotation: 0.0,
            scale: 1.0,
            color: (1.0, 1.0, 1.0, 1.0),
            opacity: 1.0,
            blur: 0.0,
            baseline_offset: 0.0,
            tracking: 0.0,
        };

        let key = compositor.create_glyph_key(&render_info);
        assert!(key.contains("char_A"));
        assert!(key.contains("pos_100_200"));
    }
}
