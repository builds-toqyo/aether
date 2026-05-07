

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use log::debug;

use super::types::{HdrDisplayType, ColorPrimaries, TransferFunction};


pub struct HdrDisplayManager {
    config: super::config::HdrDisplayConfig,
    display_profiles: HashMap<HdrDisplayType, HdrDisplayProfile>,
}

impl HdrDisplayManager {

    pub fn new(config: super::config::HdrDisplayConfig) -> Result<Self> {
        let mut manager = Self {
            config: config.clone(),
            display_profiles: HashMap::new(),
        };

        manager.initialize_display_profiles()?;

        Ok(manager)
    }


    fn initialize_display_profiles(&mut self) -> Result<()> {
        debug!("Initializing HDR display profiles");


        self.display_profiles.insert(
            HdrDisplayType::Sdr,
            HdrDisplayProfile {
                display_type: HdrDisplayType::Sdr,
                max_nits: 100.0,
                min_nits: 0.001,
                peak_luminance: 100.0,
                color_gamut: ColorPrimaries::Rec709,
                transfer_function: TransferFunction::Rec709,
            },
        );


        self.display_profiles.insert(
            HdrDisplayType::Hdr10,
            HdrDisplayProfile {
                display_type: HdrDisplayType::Hdr10,
                max_nits: 1000.0,
                min_nits: 0.001,
                peak_luminance: 1000.0,
                color_gamut: ColorPrimaries::Rec2020,
                transfer_function: TransferFunction::Pq,
            },
        );


        self.display_profiles.insert(
            HdrDisplayType::DolbyVision,
            HdrDisplayProfile {
                display_type: HdrDisplayType::DolbyVision,
                max_nits: 10000.0,
                min_nits: 0.001,
                peak_luminance: 4000.0,
                color_gamut: ColorPrimaries::P3,
                transfer_function: TransferFunction::Pq,
            },
        );


        self.display_profiles.insert(
            HdrDisplayType::Hdr10Plus,
            HdrDisplayProfile {
                display_type: HdrDisplayType::Hdr10Plus,
                max_nits: 4000.0,
                min_nits: 0.001,
                peak_luminance: 1500.0,
                color_gamut: ColorPrimaries::Rec2020,
                transfer_function: TransferFunction::Pq,
            },
        );


        self.display_profiles.insert(
            HdrDisplayType::Hlg,
            HdrDisplayProfile {
                display_type: HdrDisplayType::Hlg,
                max_nits: 1000.0,
                min_nits: 0.001,
                peak_luminance: 1000.0,
                color_gamut: ColorPrimaries::Rec2020,
                transfer_function: TransferFunction::Hlg,
            },
        );

        Ok(())
    }


    pub fn get_display_info(&self, display_type: HdrDisplayType) -> Result<HdrDisplayProfile> {
        self.display_profiles.get(&display_type)
            .cloned()
            .ok_or_else(|| anyhow!("Display profile not found: {:?}", display_type))
    }


    pub fn update_config(&mut self, config: super::config::HdrDisplayConfig) -> Result<()> {
        debug!("Updating display manager configuration");
        self.config = config;
        Ok(())
    }


    pub fn get_available_displays(&self) -> Vec<HdrDisplayType> {
        self.display_profiles.keys().copied().collect()
    }


    pub fn get_display_capabilities(&self, display_type: HdrDisplayType) -> Result<DisplayCapabilities> {
        let profile = self.get_display_info(display_type)?;

        Ok(DisplayCapabilities {
            hdr_capable: profile.max_nits > 100.0,
            max_luminance: profile.max_nits,
            min_luminance: profile.min_nits,
            contrast_ratio: profile.max_nits / profile.min_nits,
            color_gamut: profile.color_gamut,
            transfer_function: profile.transfer_function,
            supports_dynamic_metadata: matches!(display_type, HdrDisplayType::DolbyVision | HdrDisplayType::Hdr10Plus),
        })
    }


    pub fn detect_display_type(&self, max_nits: f32, color_gamut: ColorPrimaries) -> HdrDisplayType {
        if max_nits <= 100.0 {
            HdrDisplayType::Sdr
        } else if max_nits <= 1000.0 && matches!(color_gamut, ColorPrimaries::Rec2020) {
            HdrDisplayType::Hdr10
        } else if max_nits <= 1000.0 && matches!(color_gamut, ColorPrimaries::Rec709) {
            HdrDisplayType::Hlg
        } else if max_nits <= 4000.0 {
            HdrDisplayType::Hdr10Plus
        } else {
            HdrDisplayType::DolbyVision
        }
    }


    pub fn get_optimal_display(&self, hdr_image: &super::types::HdrImage) -> Result<HdrDisplayType> {
        let detected_type = self.detect_display_type(hdr_image.max_nits, hdr_image.color_primaries);


        if self.display_profiles.contains_key(&detected_type) {
            Ok(detected_type)
        } else {
            Ok(self.config.default_display)
        }
    }
}


