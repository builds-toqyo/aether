

use anyhow::{Result, anyhow};
use log::debug;

use super::types::HdrImage;
use super::config::GamutMappingConfig;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamutMappingAlgorithm {
    Itp,
    Yuv,
    Saturation,
    Perceptual,
}


pub struct GamutMapper {
    config: GamutMappingConfig,
}

impl GamutMapper {

    pub fn new(config: GamutMappingConfig) -> Self {
        Self { config }
    }


    pub fn apply_gamut_mapping(
        &self,
        hdr_image: &HdrImage,
        target_color_space: crate::types::ColorSpace,
        config: &GamutMappingConfig,
    ) -> Result<HdrImage> {
        debug!("Applying gamut mapping with algorithm: {:?}", config.algorithm);

        let mut gamut_mapped = hdr_image.clone();

        for pixel in &mut gamut_mapped.data {
            *pixel = match config.algorithm {
                GamutMappingAlgorithm::Itp => self.itp_gamut_map(*pixel, config),
                GamutMappingAlgorithm::Yuv => self.yuv_gamut_map(*pixel, config),
                GamutMappingAlgorithm::Saturation => self.saturation_gamut_map(*pixel, config),
                GamutMappingAlgorithm::Perceptual => self.perceptual_gamut_map(*pixel, config),
            };
        }

        Ok(gamut_mapped)
    }


    fn itp_gamut_map(&self, pixel: super::types::HdrPixel, config: &GamutMappingConfig) -> super::types::HdrPixel {

        let (l, m, s) = self.rgb_to_itp(pixel.r, pixel.g, pixel.b);


        let (l_mapped, m_mapped, s_mapped) = self.map_itp_gamut(l, m, s, config);


        let (r, g, b) = self.itp_to_rgb(l_mapped, m_mapped, s_mapped);

        super::types::HdrPixel { r, g, b }
    }


    fn rgb_to_itp(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {

        let l = (r + g + b) / 3.0;
        let m = (r - g) / 2.0;
        let s = (r + g - 2.0 * b) / 6.0;

        (l, m, s)
    }


    fn itp_to_rgb(&self, l: f32, m: f32, s: f32) -> (f32, f32, f32) {
        let r = l + m + s;
        let g = l - m + s;
        let b = l - 2.0 * s;

        (r, g, b)
    }


    fn map_itp_gamut(&self, l: f32, m: f32, s: f32, config: &GamutMappingConfig) -> (f32, f32, f32) {
        let saturation_factor = config.saturation_preservation;

        let l_mapped = l;
        let m_mapped = m * saturation_factor;
        let s_mapped = s * saturation_factor;

        (l_mapped, m_mapped, s_mapped)
    }


    fn yuv_gamut_map(&self, pixel: super::types::HdrPixel, config: &GamutMappingConfig) -> super::types::HdrPixel {

        let y = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
        let u = (pixel.b - y) * 0.5;
        let v = (pixel.r - y) * 0.5;


        let saturation_factor = config.saturation_preservation;
        let y_mapped = y;
        let u_mapped = u * saturation_factor;
        let v_mapped = v * saturation_factor;


        let r = y_mapped + 1.403 * v_mapped;
        let g = y_mapped - 0.344 * u_mapped - 0.714 * v_mapped;
        let b = y_mapped + 1.770 * u_mapped;

        super::types::HdrPixel { r, g, b }
    }


    fn saturation_gamut_map(&self, pixel: super::types::HdrPixel, config: &GamutMappingConfig) -> super::types::HdrPixel {
        let luma = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
        let saturation_factor = config.saturation_preservation;

        let r = luma + (pixel.r - luma) * saturation_factor;
        let g = luma + (pixel.g - luma) * saturation_factor;
        let b = luma + (pixel.b - luma) * saturation_factor;

        super::types::HdrPixel { r, g, b }
    }


    fn perceptual_gamut_map(&self, pixel: super::types::HdrPixel, config: &GamutMappingConfig) -> super::types::HdrPixel {

        let luma = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
        let saturation_factor = config.saturation_preservation;


        let r_weight = 0.299;
        let g_weight = 0.587;
        let b_weight = 0.114;

        let r_perceptual = luma + (pixel.r - luma) * saturation_factor * r_weight;
        let g_perceptual = luma + (pixel.g - luma) * saturation_factor * g_weight;
        let b_perceptual = luma + (pixel.b - luma) * saturation_factor * b_weight;

        super::types::HdrPixel {
            r: r_perceptual,
            g: g_perceptual,
            b: b_perceptual,
        }
    }


    pub fn update_config(&mut self, config: GamutMappingConfig) {
        self.config = config;
    }


    pub fn get_available_algorithms(&self) -> Vec<GamutMappingAlgorithm> {
        vec![
            GamutMappingAlgorithm::Itp,
            GamutMappingAlgorithm::Yuv,
            GamutMappingAlgorithm::Saturation,
            GamutMappingAlgorithm::Perceptual,
        ]
    }


    pub fn get_algorithm_description(&self, algorithm: GamutMappingAlgorithm) -> &'static str {
        match algorithm {
            GamutMappingAlgorithm::Itp => "ICTCP perceptual color space mapping",
            GamutMappingAlgorithm::Yuv => "Traditional YUV color space mapping",
            GamutMappingAlgorithm::Saturation => "Saturation-preserving gamut mapping",
            GamutMappingAlgorithm::Perceptual => "Weighted perceptual gamut mapping",
        }
    }


