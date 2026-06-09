use anyhow::{Result, anyhow};
use log::debug;

use super::types::HdrImage;
use super::config::ToneMappingConfig;
use super::display::HdrDisplayProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneMappingAlgorithm {
    Reinhard,
    Filmic,
    Aces,
    Hable,
    Drago,
}

pub struct ToneMapper {
    config: ToneMappingConfig,
}

impl ToneMapper {

    pub fn new(config: ToneMappingConfig) -> Self {
        Self { config }
    }

    pub fn apply_tone_mapping(
        &self,
        hdr_image: &HdrImage,
        display_info: &HdrDisplayProfile,
        config: &ToneMappingConfig,
    ) -> Result<HdrImage> {
        debug!("Applying tone mapping with algorithm: {:?}", config.algorithm);

        let mut tone_mapped = hdr_image.clone();

        for pixel in &mut tone_mapped.data {
            *pixel = match config.algorithm {
                ToneMappingAlgorithm::Reinhard => self.reinhard_tone_map(*pixel, display_info, config),
                ToneMappingAlgorithm::Filmic => self.filmic_tone_map(*pixel, display_info, config),
                ToneMappingAlgorithm::Aces => self.aces_tone_map(*pixel, display_info, config),
                ToneMappingAlgorithm::Hable => self.hable_tone_map(*pixel, display_info, config),
                ToneMappingAlgorithm::Drago => self.drago_tone_map(*pixel, display_info, config),
            };
        }

        Ok(tone_mapped)
    }


    fn reinhard_tone_map(&self, pixel: super::types::HdrPixel, display: &HdrDisplayProfile, config: &ToneMappingConfig) -> super::types::HdrPixel {
        let scale = 1.0 / display.peak_luminance;
        let r = pixel.r * scale;
        let g = pixel.g * scale;
        let b = pixel.b * scale;

        let r_reinhard = r / (1.0 + r);
        let g_reinhard = g / (1.0 + g);
        let b_reinhard = b / (1.0 + b);

        super::types::HdrPixel {
            r: r_reinhard * display.peak_luminance,
            g: g_reinhard * display.peak_luminance,
            b: b_reinhard * display.peak_luminance,
        }
    }


    fn filmic_tone_map(&self, pixel: super::types::HdrPixel, display: &HdrDisplayProfile, config: &ToneMappingConfig) -> super::types::HdrPixel {
        let shoulder_strength = config.shoulder_strength;
        let mid_tone = config.mid_tone;
        let highlight_strength = config.highlight_strength;

        let r = self.filmic_curve(pixel.r / display.peak_luminance, shoulder_strength, mid_tone, highlight_strength);
        let g = self.filmic_curve(pixel.g / display.peak_luminance, shoulder_strength, mid_tone, highlight_strength);
        let b = self.filmic_curve(pixel.b / display.peak_luminance, shoulder_strength, mid_tone, highlight_strength);

        super::types::HdrPixel {
            r: r * display.peak_luminance,
            g: g * display.peak_luminance,
            b: b * display.peak_luminance,
        }
    }


    fn filmic_curve(&self, x: f32, shoulder: f32, mid: f32, highlight: f32) -> f32 {
        let a = shoulder;
        let b = mid;
        let c = highlight;

        if x < b {
            x * (a / b)
        } else {
            a + (x - b) * (c - a) / (1.0 - b)
        }
    }


    fn aces_tone_map(&self, pixel: super::types::HdrPixel, display: &HdrDisplayProfile, config: &ToneMappingConfig) -> super::types::HdrPixel {
        let a = 2.51;
        let b = 0.03;
        let c = 2.43;
        let d = 0.59;
        let e = 0.14;

        let scale = 1.0 / display.peak_luminance;
        let r = pixel.r * scale;
        let g = pixel.g * scale;
        let b = pixel.b * scale;

        let r_aces = (r * (a * r + b)) / (r * (c * r + d) + e);
        let g_aces = (g * (a * g + b)) / (g * (c * g + d) + e);
        let b_aces = (b * (a * b + b)) / (b * (c * b + d) + e);

        super::types::HdrPixel {
            r: r_aces * display.peak_luminance,
            g: g_aces * display.peak_luminance,
            b: b_aces * display.peak_luminance,
        }
    }


