#[derive(Debug, Clone)]
pub struct HdrConfig {
    pub display_config: HdrDisplayConfig,
    pub tone_mapping_config: ToneMappingConfig,
    pub gamut_mapping_config: GamutMappingConfig,
}

impl Default for HdrConfig {
    fn default() -> Self {
        Self {
            display_config: HdrDisplayConfig::default(),
            tone_mapping_config: ToneMappingConfig::default(),
            gamut_mapping_config: GamutMappingConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HdrDisplayConfig {
    pub default_display: crate::color::hdr::types::HdrDisplayType,
    pub max_display_nits: f32,
    pub min_display_nits: f32,
    pub peak_luminance: f32,
}

impl Default for HdrDisplayConfig {
    fn default() -> Self {
        Self {
            default_display: crate::color::hdr::types::HdrDisplayType::DolbyVision,
            max_display_nits: 10000.0,
            min_display_nits: 0.001,
            peak_luminance: 1000.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToneMappingConfig {
    pub algorithm: crate::color::hdr::tone::ToneMappingAlgorithm,
    pub shoulder_strength: f32,
    pub mid_tone: f32,
    pub highlight_strength: f32,
    pub contrast: f32,
}

impl Default for ToneMappingConfig {
    fn default() -> Self {
        Self {
            algorithm: crate::color::hdr::tone::ToneMappingAlgorithm::Reinhard,
            shoulder_strength: 0.8,
            mid_tone: 0.5,
            highlight_strength: 0.9,
            contrast: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GamutMappingConfig {
    pub algorithm: crate::color::hdr::gamut::GamutMappingAlgorithm,
    pub source_gamut: crate::color::hdr::types::ColorPrimaries,
    pub target_gamut: crate::color::hdr::types::ColorPrimaries,
    pub saturation_preservation: f32,
}

impl Default for GamutMappingConfig {
    fn default() -> Self {
        Self {
            algorithm: crate::color::hdr::gamut::GamutMappingAlgorithm::Itp,
            source_gamut: crate::color::hdr::types::ColorPrimaries::Rec2020,
            target_gamut: crate::color::hdr::types::ColorPrimaries::Rec709,
            saturation_preservation: 0.8,
        }
    }
}

impl HdrConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_display_config(mut self, config: HdrDisplayConfig) -> Self {
        self.display_config = config;
        self
    }

    pub fn with_tone_mapping_config(mut self, config: ToneMappingConfig) -> Self {
        self.tone_mapping_config = config;
        self
    }

    pub fn with_gamut_mapping_config(mut self, config: GamutMappingConfig) -> Self {
        self.gamut_mapping_config = config;
        self
    }

    pub fn with_default_display(mut self, display_type: crate::color::hdr::types::HdrDisplayType) -> Self {
        self.display_config.default_display = display_type;
        self
    }

    pub fn with_tone_mapping_algorithm(mut self, algorithm: crate::color::hdr::tone::ToneMappingAlgorithm) -> Self {
        self.tone_mapping_config.algorithm = algorithm;
        self
    }

    pub fn with_gamut_mapping_algorithm(mut self, algorithm: crate::color::hdr::gamut::GamutMappingAlgorithm) -> Self {
        self.gamut_mapping_config.algorithm = algorithm;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HdrConfig::default();

        assert!(matches!(config.display_config.default_display, crate::color::hdr::types::HdrDisplayType::DolbyVision));
        assert_eq!(config.display_config.max_display_nits, 10000.0);
        assert_eq!(config.display_config.min_display_nits, 0.001);
        assert_eq!(config.display_config.peak_luminance, 1000.0);

        assert!(matches!(config.tone_mapping_config.algorithm, crate::color::hdr::tone::ToneMappingAlgorithm::Reinhard));
        assert_eq!(config.tone_mapping_config.shoulder_strength, 0.8);
        assert_eq!(config.tone_mapping_config.mid_tone, 0.5);
        assert_eq!(config.tone_mapping_config.highlight_strength, 0.9);
        assert_eq!(config.tone_mapping_config.contrast, 1.0);

        assert!(matches!(config.gamut_mapping_config.algorithm, crate::color::hdr::gamut::GamutMappingAlgorithm::Itp));
        assert!(matches!(config.gamut_mapping_config.source_gamut, crate::color::hdr::types::ColorPrimaries::Rec2020));
        assert!(matches!(config.gamut_mapping_config.target_gamut, crate::color::hdr::types::ColorPrimaries::Rec709));
        assert_eq!(config.gamut_mapping_config.saturation_preservation, 0.8);
    }

    #[test]
    fn test_config_builder() {
        let config = HdrConfig::new()
            .with_default_display(crate::color::hdr::types::HdrDisplayType::Hdr10)
            .with_tone_mapping_algorithm(crate::color::hdr::tone::ToneMappingAlgorithm::Aces)
            .with_gamut_mapping_algorithm(crate::color::hdr::gamut::GamutMappingAlgorithm::Yuv);

        assert!(matches!(config.display_config.default_display, crate::color::hdr::types::HdrDisplayType::Hdr10));
        assert!(matches!(config.tone_mapping_config.algorithm, crate::color::hdr::tone::ToneMappingAlgorithm::Aces));
        assert!(matches!(config.gamut_mapping_config.algorithm, crate::color::hdr::gamut::GamutMappingAlgorithm::Yuv));
    }
}
