use anyhow::{Result};
use image::{Rgb, RgbImage};

use crate::types::{VectorscopeTarget, VectorscopeConfig};

pub struct TargetRenderer {
    target_colors: std::collections::HashMap<VectorscopeTarget, Rgb<u8>>,
}

impl TargetRenderer {
    pub fn new() -> Self {
        let mut target_colors = std::collections::HashMap::new();

        target_colors.insert(VectorscopeTarget::Primary, Rgb([255, 255, 255]));
        target_colors.insert(VectorscopeTarget::SkinTones, Rgb([255, 200, 150]));
        target_colors.insert(VectorscopeTarget::Blue, Rgb([0, 100, 255]));
        target_colors.insert(VectorscopeTarget::Yellow, Rgb([255, 255, 0]));
        target_colors.insert(VectorscopeTarget::Cyan, Rgb([0, 255, 255]));
        target_colors.insert(VectorscopeTarget::Green, Rgb([0, 255, 0]));
        target_colors.insert(VectorscopeTarget::Magenta, Rgb([255, 0, 255]));
        target_colors.insert(VectorscopeTarget::Red, Rgb([255, 0, 0]));

        Self { target_colors }
    }


    pub fn draw_target_overlays(&self, image: &mut RgbImage, config: &VectorscopeConfig) -> Result<()> {
        let (width, height) = image.dimensions();
        let center_x = width / 2;
        let center_y = height / 2;
        let radius = width.min(height) / 2 - 10;

        for target in &config.targets {
            let color = self.get_target_color(*target);
            let (u, v) = self.get_target_uv(*target);


            let img_x = (center_x as i32 + (u * radius as f32) as i32) as u32;
            let img_y = (center_y as i32 - (v * radius as f32) as i32) as u32;

            self.draw_target_marker(image, img_x as i32, img_y as i32, color)?;
        }

        self.draw_crosshair(image, center_x, center_y, Rgb([128, 128, 128]))?;

        Ok(())
    }

    pub fn get_target_color(&self, target: VectorscopeTarget) -> Rgb<u8> {
        self.target_colors.get(&target).copied().unwrap_or(Rgb([255, 255, 255]))
    }


    pub fn get_target_uv(&self, target: VectorscopeTarget) -> (f32, f32) {
        match target {
            VectorscopeTarget::Primary => (0.0, 0.0),
            VectorscopeTarget::SkinTones => (0.1, 0.2),
            VectorscopeTarget::Blue => (-0.2, -0.4),
            VectorscopeTarget::Yellow => (0.3, 0.4),
            VectorscopeTarget::Cyan => (-0.3, 0.2),
            VectorscopeTarget::Green => (-0.4, -0.2),
            VectorscopeTarget::Magenta => (0.4, -0.2),
            VectorscopeTarget::Red => (0.3, -0.4),
        }
    }


    fn draw_target_marker(&self, image: &mut RgbImage, x: i32, y: i32, color: Rgb<u8>) -> Result<()> {
        let (width, height) = image.dimensions();

        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            let x = x as u32;
            let y = y as u32;


            for dx in -2..=2 {
                if x as i32 + dx >= 0 && x as i32 + dx < width as i32 {
                    let pixel = image.get_pixel_mut((x as i32 + dx) as u32, y as u32);
                        *pixel = color;
                }
            }

            for dy in -2..=2 {
                if y as i32 + dy >= 0 && y as i32 + dy < height as i32 {
                    let pixel = image.get_pixel_mut(x as u32, (y as i32 + dy) as u32);
                        *pixel = color;
                }
            }


            for angle in 0..360 {
                let rad = angle as f32 * std::f32::consts::PI / 180.0;
                let cx = x as f32 + rad.cos() * 5.0;
                let cy = y as f32 + rad.sin() * 5.0;

                if cx >= 0.0 && cx < width as f32 && cy >= 0.0 && cy < height as f32 {
                    let pixel = image.get_pixel_mut(cx as u32, cy as u32);
                        *pixel = color;
                }
            }
        }

        Ok(())
    }


    fn draw_crosshair(&self, image: &mut RgbImage, cx: u32, cy: u32, color: Rgb<u8>) -> Result<()> {
        let (width, height) = image.dimensions();


        for x in 0..width {
            let pixel = image.get_pixel_mut(x, cy);
                *pixel = color;
        }


        for y in 0..height {
            let pixel = image.get_pixel_mut(cx, y);
                *pixel = color;
        }

        Ok(())
    }
}

impl Default for TargetRenderer {
    fn default() -> Self {
        Self::new()
    }
}