    fn hable_tone_map(&self, pixel: super::types::HdrPixel, display: &HdrDisplayProfile, config: &ToneMappingConfig) -> super::types::HdrPixel {
        let a = 0.22;
        let b = 0.30;
        let c = 0.10;
        let d = 0.20;
        let e = 0.01;
        let f = 0.30;

        let scale = 1.0 / display.peak_luminance;
        let r = pixel.r * scale;
        let g = pixel.g * scale;
        let b = pixel.b * scale;

        let hable = |x: f32| -> f32 {
            ((x * (a * x + b) + c) / (x * (d * x + e) + f)) - e / f
        };

        let r_hable = hable(r);
        let g_hable = hable(g);
        let b_hable = hable(b);

        super::types::HdrPixel {
            r: r_hable * display.peak_luminance,
            g: g_hable * display.peak_luminance,
            b: b_hable * display.peak_luminance,
        }
    }


    fn drago_tone_map(&self, pixel: super::types::HdrPixel, display: &HdrDisplayProfile, config: &ToneMappingConfig) -> super::types::HdrPixel {
        let log_max = display.peak_luminance.log10();
        let bias = 0.85;

        let r = pixel.r / display.peak_luminance;
        let g = pixel.g / display.peak_luminance;
        let b = pixel.b / display.peak_luminance;

        let drago = |x: f32| -> f32 {
            let log_x = x.log10();
            (log_x * (1.0 + bias * log_x / log_max)) / (1.0 + bias)
        };

        let r_drago = drago(r);
        let g_drago = drago(g);
        let b_drago = drago(b);

        super::types::HdrPixel {
            r: r_drago * display.peak_luminance,
            g: g_drago * display.peak_luminance,
            b: b_drago * display.peak_luminance,
        }
    }


    pub fn update_config(&mut self, config: ToneMappingConfig) {
        self.config = config;
    }


    pub fn get_available_algorithms(&self) -> Vec<ToneMappingAlgorithm> {
        vec![
            ToneMappingAlgorithm::Reinhard,
            ToneMappingAlgorithm::Filmic,
            ToneMappingAlgorithm::Aces,
            ToneMappingAlgorithm::Hable,
            ToneMappingAlgorithm::Drago,
        ]
    }


    pub fn get_algorithm_description(&self, algorithm: ToneMappingAlgorithm) -> &'static str {
        match algorithm {
            ToneMappingAlgorithm::Reinhard => "Classic photographic tone mapping with smooth highlights",
            ToneMappingAlgorithm::Filmic => "Film-like tone reproduction with cinematic look",
            ToneMappingAlgorithm::Aces => "Academy Color Encoding System tone mapping",
            ToneMappingAlgorithm::Hable => "Uncharted 2 game engine tone mapping",
            ToneMappingAlgorithm::Drago => "Adaptive logarithmic tone mapping for high contrast",
        }
    }


    pub fn get_algorithm_characteristics(&self, algorithm: ToneMappingAlgorithm) -> ToneMappingCharacteristics {
        match algorithm {
            ToneMappingAlgorithm::Reinhard => ToneMappingCharacteristics {
                preserves_highlights: false,
                preserves_shadows: true,
                contrast_preservation: 0.7,
                saturation_preservation: 0.8,
                computational_cost: 1.0,
            },
            ToneMappingAlgorithm::Filmic => ToneMappingCharacteristics {
                preserves_highlights: true,
                preserves_shadows: true,
                contrast_preservation: 0.9,
                saturation_preservation: 0.9,
                computational_cost: 1.2,
            },
            ToneMappingAlgorithm::Aces => ToneMappingCharacteristics {
                preserves_highlights: true,
                preserves_shadows: true,
                contrast_preservation: 0.85,
                saturation_preservation: 0.85,
                computational_cost: 1.5,
            },
            ToneMappingAlgorithm::Hable => ToneMappingCharacteristics {
                preserves_highlights: true,
                preserves_shadows: false,
                contrast_preservation: 0.8,
                saturation_preservation: 0.7,
                computational_cost: 1.3,
            },
            ToneMappingAlgorithm::Drago => ToneMappingCharacteristics {
                preserves_highlights: false,
                preserves_shadows: true,
                contrast_preservation: 0.6,
                saturation_preservation: 0.6,
                computational_cost: 2.0,
            },
        }
    }
}


