use anyhow::Result;
use log::{debug, info, warn};
use image::{Rgb, RgbImage};

use crate::types::{ColorSpace, VideoRange};
use super::{transforms::TransformManager, looks::LookManager, config::AcesConfig};

pub struct AcesProcessor {
    config: AcesConfig,
    color_space: ColorSpace,
    video_range: VideoRange,
    ocio_pipeline: Option<super::ocio::OcioPipeline>,
    transform_manager: TransformManager,
    look_manager: LookManager,
}

impl AcesProcessor {
    pub fn new(config: AcesConfig) -> Result<Self> {
        info!("Creating ACES processor with config: {:?}", config);

        let mut processor = Self {
            config: config.clone(),
            color_space: ColorSpace::Rec709,
            video_range: VideoRange::Limited,
            ocio_pipeline: None,
            transform_manager: TransformManager::new()?,
            look_manager: LookManager::new(),
        };

        if config.use_opencolorio {
            processor.initialize_opencolorio()?;
        }

        processor.transform_manager.initialize_transforms()?;
        processor.look_manager.initialize_default_looks();

        info!("ACES processor created successfully");

        Ok(processor)
    }

    fn initialize_opencolorio(&mut self) -> Result<()> {
        debug!("Initializing OpenColorIO via runtime FFI");

        match super::ocio::OcioPipeline::new(&self.config.ocio_config_path) {
            Some(pipeline) => {
                self.ocio_pipeline = Some(pipeline);
                info!("OpenColorIO pipeline initialized from: {}", self.config.ocio_config_path);
            }
            None => {
                warn!(
                    "OpenColorIO library or config not available ({}). Using fallback matrices.",
                    self.config.ocio_config_path
                );
            }
        }

        Ok(())
    }

