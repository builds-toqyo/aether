

#[derive(Debug, Clone)]
pub struct LutConfig {
    pub interpolation_quality: crate::color::luts::types::InterpolationQuality,
    pub color_space: crate::types::ColorSpace,
    pub clamp_output: bool,
    pub cache_enabled: bool,
    pub max_cache_size: usize,
}

impl Default for LutConfig {
    fn default() -> Self {
        Self {
            interpolation_quality: crate::color::luts::types::InterpolationQuality::Trilinear,
            color_space: crate::types::ColorSpace::Rec709,
            clamp_output: true,
            cache_enabled: true,
            max_cache_size: 100,
        }
    }
}

impl LutConfig {

    pub fn new() -> Self {
        Self::default()
    }


    pub fn with_interpolation_quality(mut self, quality: crate::color::luts::types::InterpolationQuality) -> Self {
        self.interpolation_quality = quality;
        self
    }


    pub fn with_color_space(mut self, color_space: crate::types::ColorSpace) -> Self {
        self.color_space = color_space;
        self
    }


    pub fn with_clamp_output(mut self, clamp: bool) -> Self {
        self.clamp_output = clamp;
        self
    }


    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }


    pub fn with_max_cache_size(mut self, size: usize) -> Self {
        self.max_cache_size = size;
        self
    }


    pub fn validate(&self) -> Result<(), String> {
        if self.max_cache_size == 0 {
            return Err("Maximum cache size must be greater than 0".to_string());
        }

        Ok(())
    }


    pub fn summary(&self) -> String {
        format!(
            "LUT Config: {} interpolation, {} color space, clamp: {}, cache: {} (max {})",
            self.interpolation_quality.description(),
            format!("{:?}", self.color_space),
            self.clamp_output,
            self.cache_enabled,
            self.max_cache_size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::luts::types::InterpolationQuality;

    #[test]
    fn test_default_config() {
        let config = LutConfig::default();

        assert!(matches!(config.interpolation_quality, InterpolationQuality::Trilinear));
        assert!(matches!(config.color_space, crate::types::ColorSpace::Rec709));
        assert!(config.clamp_output);
        assert!(config.cache_enabled);
        assert_eq!(config.max_cache_size, 100);
    }

    #[test]
    fn test_config_builder() {
        let config = LutConfig::new()
            .with_interpolation_quality(InterpolationQuality::Linear)
            .with_color_space(crate::types::ColorSpace::Rec2020)
            .with_clamp_output(false)
            .with_cache(false)
            .with_max_cache_size(50);

        assert!(matches!(config.interpolation_quality, InterpolationQuality::Linear));
        assert!(matches!(config.color_space, crate::types::ColorSpace::Rec2020));
        assert!(!config.clamp_output);
        assert!(!config.cache_enabled);
        assert_eq!(config.max_cache_size, 50);
    }

    #[test]
    fn test_config_validation() {
        let config = LutConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = LutConfig::new().with_max_cache_size(0);
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_summary() {
        let config = LutConfig::default();
        let summary = config.summary();

        assert!(!summary.is_empty());
        assert!(summary.contains("LUT Config"));
        assert!(summary.contains("interpolation"));
        assert!(summary.contains("color space"));
    }
}
