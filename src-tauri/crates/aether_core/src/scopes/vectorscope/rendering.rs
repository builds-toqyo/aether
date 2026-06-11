use anyhow::{Result};
use image::{Rgb, RgbImage};

use crate::types::{VectorscopeData, VectorscopeConfig};

pub struct VectorscopeRenderer {
    intensity_colors: Vec<Rgb<u8>>,
}

impl VectorscopeRenderer {

    pub fn new() -> Self {
        let mut intensity_colors = Vec::new();

        for i in 0..1000 {
            intensity_colors.push(Self::intensity_to_color(i));
        }

        Self { intensity_colors }
    }

    pub fn generate_image(
        &self,
        width: u32,
        height: u32,
        data: &VectorscopeData,
        config: &VectorscopeConfig
    ) -> Result<RgbImage> {
        let mut image = RgbImage::new(width, height);


        for pixel in image.pixels_mut() {
            *pixel = Rgb([16, 16, 16]);
        }


        self.draw_intensity_grid(&mut image, &data, config)?;

        Ok(image)
    }


    fn draw_intensity_grid(
        &self,
        image: &mut RgbImage,
        data: &VectorscopeData,
        config: &VectorscopeConfig
    ) -> Result<()> {
        let grid_size = config.resolution.vectorscope_grid_size();

        for y in 0..grid_size {
            for x in 0..grid_size {
                let index = y * grid_size + x;
                let intensity = data.intensity_grid[index];

                if intensity > 0 {

                    let img_x = (x * image.width() as usize / grid_size) as u32;
                    let img_y = (y * image.height() as usize / grid_size) as u32;


                    let color = self.get_intensity_color(intensity);

                    if img_x < image.width() && img_y < image.height() {
                        let pixel = image.get_pixel_mut(img_x, img_y);
                        *pixel = color;
                    }
                }
            }
        }

        Ok(())
    }


    fn get_intensity_color(&self, intensity: u16) -> Rgb<u8> {
        if intensity < self.intensity_colors.len() as u16 {
            self.intensity_colors[intensity as usize]
        } else {

            self.intensity_colors.last().copied().unwrap_or(Rgb([255, 255, 255]))
        }
    }


    fn intensity_to_color(intensity: u16) -> Rgb<u8> {
        let normalized = (intensity as f32 / 1000.0).min(1.0);


        let r = (normalized * 255.0) as u8;
        let g = ((1.0 - (normalized - 0.5).abs() * 2.0) * 255.0) as u8;
        let b = ((1.0 - normalized) * 255.0) as u8;

        Rgb([r, g, b])
    }
}

impl Default for VectorscopeRenderer {
    fn default() -> Self {
        Self::new()
    }
}