    pub fn to_aces(&self, image: &RgbImage, input_transform: super::InputTransform) -> Result<RgbImage> {
        debug!("Converting image to ACES with transform: {:?}", input_transform);

        let (width, height) = image.dimensions();
        let mut aces_image = RgbImage::new(width, height);

        // Try OpenColorIO real-time FFI pipeline first
        if let Some(pipeline) = &self.ocio_pipeline {
            let src_space = self.ocio_input_space_name(input_transform);
            for y in 0..height {
                for x in 0..width {
                    let pixel = image.get_pixel(x, y);
                    let mut rgb = [
                        pixel.0[0] as f32 / 255.0,
                        pixel.0[1] as f32 / 255.0,
                        pixel.0[2] as f32 / 255.0,
                    ];
                    if pipeline.transform_pixel(&src_space, "ACES2065-1", &mut rgb).is_ok() {
                        aces_image.put_pixel(x, y, Rgb([
                            (rgb[0] * 255.0).clamp(0.0, 255.0) as u8,
                            (rgb[1] * 255.0).clamp(0.0, 255.0) as u8,
                            (rgb[2] * 255.0).clamp(0.0, 255.0) as u8,
                        ]));
                    } else {
                        // Fallback for this pixel if OCIO color space not found
                        let aces_rgb = self.transform_manager.apply_input_transform(pixel.0, input_transform)?;
                        aces_image.put_pixel(x, y, Rgb(aces_rgb));
                    }
                }
            }
            debug!("Image converted to ACES via OpenColorIO");
            return Ok(aces_image);
        }

        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let [r, g, b] = pixel.0;

                let _linear_rgb = self.gamma_decode([r, g, b], self.color_space);

                let aces_rgb = self.transform_manager.apply_input_transform([r, g, b], input_transform)?;

                aces_image.put_pixel(x, y, Rgb(aces_rgb));
            }
        }

        debug!("Image converted to ACES successfully");

        Ok(aces_image)
    }

    pub fn from_aces(&self, aces_image: &RgbImage, output_transform: super::OutputTransform) -> Result<RgbImage> {
        debug!("Converting ACES image with transform: {:?}", output_transform);

        let (width, height) = aces_image.dimensions();
        let mut output_image = RgbImage::new(width, height);

        // Try OpenColorIO real-time FFI pipeline first
        if let Some(pipeline) = &self.ocio_pipeline {
            let dst_space = self.ocio_output_space_name(output_transform);
            for y in 0..height {
                for x in 0..width {
                    let pixel = aces_image.get_pixel(x, y);
                    let mut rgb = [
                        pixel.0[0] as f32 / 255.0,
                        pixel.0[1] as f32 / 255.0,
                        pixel.0[2] as f32 / 255.0,
                    ];
                    if pipeline.transform_pixel("ACES2065-1", &dst_space, &mut rgb).is_ok() {
                        output_image.put_pixel(x, y, Rgb([
                            (rgb[0] * 255.0).clamp(0.0, 255.0) as u8,
                            (rgb[1] * 255.0).clamp(0.0, 255.0) as u8,
                            (rgb[2] * 255.0).clamp(0.0, 255.0) as u8,
                        ]));
                    } else {
                        let output_rgb = self.transform_manager.apply_output_transform(pixel.0, output_transform)?;
                        let gamma_rgb = self.gamma_encode(output_rgb, self.color_space);
                        output_image.put_pixel(x, y, Rgb([
                            gamma_rgb[0].clamp(0.0, 255.0) as u8,
                            gamma_rgb[1].clamp(0.0, 255.0) as u8,
                            gamma_rgb[2].clamp(0.0, 255.0) as u8,
                        ]));
                    }
                }
            }
            debug!("ACES image converted via OpenColorIO");
            return Ok(output_image);
        }

        for y in 0..height {
            for x in 0..width {
                let pixel = aces_image.get_pixel(x, y);
                let [r, g, b] = pixel.0;

                let output_rgb = self.transform_manager.apply_output_transform([r, g, b], output_transform)?;

                let gamma_rgb = self.gamma_encode(output_rgb, self.color_space);

                let clamped_rgb = [
                    gamma_rgb[0].clamp(0.0, 255.0) as u8,
                    gamma_rgb[1].clamp(0.0, 255.0) as u8,
                    gamma_rgb[2].clamp(0.0, 255.0) as u8,
                ];

                output_image.put_pixel(x, y, Rgb(clamped_rgb));
            }
        }

        debug!("ACES image converted successfully");

        Ok(output_image)
    }

    pub fn apply_look(&self, aces_image: &mut RgbImage, look_name: &str) -> Result<()> {
        debug!("Applying look: {}", look_name);

        let look = self.look_manager.get_look(look_name)?;

        let (width, height) = aces_image.dimensions();

        for y in 0..height {
            for x in 0..width {
                let pixel = aces_image.get_pixel(x, y);
                let [r, g, b] = pixel.0;


                let modified_rgb = look.apply([r, g, b]);

                aces_image.put_pixel(x, y, Rgb(modified_rgb));
            }
        }

        debug!("Look applied successfully");

        Ok(())
    }

    pub fn process_pipeline(
        &self,
        input_image: &RgbImage,
        input_transform: super::InputTransform,
        look_name: Option<&str>,
        output_transform: super::OutputTransform,
    ) -> Result<RgbImage> {
        debug!("Processing complete ACES pipeline");

        let mut aces_image = self.to_aces(input_image, input_transform)?;

        if let Some(look) = look_name {
            self.apply_look(&mut aces_image, look)?;
        }

        let output_image = self.from_aces(&aces_image, output_transform)?;

        debug!("ACES pipeline completed successfully");

        Ok(output_image)
    }

    fn gamma_decode(&self, rgb: [u8; 3], color_space: ColorSpace) -> [f32; 3] {
        match color_space {
            ColorSpace::Rec709 => {
                [
                    super::gamma::rec709_gamma_decode(rgb[0] as f32 / 255.0),
                    super::gamma::rec709_gamma_decode(rgb[1] as f32 / 255.0),
                    super::gamma::rec709_gamma_decode(rgb[2] as f32 / 255.0),
                ]
            }
            ColorSpace::Rec2020 => {
                [
                    super::gamma::rec2020_gamma_decode(rgb[0] as f32 / 255.0),
                    super::gamma::rec2020_gamma_decode(rgb[1] as f32 / 255.0),
                    super::gamma::rec2020_gamma_decode(rgb[2] as f32 / 255.0),
                ]
            }
            ColorSpace::SRGB => {
                [
                    super::gamma::srgb_gamma_decode(rgb[0] as f32 / 255.0),
                    super::gamma::srgb_gamma_decode(rgb[1] as f32 / 255.0),
                    super::gamma::srgb_gamma_decode(rgb[2] as f32 / 255.0),
                ]
            }
            _ => [rgb[0] as f32 / 255.0, rgb[1] as f32 / 255.0, rgb[2] as f32 / 255.0],
        }
    }

    fn gamma_encode(&self, rgb: [u8; 3], color_space: ColorSpace) -> [f32; 3] {
        match color_space {
            ColorSpace::Rec709 => {
                [
                    super::gamma::rec709_gamma_encode(rgb[0] as f32 / 255.0),
                    super::gamma::rec709_gamma_encode(rgb[1] as f32 / 255.0),
                    super::gamma::rec709_gamma_encode(rgb[2] as f32 / 255.0),
                ]
            }
            ColorSpace::Rec2020 => {
                [
                    super::gamma::rec2020_gamma_encode(rgb[0] as f32 / 255.0),
                    super::gamma::rec2020_gamma_encode(rgb[1] as f32 / 255.0),
                    super::gamma::rec2020_gamma_encode(rgb[2] as f32 / 255.0),
                ]
            }
            ColorSpace::SRGB => {
                [
                    super::gamma::srgb_gamma_encode(rgb[0] as f32 / 255.0),
                    super::gamma::srgb_gamma_encode(rgb[1] as f32 / 255.0),
                    super::gamma::srgb_gamma_encode(rgb[2] as f32 / 255.0),
                ]
            }
            _ => [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32],
        }
    }

    pub fn update_color_space(&mut self, color_space: ColorSpace) -> Result<()> {
        debug!("Updating ACES processor color space: {:?}", color_space);
        self.color_space = color_space;
        Ok(())
    }

    pub fn update_video_range(&mut self, video_range: VideoRange) -> Result<()> {
        debug!("Updating ACES processor video range: {:?}", video_range);
        self.video_range = video_range;
        Ok(())
    }

    pub fn get_available_input_transforms(&self) -> Vec<super::InputTransform> {
        self.transform_manager.get_available_input_transforms()
    }

    pub fn get_available_output_transforms(&self) -> Vec<super::OutputTransform> {
        self.transform_manager.get_available_output_transforms()
    }

    pub fn get_available_looks(&self) -> Vec<String> {
        self.look_manager.get_available_looks()
    }

    fn ocio_input_space_name(&self, transform: super::InputTransform) -> String {
        match transform {
            super::InputTransform::Rec709ToAces => "Input - Generic - sRGB - Texture".to_string(),
            super::InputTransform::Rec2020ToAces => "Input - Generic - Rec.2020".to_string(),
            super::InputTransform::SrgbToAces => "Input - Generic - sRGB - Texture".to_string(),
            super::InputTransform::RawToAces => "Input - Camera - Raw".to_string(),
        }
    }

    fn ocio_output_space_name(&self, transform: super::OutputTransform) -> String {
        match transform {
            super::OutputTransform::AcesToRec709 => "Output - Rec.709".to_string(),
            super::OutputTransform::AcesToRec2020 => "Output - Rec.2020".to_string(),
            super::OutputTransform::AcesToSrgb => "Output - sRGB".to_string(),
            super::OutputTransform::AcesToHdr10 => "Output - Rec.2020 - PQ".to_string(),
        }
    }
}

impl Default for AcesProcessor {
    fn default() -> Self {
        Self::new(AcesConfig::default()).unwrap()
    }
}
