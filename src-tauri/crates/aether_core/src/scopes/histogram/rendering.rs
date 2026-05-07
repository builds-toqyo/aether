

use anyhow::{Result};
use log::debug;
use image::{Rgb, RgbImage};

use crate::types::{HistogramData, HistogramChannel, HistogramConfig, HistogramMode};


pub struct HistogramRenderer {
    grid_color: Rgb<u8>,
}

impl HistogramRenderer {

    pub fn new() -> Self {
        Self {
            grid_color: Rgb([8, 8, 8]),
        }
    }


    pub fn generate_image(
        &self,
        width: u32,
        height: u32,
        data: &HistogramData,
        config: &HistogramConfig
    ) -> Result<RgbImage> {
        let mut image = RgbImage::new(width, height);


        for pixel in image.pixels_mut() {
            *pixel = Rgb([16, 16, 16]);
        }


        match config.mode {
            HistogramMode::RGB => self.draw_rgb_histogram(&mut image, data)?,
            HistogramMode::Luma => self.draw_luma_histogram(&mut image, data)?,
            HistogramMode::Individual => self.draw_individual_histograms(&mut image, data)?,
            HistogramMode::Parade => self.draw_parade_histograms(&mut image, data)?,
        }


        self.draw_grid(&mut image)?;

        Ok(image)
    }


    fn draw_rgb_histogram(&self, image: &mut RgbImage, data: &HistogramData) -> Result<()> {
        let (width, height) = image.dimensions();


        let red_norm = data.normalize(HistogramChannel::Red);
        let green_norm = data.normalize(HistogramChannel::Green);
        let blue_norm = data.normalize(HistogramChannel::Blue);


        for x in 0..width.min(256) {
            let bin = x as usize;


            let red_height = (red_norm[bin] as u32 * height / 256) as u32;
            let green_height = (green_norm[bin] as u32 * height / 256) as u32;
            let blue_height = (blue_norm[bin] as u32 * height / 256) as u32;


            for y in (height - red_height)..height {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    let [r, g, b] = pixel.0;
                    *pixel = Rgb([255.min(r + 128), g, b]);
                }
            }


            for y in (height - green_height)..height {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    let [r, g, b] = pixel.0;
                    *pixel = Rgb([r, 255.min(g + 128), b]);
                }
            }


            for y in (height - blue_height)..height {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    let [r, g, b] = pixel.0;
                    *pixel = Rgb([r, g, 255.min(b + 128)]);
                }
            }
        }

        Ok(())
    }


    fn draw_luma_histogram(&self, image: &mut RgbImage, data: &HistogramData) -> Result<()> {
        let (width, height) = image.dimensions();

        let luma_norm = data.normalize(HistogramChannel::Luma);

        for x in 0..width.min(256) {
            let bin = x as usize;
            let luma_height = (luma_norm[bin] as u32 * height / 256) as u32;

            for y in (height - luma_height)..height {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    *pixel = Rgb([200, 200, 200]);
                }
            }
        }

        Ok(())
    }


    fn draw_individual_histograms(&self, image: &mut RgbImage, data: &HistogramData) -> Result<()> {
        let (width, height) = image.dimensions();
        let channel_width = width / 3;

        let channels = [
            (HistogramChannel::Red, Rgb([255, 0, 0]), 0),
            (HistogramChannel::Green, Rgb([0, 255, 0]), 1),
            (HistogramChannel::Blue, Rgb([0, 0, 255]), 2),
        ];

        for (channel, color, index) in channels {
            let x_offset = (index as u32 * channel_width) as u32;
            let channel_data = data.normalize(channel);

            for x in 0..channel_width.min(256) {
                let bin = x as usize;
                let bin_height = (channel_data[bin] as u32 * height / 256) as u32;

                for y in (height - bin_height)..height {
                    let img_x = x_offset + x;
                    if img_x < width && let Some(pixel) = image.get_pixel_mut(img_x, y) {
                        *pixel = color;
                    }
                }
            }
        }

        Ok(())
    }


    fn draw_parade_histograms(&self, image: &mut RgbImage, data: &HistogramData) -> Result<()> {
        let (width, height) = image.dimensions();
        let channel_height = height / 3;

        let channels = [
            (HistogramChannel::Red, Rgb([255, 0, 0]), 0),
            (HistogramChannel::Green, Rgb([0, 255, 0]), 1),
            (HistogramChannel::Blue, Rgb([0, 0, 255]), 2),
        ];

        for (channel, color, index) in channels {
            let y_offset = (index as u32 * channel_height) as u32;
            let channel_data = data.normalize(channel);

            for x in 0..width.min(256) {
                let bin = x as usize;
                let bin_height = (channel_data[bin] as u32 * channel_height / 256) as u32;

                for y in 0..bin_height {
                    let img_y = y_offset + (channel_height - y);
                    if img_y < height && let Some(pixel) = image.get_pixel_mut(x, img_y) {
                        *pixel = color;
                    }
                }
            }
        }

        Ok(())
    }


    fn draw_grid(&self, image: &mut RgbImage) -> Result<()> {
        let (width, height) = image.dimensions();


        for x in (0..width.min(256)).step_by(32) {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    let [r, g, b] = pixel.0;
                    *pixel = Rgb([r/2, g/2, b/2]);
                }
            }
        }


        for y_percent in [25, 50, 75] {
            let y = height * (100 - y_percent) / 100;
            for x in 0..width {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    let [r, g, b] = pixel.0;
                    *pixel = Rgb([r/2, g/2, b/2]);
                }
            }
        }

        Ok(())
    }
}

impl Default for HistogramRenderer {
    fn default() -> Self {
        Self::new()
    }
}
