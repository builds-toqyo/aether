

use image::{Rgb, RgbImage};
use anyhow::{Result, anyhow};
use log::debug;

use super::{types::LutData, config::LutConfig};


pub struct LutApplicator {
    config: LutConfig,
    interpolation_cache: std::collections::HashMap<(u32, u32, u32), [f32; 3]>,
}

impl LutApplicator {

    pub fn new(config: LutConfig) -> Self {
        Self {
            config: config.clone(),
            interpolation_cache: std::collections::HashMap::new(),
        }
    }


    pub fn apply_lut(&self, image: &mut RgbImage, lut_data: &LutData) -> Result<()> {
        debug!("Applying LUT '{}' to image ({}x{})", lut_data.name, image.width(), image.height());

        let (width, height) = image.dimensions();

        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let [r, g, b] = pixel.0;


                let rf = r as f32 / 255.0;
                let gf = g as f32 / 255.0;
                let bf = b as f32 / 255.0;


                let transformed = match self.config.interpolation_quality {
                    crate::color::luts::types::InterpolationQuality::Nearest => {
                        self.interpolate_nearest(lut_data, [rf, gf, bf])
                    }
                    crate::color::luts::types::InterpolationQuality::Linear => {
                        self.interpolate_linear(lut_data, [rf, gf, bf])
                    }
                    crate::color::luts::types::InterpolationQuality::Trilinear => {
                        self.interpolate_trilinear(lut_data, [rf, gf, bf])
                    }
                };


                let final_values = if self.config.clamp_output {
                    [
                        transformed[0].clamp(0.0, 1.0),
                        transformed[1].clamp(0.0, 1.0),
                        transformed[2].clamp(0.0, 1.0),
                    ]
                } else {
                    transformed
                };


                let new_r = (final_values[0] * 255.0).round().clamp(0.0, 255.0) as u8;
                let new_g = (final_values[1] * 255.0).round().clamp(0.0, 255.0) as u8;
                let new_b = (final_values[2] * 255.0).round().clamp(0.0, 255.0) as u8;

                image.put_pixel(x, y, Rgb([new_r, new_g, new_b]));
            }
        }

        debug!("LUT applied successfully");

        Ok(())
    }


    pub fn apply_lut_to_pixel(&self, pixel: [u8; 3], lut_data: &LutData) -> Result<[u8; 3]> {

        let rf = pixel[0] as f32 / 255.0;
        let gf = pixel[1] as f32 / 255.0;
        let bf = pixel[2] as f32 / 255.0;


        let transformed = match self.config.interpolation_quality {
            crate::color::luts::types::InterpolationQuality::Nearest => {
                self.interpolate_nearest(lut_data, [rf, gf, bf])
            }
            crate::color::luts::types::InterpolationQuality::Linear => {
                self.interpolate_linear(lut_data, [rf, gf, bf])
            }
            crate::color::luts::types::InterpolationQuality::Trilinear => {
                self.interpolate_trilinear(lut_data, [rf, gf, bf])
            }
        };


        let final_values = if self.config.clamp_output {
            [
                transformed[0].clamp(0.0, 1.0),
                transformed[1].clamp(0.0, 1.0),
                transformed[2].clamp(0.0, 1.0),
            ]
        } else {
            transformed
        };


        Ok([
            (final_values[0] * 255.0).round().clamp(0.0, 255.0) as u8,
            (final_values[1] * 255.0).round().clamp(0.0, 255.0) as u8,
            (final_values[2] * 255.0).round().clamp(0.0, 255.0) as u8,
        ])
    }


    fn interpolate_nearest(&self, lut_data: &LutData, input: [f32; 3]) -> [f32; 3] {
        let size = lut_data.size;
        let size_minus_1 = size - 1;


        let r = input[0].clamp(0.0, 1.0);
        let g = input[1].clamp(0.0, 1.0);
        let b = input[2].clamp(0.0, 1.0);


        let r_index = (r * size_minus_1 as f32).round() as u32;
        let g_index = (g * size_minus_1 as f32).round() as u32;
        let b_index = (b * size_minus_1 as f32).round() as u32;


        let r_index = r_index.min(size_minus_1);
        let g_index = g_index.min(size_minus_1);
        let b_index = b_index.min(size_minus_1);

        let idx = self.lut_index(r_index, g_index, b_index, size);

        lut_data.data.get(idx).copied().unwrap_or([0.0, 0.0, 0.0])
    }


    fn interpolate_linear(&self, lut_data: &LutData, input: [f32; 3]) -> [f32; 3] {
        let size = lut_data.size;
        let size_minus_1 = size - 1;


        let r = input[0].clamp(0.0, 1.0);
        let g = input[1].clamp(0.0, 1.0);
        let b = input[2].clamp(0.0, 1.0);


        let r_index = (r * size_minus_1 as f32) as u32;
        let g_index = (g * size_minus_1 as f32) as u32;
        let b_index = (b * size_minus_1 as f32) as u32;


        let r_frac = (r * size_minus_1 as f32) - r_index as f32;
        let g_frac = (g * size_minus_1 as f32) - g_index as f32;
        let b_frac = (b * size_minus_1 as f32) - b_index as f32;


        let idx000 = self.lut_index(r_index, g_index, b_index, size);
        let idx100 = self.lut_index((r_index + 1).min(size_minus_1), g_index, b_index, size);
        let idx010 = self.lut_index(r_index, (g_index + 1).min(size_minus_1), b_index, size);
        let idx001 = self.lut_index(r_index, g_index, (b_index + 1).min(size_minus_1), size);

        let c000 = lut_data.data.get(idx000).unwrap_or(&[0.0, 0.0, 0.0]);
        let c100 = lut_data.data.get(idx100).unwrap_or(&[0.0, 0.0, 0.0]);
        let c010 = lut_data.data.get(idx010).unwrap_or(&[0.0, 0.0, 0.0]);
        let c001 = lut_data.data.get(idx001).unwrap_or(&[0.0, 0.0, 0.0]);


        let c00 = self.lerp(c000, c100, r_frac);
        let c01 = self.lerp(&c001, c001, r_frac);
        let c0 = self.lerp(&c00, &c01, g_frac);
        self.lerp(&c0, &c0, b_frac)
    }


    fn interpolate_trilinear(&self, lut_data: &LutData, input: [f32; 3]) -> [f32; 3] {
        let size = lut_data.size;
        let size_minus_1 = size - 1;


        let r = input[0].clamp(0.0, 1.0);
        let g = input[1].clamp(0.0, 1.0);
        let b = input[2].clamp(0.0, 1.0);


        let r_index = (r * size_minus_1 as f32) as u32;
        let g_index = (g * size_minus_1 as f32) as u32;
        let b_index = (b * size_minus_1 as f32) as u32;


        let r_frac = (r * size_minus_1 as f32) - r_index as f32;
        let g_frac = (g * size_minus_1 as f32) - g_index as f32;
        let b_frac = (b * size_minus_1 as f32) - b_index as f32;


        let idx000 = self.lut_index(r_index, g_index, b_index, size);
        let idx100 = self.lut_index((r_index + 1).min(size_minus_1), g_index, b_index, size);
        let idx010 = self.lut_index(r_index, (g_index + 1).min(size_minus_1), b_index, size);
        let idx110 = self.lut_index((r_index + 1).min(size_minus_1), (g_index + 1).min(size_minus_1), b_index, size);
        let idx001 = self.lut_index(r_index, g_index, (b_index + 1).min(size_minus_1), size);
        let idx101 = self.lut_index((r_index + 1).min(size_minus_1), g_index, (b_index + 1).min(size_minus_1), size);
        let idx011 = self.lut_index(r_index, (g_index + 1).min(size_minus_1), (b_index + 1).min(size_minus_1), size);
        let idx111 = self.lut_index((r_index + 1).min(size_minus_1), (g_index + 1).min(size_minus_1), (b_index + 1).min(size_minus_1), size);

        let c000 = lut_data.data.get(idx000).unwrap_or(&[0.0, 0.0, 0.0]);
        let c100 = lut_data.data.get(idx100).unwrap_or(&[0.0, 0.0, 0.0]);
        let c010 = lut_data.data.get(idx010).unwrap_or(&[0.0, 0.0, 0.0]);
        let c110 = lut_data.data.get(idx110).unwrap_or(&[0.0, 0.0, 0.0]);
        let c001 = lut_data.data.get(idx001).unwrap_or(&[0.0, 0.0, 0.0]);
        let c101 = lut_data.data.get(idx101).unwrap_or(&[0.0, 0.0, 0.0]);
        let c011 = lut_data.data.get(idx011).unwrap_or(&[0.0, 0.0, 0.0]);
        let c111 = lut_data.data.get(idx111).unwrap_or(&[0.0, 0.0, 0.0]);


        let c00 = self.lerp(c000, c100, r_frac);
        let c01 = self.lerp(c001, c101, r_frac);
        let c10 = self.lerp(c010, c110, r_frac);
        let c11 = self.lerp(c011, c111, r_frac);

        let c0 = self.lerp(&c00, &c10, g_frac);
        let c1 = self.lerp(&c01, &c11, g_frac);

        self.lerp(&c0, &c1, b_frac)
    }


    fn lut_index(&self, r: u32, g: u32, b: u32, size: u32) -> usize {
        ((b * size + g) * size + r) as usize
    }


    fn lerp(&self, a: &[f32; 3], b: &[f32; 3], t: f32) -> [f32; 3] {
        [
            a[0] + t * (b[0] - a[0]),
            a[1] + t * (b[1] - a[1]),
            a[2] + t * (b[2] - a[2]),
        ]
    }


    pub fn apply_lut_with_intensity(&self, image: &mut RgbImage, lut_data: &LutData, intensity: f32) -> Result<()> {
        if intensity <= 0.0 {
            return Ok(());
        }
        if intensity >= 1.0 {
            return self.apply_lut(image, lut_data);
        }

        debug!("Applying LUT '{}' with intensity {}", lut_data.name, intensity);

        let (width, height) = image.dimensions();
        let original_image = image.clone();

        for y in 0..height {
            for x in 0..width {
                let original_pixel = original_image.get_pixel(x, y);
                let original_rgb = [original_pixel.0[0] as f32, original_pixel.0[1] as f32, original_pixel.0[2] as f32];


                let lut_pixel = self.apply_lut_to_pixel(original_pixel.0, lut_data)?;
                let lut_rgb = [lut_pixel[0] as f32, lut_pixel[1] as f32, lut_pixel[2] as f32];


                let blended = [
                    original_rgb[0] * (1.0 - intensity) + lut_rgb[0] * intensity,
                    original_rgb[1] * (1.0 - intensity) + lut_rgb[1] * intensity,
                    original_rgb[2] * (1.0 - intensity) + lut_rgb[2] * intensity,
                ];

                image.put_pixel(x, y, Rgb([
                    blended[0].round().clamp(0.0, 255.0) as u8,
                    blended[1].round().clamp(0.0, 255.0) as u8,
                    blended[2].round().clamp(0.0, 255.0) as u8,
                ]));
            }
        }

        Ok(())
    }


    pub fn apply_lut_to_region(&self, image: &mut RgbImage, lut_data: &LutData, region: &Region) -> Result<()> {
        debug!("Applying LUT '{}' to region {:?}", lut_data.name, region);

        let (width, height) = image.dimensions();

        let start_x = region.x.min(width);
        let start_y = region.y.min(height);
        let end_x = (region.x + region.width).min(width);
        let end_y = (region.y + region.height).min(height);

        for y in start_y..end_y {
            for x in start_x..end_x {
                let pixel = image.get_pixel(x, y);
                let transformed = self.apply_lut_to_pixel(pixel.0, lut_data)?;
                image.put_pixel(x, y, Rgb(transformed));
            }
        }

        Ok(())
    }


    pub fn preview_lut(&self, image: &mut RgbImage, lut_data: &LutData, sample_rate: u32) -> Result<()> {
        debug!("Previewing LUT '{}' with sample rate {}", lut_data.name, sample_rate);

        let (width, height) = image.dimensions();

        for y in (0..height).step_by(sample_rate as usize) {
            for x in (0..width).step_by(sample_rate as usize) {
                let pixel = image.get_pixel(x, y);
                let transformed = self.apply_lut_to_pixel(pixel.0, lut_data)?;
                image.put_pixel(x, y, Rgb(transformed));
            }
        }

        Ok(())
    }


    pub fn get_performance_metrics(&self, lut_data: &LutData) -> LutPerformanceMetrics {
        let size = lut_data.size as f32;
        let memory_usage = lut_data.data.len() * 3 * std::mem::size_of::<f32>();

        LutPerformanceMetrics {
            lut_size: lut_data.size,
            data_points: lut_data.data.len(),
            memory_usage_bytes: memory_usage,
            interpolation_quality: self.config.interpolation_quality,
            estimated_cost_per_pixel: match self.config.interpolation_quality {
                crate::color::luts::types::InterpolationQuality::Nearest => 1.0,
                crate::color::luts::types::InterpolationQuality::Linear => 8.0,
                crate::color::luts::types::InterpolationQuality::Trilinear => 27.0,
            },
        }
    }


    pub fn update_config(&mut self, config: LutConfig) {
        self.config = config;

        self.interpolation_cache.clear();
    }


    pub fn clear_cache(&mut self) {
        self.interpolation_cache.clear();
    }


    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            entries: self.interpolation_cache.len(),
            memory_usage_bytes: self.interpolation_cache.len() * std::mem::size_of::<((u32, u32, u32), [f32; 3])>(),
        }
    }
}

