

use anyhow::{Result, anyhow};
use log::{debug, info};
use image::{Rgb, RgbImage};

use crate::types::{ColorSpace, VideoRange};
use super::{display::HdrDisplayManager, tone::ToneMapper, gamut::GamutMapper, config::HdrConfig, types::HdrImage};


pub struct HdrProcessor {
    config: HdrConfig,
    display_manager: HdrDisplayManager,
    tone_mapper: ToneMapper,
    gamut_mapper: GamutMapper,
}

impl HdrProcessor {

    pub fn new(config: HdrConfig) -> Result<Self> {
        info!("Creating HDR processor with config: {:?}", config);

        let processor = Self {
            config: config.clone(),
            display_manager: HdrDisplayManager::new(config.display_config.clone())?,
            tone_mapper: ToneMapper::new(config.tone_mapping_config.clone()),
            gamut_mapper: GamutMapper::new(config.gamut_mapping_config.clone()),
        };

        info!("HDR processor created successfully");

        Ok(processor)
    }


    pub fn process_hdr_image(
        &self,
        hdr_image: &HdrImage,
        target_display: super::types::HdrDisplayType,
        output_color_space: ColorSpace,
    ) -> Result<RgbImage> {
        debug!("Processing HDR image for display: {:?}", target_display);


        let display_info = self.display_manager.get_display_info(target_display)?;


        let tone_mapped = self.tone_mapper.apply_tone_mapping(
            hdr_image,
            &display_info,
            &self.config.tone_mapping_config,
        )?;


        let gamut_mapped = self.gamut_mapper.apply_gamut_mapping(
            &tone_mapped,
            output_color_space,
            &self.config.gamut_mapping_config,
        )?;


        let output_image = self.hdr_to_sdr(&gamut_mapped, output_color_space)?;

        debug!("HDR image processed successfully");

        Ok(output_image)
    }


    pub fn sdr_to_hdr(&self, sdr_image: &RgbImage, target_nits: f32) -> Result<HdrImage> {
        debug!("Converting SDR to HDR with {} nits", target_nits);

        let (width, height) = sdr_image.dimensions();
        let mut hdr_data = vec![super::types::HdrPixel::default(); (width * height) as usize];

        for y in 0..height {
            for x in 0..width {
                let pixel = sdr_image.get_pixel(x, y);
                let [r, g, b] = pixel.0;


                let hdr_pixel = super::types::HdrPixel {
                    r: (r as f32 / 255.0) * target_nits,
                    g: (g as f32 / 255.0) * target_nits,
                    b: (b as f32 / 255.0) * target_nits,
                };

                hdr_data[(y * width + x) as usize] = hdr_pixel;
            }
        }

        let hdr_image = HdrImage {
            width,
            height,
            data: hdr_data,
            color_primaries: super::types::ColorPrimaries::Rec709,
            transfer_function: super::types::TransferFunction::Pq,
            max_nits: target_nits,
        };

        debug!("SDR to HDR conversion completed");

        Ok(hdr_image)
    }


    fn hdr_to_sdr(&self, hdr_image: &HdrImage, output_color_space: ColorSpace) -> Result<RgbImage> {
        let (width, height) = (hdr_image.width, hdr_image.height);
        let mut sdr_image = RgbImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let hdr_pixel = &hdr_image.data[(y * width + x) as usize];


                let sdr_r = (hdr_pixel.r / 1000.0 * 255.0).clamp(0.0, 255.0) as u8;
                let sdr_g = (hdr_pixel.g / 1000.0 * 255.0).clamp(0.0, 255.0) as u8;
                let sdr_b = (hdr_pixel.b / 1000.0 * 255.0).clamp(0.0, 255.0) as u8;

                sdr_image.put_pixel(x, y, Rgb([sdr_r, sdr_g, sdr_b]));
            }
        }

        Ok(sdr_image)
    }


    pub fn analyze_hdr_content(&self, hdr_image: &HdrImage) -> Result<super::analysis::HdrAnalysis> {
        debug!("Analyzing HDR content");

        let mut max_nits = 0.0;
        let mut avg_nits = 0.0;
        let mut pixel_count = 0u64;

        for pixel in &hdr_image.data {
            let luminance = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
            max_nits = max_nits.max(luminance);
            avg_nits += luminance;
            pixel_count += 1;
        }

        avg_nits /= pixel_count as f32;

        let analysis = super::analysis::HdrAnalysis {
            max_nits,
            avg_nits,
            dynamic_range: max_nits / avg_nits.max(0.1),
            peak_percentage: (max_nits / 10000.0 * 100.0).min(100.0),
            content_type: super::analysis::classify_content_type(max_nits, avg_nits),
        };

        debug!("HDR analysis: {:?}", analysis);

        Ok(analysis)
    }


    pub fn update_config(&mut self, config: HdrConfig) -> Result<()> {
        debug!("Updating HDR configuration");

        self.config = config.clone();
        self.display_manager.update_config(config.display_config)?;
        self.tone_mapper.update_config(config.tone_mapping_config);
        self.gamut_mapper.update_config(config.gamut_mapping_config);

        info!("HDR configuration updated");

        Ok(())
    }


    pub fn get_available_displays(&self) -> Vec<super::types::HdrDisplayType> {
        self.display_manager.get_available_displays()
    }


    pub fn get_available_tone_mappers(&self) -> Vec<super::tone::ToneMappingAlgorithm> {
        self.tone_mapper.get_available_algorithms()
    }


    pub fn get_available_gamut_mappers(&self) -> Vec<super::gamut::GamutMappingAlgorithm> {
        self.gamut_mapper.get_available_algorithms()
    }
}

impl Default for HdrProcessor {
    fn default() -> Self {
        Self::new(HdrConfig::default()).unwrap()
    }
}