#[derive(Debug, Clone)]
pub struct ToneMappingCharacteristics {
    pub preserves_highlights: bool,
    pub preserves_shadows: bool,
    pub contrast_preservation: f32,
    pub saturation_preservation: f32,
    pub computational_cost: f32,
}

impl Default for ToneMapper {
    fn default() -> Self {
        Self::new(ToneMappingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_mapper_creation() {
        let config = ToneMappingConfig::default();
        let mapper = ToneMapper::new(config);
        assert!(matches!(mapper.config.algorithm, ToneMappingAlgorithm::Reinhard));
    }

    #[test]
    fn test_available_algorithms() {
        let mapper = ToneMapper::default();
        let algorithms = mapper.get_available_algorithms();

        assert!(algorithms.contains(&ToneMappingAlgorithm::Reinhard));
        assert!(algorithms.contains(&ToneMappingAlgorithm::Filmic));
        assert!(algorithms.contains(&ToneMappingAlgorithm::Aces));
        assert!(algorithms.contains(&ToneMappingAlgorithm::Hable));
        assert!(algorithms.contains(&ToneMappingAlgorithm::Drago));
    }

    #[test]
    fn test_algorithm_descriptions() {
        let mapper = ToneMapper::default();

        let reinhard_desc = mapper.get_algorithm_description(ToneMappingAlgorithm::Reinhard);
        assert!(!reinhard_desc.is_empty());

        let aces_desc = mapper.get_algorithm_description(ToneMappingAlgorithm::Aces);
        assert!(aces_desc.contains("ACES"));
    }

    #[test]
    fn test_algorithm_characteristics() {
        let mapper = ToneMapper::default();

        let reinhard_chars = mapper.get_algorithm_characteristics(ToneMappingAlgorithm::Reinhard);
        assert!(!reinhard_chars.preserves_highlights);
        assert!(reinhard_chars.preserves_shadows);
        assert!(reinhard_chars.contrast_preservation > 0.0);
        assert!(reinhard_chars.contrast_preservation <= 1.0);

        let filmic_chars = mapper.get_algorithm_characteristics(ToneMappingAlgorithm::Filmic);
        assert!(filmic_chars.preserves_highlights);
        assert!(filmic_chars.preserves_shadows);
        assert!(filmic_chars.contrast_preservation > reinhard_chars.contrast_preservation);
    }

    #[test]
    fn test_reinhard_tone_mapping() {
        let mapper = ToneMapper::default();
        let display = super::super::display::HdrDisplayProfile {
            display_type: super::super::types::HdrDisplayType::Sdr,
            max_nits: 100.0,
            min_nits: 0.001,
            peak_luminance: 100.0,
            color_gamut: super::super::types::ColorPrimaries::Rec709,
            transfer_function: super::super::types::TransferFunction::Rec709,
        };

        let config = ToneMappingConfig::default();


        let bright_pixel = super::types::HdrPixel { r: 1000.0, g: 1000.0, b: 1000.0 };
        let mapped = mapper.reinhard_tone_map(bright_pixel, &display, &config);


        assert!(mapped.r < bright_pixel.r);
        assert!(mapped.g < bright_pixel.g);
        assert!(mapped.b < bright_pixel.b);


        let dark_pixel = super::types::HdrPixel { r: 10.0, g: 10.0, b: 10.0 };
        let mapped_dark = mapper.reinhard_tone_map(dark_pixel, &display, &config);


        assert!(mapped_dark.r >= dark_pixel.r);
        assert!(mapped_dark.g >= dark_pixel.g);
        assert!(mapped_dark.b >= dark_pixel.b);
    }
}