impl Default for LutApplicator {
    fn default() -> Self {
        Self::new(LutConfig::default())
    }
}


#[derive(Debug, Clone)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {

    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }


    pub fn entire(image_width: u32, image_height: u32) -> Self {
        Self::new(0, 0, image_width, image_height)
    }


    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }


    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}


#[derive(Debug, Clone)]
pub struct LutPerformanceMetrics {
    pub lut_size: u32,
    pub data_points: usize,
    pub memory_usage_bytes: usize,
    pub interpolation_quality: crate::color::luts::types::InterpolationQuality,
    pub estimated_cost_per_pixel: f32,
}

impl LutPerformanceMetrics {

    pub fn memory_usage_string(&self) -> String {
        let bytes = self.memory_usage_bytes;

        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f32 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f32 / (1024.0 * 1024.0))
        }
    }


    pub fn estimated_processing_time(&self, image_pixels: u32) -> f32 {

        let base_time = 0.1;
        base_time * self.estimated_cost_per_pixel * image_pixels as f32
    }
}


#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub memory_usage_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lut_applicator_creation() {
        let config = LutConfig::default();
        let applicator = LutApplicator::new(config);

        assert!(matches!(applicator.config.interpolation_quality, crate::color::luts::types::InterpolationQuality::Trilinear));
    }

    #[test]
    fn test_nearest_interpolation() {
        let mut lut_data = LutData::new("test".to_string(), 3, LutFormat::Cube);
        lut_data.generate_identity_lut(3);

        let applicator = LutApplicator::new(LutConfig::new().with_interpolation_quality(crate::color::luts::types::InterpolationQuality::Nearest));

        let input = [0.5, 0.5, 0.5];
        let output = applicator.interpolate_nearest(&lut_data, input);


        assert!((output[0] - input[0]).abs() < 0.1);
        assert!((output[1] - input[1]).abs() < 0.1);
        assert!((output[2] - input[2]).abs() < 0.1);
    }

    #[test]
    fn test_pixel_application() {
        let mut lut_data = LutData::new("test".to_string(), 3, LutFormat::Cube);
        lut_data.generate_identity_lut(3);

        let applicator = LutApplicator::default();

        let input_pixel = [128, 128, 128];
        let output_pixel = applicator.apply_lut_to_pixel(input_pixel, &lut_data).unwrap();


        for i in 0..3 {
            let diff = (output_pixel[i] as f32 - input_pixel[i] as f32).abs();
            assert!(diff <= 2.0, "Channel {} changed too much: {}", i, diff);
        }
    }

    #[test]
    fn test_intensity_blending() {
        let mut lut_data = LutData::new("test".to_string(), 3, LutFormat::Cube);
        lut_data.generate_identity_lut(3);

        let applicator = LutApplicator::default();


        let mut image = RgbImage::new(10, 10);
        image.put_pixel(0, 0, Rgb([100, 100, 100]));

        let original_pixel = image.get_pixel(0, 0);


        applicator.apply_lut_with_intensity(&mut image, &lut_data, 0.5).unwrap();

        let blended_pixel = image.get_pixel(0, 0);


        assert_eq!(original_pixel.0, blended_pixel.0);
    }

    #[test]
    fn test_region_application() {
        let mut lut_data = LutData::new("test".to_string(), 3, LutFormat::Cube);
        lut_data.generate_identity_lut(3);

        let applicator = LutApplicator::default();


        let mut image = RgbImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                image.put_pixel(x, y, Rgb([x as u8 * 25, y as u8 * 25, 128]));
            }
        }

        let original_pixel = image.get_pixel(5, 5);


        let region = Region::new(0, 0, 3, 3);
        applicator.apply_lut_to_region(&mut image, &lut_data, &region).unwrap();

        let unchanged_pixel = image.get_pixel(5, 5);
        assert_eq!(original_pixel.0, unchanged_pixel.0);
    }

    #[test]
    fn test_performance_metrics() {
        let mut lut_data = LutData::new("test".to_string(), 33, LutFormat::Cube);
        lut_data.generate_identity_lut(33);

        let applicator = LutApplicator::default();
        let metrics = applicator.get_performance_metrics(&lut_data);

        assert_eq!(metrics.lut_size, 33);
        assert_eq!(metrics.data_points, 33 * 33 * 33);
        assert!(metrics.memory_usage_bytes > 0);
        assert!(metrics.estimated_cost_per_pixel > 0.0);

        let time_1mp = metrics.estimated_processing_time(1_000_000);
        assert!(time_1mp > 0.0);
    }

    #[test]
    fn test_region() {
        let region = Region::new(10, 20, 100, 200);

        assert!(region.is_valid());
        assert_eq!(region.area(), 20000);

        let entire = Region::entire(1920, 1080);
        assert_eq!(entire.x, 0);
        assert_eq!(entire.y, 0);
        assert_eq!(entire.width, 1920);
        assert_eq!(entire.height, 1080);
    }
}