    pub fn get_algorithm_characteristics(&self, algorithm: GamutMappingAlgorithm) -> GamutMappingCharacteristics {
        match algorithm {
            GamutMappingAlgorithm::Itp => GamutMappingCharacteristics {
                preserves_saturation: 0.9,
                preserves_hue: 0.95,
                computational_cost: 2.0,
                perceptual_accuracy: 0.95,
            },
            GamutMappingAlgorithm::Yuv => GamutMappingCharacteristics {
                preserves_saturation: 0.7,
                preserves_hue: 0.8,
                computational_cost: 1.0,
                perceptual_accuracy: 0.7,
            },
            GamutMappingAlgorithm::Saturation => GamutMappingCharacteristics {
                preserves_saturation: 1.0,
                preserves_hue: 0.9,
                computational_cost: 0.5,
                perceptual_accuracy: 0.6,
            },
            GamutMappingAlgorithm::Perceptual => GamutMappingCharacteristics {
                preserves_saturation: 0.8,
                preserves_hue: 0.85,
                computational_cost: 1.5,
                perceptual_accuracy: 0.85,
            },
        }
    }


    pub fn convert_color_primaries(
        &self,
        pixel: super::types::HdrPixel,
        from: super::types::ColorPrimaries,
        to: super::types::ColorPrimaries,
    ) -> Result<super::types::HdrPixel> {
        if from == to {
            return Ok(pixel);
        }

        let matrix = self.get_conversion_matrix(from, to)?;
        let r = matrix[0][0] * pixel.r + matrix[0][1] * pixel.g + matrix[0][2] * pixel.b;
        let g = matrix[1][0] * pixel.r + matrix[1][1] * pixel.g + matrix[1][2] * pixel.b;
        let b = matrix[2][0] * pixel.r + matrix[2][1] * pixel.g + matrix[2][2] * pixel.b;

        Ok(super::types::HdrPixel { r, g, b })
    }


    fn get_conversion_matrix(&self, from: super::types::ColorPrimaries, to: super::types::ColorPrimaries) -> Result<[[f32; 3]; 3]> {
        match (from, to) {
            (super::types::ColorPrimaries::Rec709, super::types::ColorPrimaries::Rec2020) => {
                Ok([[0.627404, 0.329283, 0.043313],
                    [0.069097, 0.919540, 0.011363],
                    [0.016391, 0.087013, 0.896596]])
            }
            (super::types::ColorPrimaries::Rec2020, super::types::ColorPrimaries::Rec709) => {
                Ok([[1.641023, -0.324803, -0.316216],
                    [-0.418194, 1.279050, 0.139144],
                    [-0.016279, -0.042748, 1.059027]])
            }
            (super::types::ColorPrimaries::Rec709, super::types::ColorPrimaries::P3) => {
                Ok([[0.822462, 0.177538, 0.0],
                    [0.033194, 0.966806, 0.0],
                    [0.017083, 0.072697, 0.910220]])
            }
            (super::types::ColorPrimaries::P3, super::types::ColorPrimaries::Rec709) => {
                Ok([[1.224940, -0.224940, 0.0],
                    [-0.042057, 1.042057, 0.0],
                    [-0.019637, -0.078636, 1.098273]])
            }
            _ => Err(anyhow!("Unsupported color space conversion: {:?} to {:?}", from, to))
        }
    }
}