#[derive(Debug, Clone)]
pub struct HdrDisplayProfile {
    pub display_type: HdrDisplayType,
    pub max_nits: f32,
    pub min_nits: f32,
    pub peak_luminance: f32,
    pub color_gamut: ColorPrimaries,
    pub transfer_function: TransferFunction,
}


#[derive(Debug, Clone)]
pub struct DisplayCapabilities {
    pub hdr_capable: bool,
    pub max_luminance: f32,
    pub min_luminance: f32,
    pub contrast_ratio: f32,
    pub color_gamut: ColorPrimaries,
    pub transfer_function: TransferFunction,
    pub supports_dynamic_metadata: bool,
}

impl DisplayCapabilities {

    pub fn is_hdr_capable(&self) -> bool {
        self.hdr_capable
    }


    pub fn dynamic_range_stops(&self) -> f32 {
        (self.max_luminance / self.min_luminance).log2()
    }


    pub fn is_wide_gamut(&self) -> bool {
        matches!(self.color_gamut, ColorPrimaries::Rec2020 | ColorPrimaries::P3 | ColorPrimaries::DciP3)
    }


    pub fn supports_dolby_vision(&self) -> bool {
        matches!(self.transfer_function, TransferFunction::Pq) && self.supports_dynamic_metadata
    }
}

impl Default for HdrDisplayManager {
    fn default() -> Self {
        Self::new(super::config::HdrDisplayConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_manager_creation() {
        let config = super::config::HdrDisplayConfig::default();
        let manager = HdrDisplayManager::new(config);
        assert!(manager.is_ok());

        let display_manager = manager.unwrap();
        let displays = display_manager.get_available_displays();

        assert!(displays.contains(&HdrDisplayType::Sdr));
        assert!(displays.contains(&HdrDisplayType::Hdr10));
        assert!(displays.contains(&HdrDisplayType::DolbyVision));
        assert!(displays.contains(&HdrDisplayType::Hdr10Plus));
        assert!(displays.contains(&HdrDisplayType::Hlg));
    }

    #[test]
    fn test_display_profiles() {
        let manager = HdrDisplayManager::default();


        let sdr_profile = manager.get_display_info(HdrDisplayType::Sdr);
        assert!(sdr_profile.is_ok());
        let profile = sdr_profile.unwrap();
        assert_eq!(profile.max_nits, 100.0);
        assert_eq!(profile.color_gamut, ColorPrimaries::Rec709);


        let hdr10_profile = manager.get_display_info(HdrDisplayType::Hdr10);
        assert!(hdr10_profile.is_ok());
        let profile = hdr10_profile.unwrap();
        assert_eq!(profile.max_nits, 1000.0);
        assert_eq!(profile.color_gamut, ColorPrimaries::Rec2020);


        let dv_profile = manager.get_display_info(HdrDisplayType::DolbyVision);
        assert!(dv_profile.is_ok());
        let profile = dv_profile.unwrap();
        assert_eq!(profile.max_nits, 10000.0);
        assert_eq!(profile.color_gamut, ColorPrimaries::P3);
    }

    #[test]
    fn test_display_capabilities() {
        let manager = HdrDisplayManager::default();

        let sdr_caps = manager.get_display_capabilities(HdrDisplayType::Sdr);
        assert!(sdr_caps.is_ok());
        let caps = sdr_caps.unwrap();
        assert!(!caps.is_hdr_capable());
        assert!(!caps.is_wide_gamut());
        assert!(!caps.supports_dolby_vision());

        let hdr10_caps = manager.get_display_capabilities(HdrDisplayType::Hdr10);
        assert!(hdr10_caps.is_ok());
        let caps = hdr10_caps.unwrap();
        assert!(caps.is_hdr_capable());
        assert!(caps.is_wide_gamut());
        assert!(!caps.supports_dolby_vision());

        let dv_caps = manager.get_display_capabilities(HdrDisplayType::DolbyVision);
        assert!(dv_caps.is_ok());
        let caps = dv_caps.unwrap();
        assert!(caps.is_hdr_capable());
        assert!(caps.is_wide_gamut());
        assert!(caps.supports_dolby_vision());
    }

    #[test]
    fn test_display_type_detection() {
        let manager = HdrDisplayManager::default();


        let sdr_type = manager.detect_display_type(100.0, ColorPrimaries::Rec709);
        assert_eq!(sdr_type, HdrDisplayType::Sdr);


        let hdr10_type = manager.detect_display_type(1000.0, ColorPrimaries::Rec2020);
        assert_eq!(hdr10_type, HdrDisplayType::Hdr10);


        let hlg_type = manager.detect_display_type(1000.0, ColorPrimaries::Rec709);
        assert_eq!(hlg_type, HdrDisplayType::Hlg);


        let dv_type = manager.detect_display_type(10000.0, ColorPrimaries::P3);
        assert_eq!(dv_type, HdrDisplayType::DolbyVision);
    }
}