#[derive(Debug, Clone)]
pub struct GamutMappingCharacteristics {
    pub preserves_saturation: f32,
    pub preserves_hue: f32,
    pub computational_cost: f32,
    pub perceptual_accuracy: f32,
}

impl Default for GamutMapper {
    fn default() -> Self {
        Self::new(GamutMappingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamut_mapper_creation() {
        let config = GamutMappingConfig::default();
        let mapper = GamutMapper::new(config);
        assert!(matches!(mapper.config.algorithm, GamutMappingAlgorithm::Itp));
    }

    #[test]
    fn test_available_algorithms() {
        let mapper = GamutMapper::default();
        let algorithms = mapper.get_available_algorithms();

        assert!(algorithms.contains(&GamutMappingAlgorithm::Itp));
        assert!(algorithms.contains(&GamutMappingAlgorithm::Yuv));
        assert!(algorithms.contains(&GamutMappingAlgorithm::Saturation));
        assert!(algorithms.contains(&GamutMappingAlgorithm::Perceptual));
    }

    #[test]
    fn test_algorithm_descriptions() {
        let mapper = GamutMapper::default();

        let itp_desc = mapper.get_algorithm_description(GamutMappingAlgorithm::Itp);
        assert!(itp_desc.contains("ICTCP"));

        let saturation_desc = mapper.get_algorithm_description(GamutMappingAlgorithm::Saturation);
        assert!(saturation_desc.contains("saturation"));
    }

    #[test]
    fn test_algorithm_characteristics() {
        let mapper = GamutMapper::default();

        let itp_chars = mapper.get_algorithm_characteristics(GamutMappingAlgorithm::Itp);
        assert!(itp_chars.preserves_saturation > 0.8);
        assert!(itp_chars.preserves_hue > 0.8);
        assert!(itp_chars.computational_cost > 1.0);

        let saturation_chars = mapper.get_algorithm_characteristics(GamutMappingAlgorithm::Saturation);
        assert_eq!(saturation_chars.preserves_saturation, 1.0);
        assert!(saturation_chars.computational_cost < 1.0);
    }

    #[test]
    fn test_color_primaries_conversion() {
        let mapper = GamutMapper::default();

        let pixel = super::types::HdrPixel { r: 100.0, g: 200.0, b: 300.0 };


        let converted = mapper.convert_color_primaries(
            pixel,
            super::types::ColorPrimaries::Rec709,
            super::types::ColorPrimaries::Rec2020,
        );
        assert!(converted.is_ok());

        let converted_pixel = converted.unwrap();

        assert!(converted_pixel.r != pixel.r || converted_pixel.g != pixel.g || converted_pixel.b != pixel.b);


        let identity = mapper.convert_color_primaries(
            pixel,
            super::types::ColorPrimaries::Rec709,
            super::types::ColorPrimaries::Rec709,
        );
        assert!(identity.is_ok());

        let identity_pixel = identity.unwrap();
        assert_eq!(identity_pixel.r, pixel.r);
        assert_eq!(identity_pixel.g, pixel.g);
        assert_eq!(identity_pixel.b, pixel.b);
    }

    #[test]
    fn test_saturation_gamut_mapping() {
        let mapper = GamutMapper::default();
        let config = GamutMappingConfig {
            algorithm: GamutMappingAlgorithm::Saturation,
            source_gamut: super::types::ColorPrimaries::Rec2020,
            target_gamut: super::types::ColorPrimaries::Rec709,
            saturation_preservation: 0.5,
        };

        let pixel = super::types::HdrPixel { r: 100.0, g: 200.0, b: 300.0 };
        let mapped = mapper.saturation_gamut_map(pixel, &config);


        let original_luma = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
        let mapped_luma = 0.2126 * mapped.r + 0.7152 * mapped.g + 0.0722 * mapped.b;

        assert!((mapped_luma - original_luma).abs() < 0.01);
    }
}
