//! HDR Support Implementation
//! 
//! This module provides comprehensive HDR (High Dynamic Range) support including
//! HDR display management, tone mapping algorithms, and gamut mapping for professional
//! video production workflows.

use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use log::{debug, info, warn, error};
use image::{Rgb, RgbImage};

use crate::types::{ColorSpace, VideoRange};

/// HDR processor for professional HDR workflows
pub struct HdrProcessor {
    config: HdrConfig,
    display_manager: HdrDisplayManager,
    tone_mapper: ToneMapper,
    gamut_mapper: GamutMapper,
}

impl HdrProcessor {
    /// Create a new HDR processor
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
    
    /// Process HDR image with tone mapping and gamut mapping
    pub fn process_hdr_image(
        &self,
        hdr_image: &HdrImage,
        target_display: HdrDisplayType,
        output_color_space: ColorSpace,
    ) -> Result<RgbImage> {
        debug!("Processing HDR image for display: {:?}", target_display);
        
        // Step 1: Get display characteristics
        let display_info = self.display_manager.get_display_info(target_display)?;
        
        // Step 2: Apply tone mapping
        let tone_mapped = self.tone_mapper.apply_tone_mapping(
            hdr_image,
            &display_info,
            &self.config.tone_mapping_config,
        )?;
        
        // Step 3: Apply gamut mapping
        let gamut_mapped = self.gamut_mapper.apply_gamut_mapping(
            &tone_mapped,
            output_color_space,
            &self.config.gamut_mapping_config,
        )?;
        
        // Step 4: Convert to standard RGB image
        let output_image = self.hdr_to_sdr(&gamut_mapped, output_color_space)?;
        
        debug!("HDR image processed successfully");
        
        Ok(output_image)
    }
    
    /// Convert SDR image to HDR
    pub fn sdr_to_hdr(&self, sdr_image: &RgbImage, target_nits: f32) -> Result<HdrImage> {
        debug!("Converting SDR to HDR with {} nits", target_nits);
        
        let (width, height) = sdr_image.dimensions();
        let mut hdr_data = vec![HdrPixel::default(); (width * height) as usize];
        
        for y in 0..height {
            for x in 0..width {
                let pixel = sdr_image.get_pixel(x, y);
                let [r, g, b] = pixel.0;
                
                // Convert SDR (0-255) to HDR (0-nits)
                let hdr_pixel = HdrPixel {
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
            color_primaries: ColorPrimaries::Rec709,
            transfer_function: TransferFunction::Pq, // Assume PQ for HDR
            max_nits: target_nits,
        };
        
        debug!("SDR to HDR conversion completed");
        
        Ok(hdr_image)
    }
    
    /// Convert HDR image to SDR
    fn hdr_to_sdr(&self, hdr_image: &HdrImage, output_color_space: ColorSpace) -> Result<RgbImage> {
        let (width, height) = (hdr_image.width, hdr_image.height);
        let mut sdr_image = RgbImage::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let hdr_pixel = &hdr_image.data[(y * width + x) as usize];
                
                // Convert HDR nits to SDR 0-255
                let sdr_r = (hdr_pixel.r / 1000.0 * 255.0).clamp(0.0, 255.0) as u8;
                let sdr_g = (hdr_pixel.g / 1000.0 * 255.0).clamp(0.0, 255.0) as u8;
                let sdr_b = (hdr_pixel.b / 1000.0 * 255.0).clamp(0.0, 255.0) as u8;
                
                sdr_image.put_pixel(x, y, Rgb([sdr_r, sdr_g, sdr_b]));
            }
        }
        
        Ok(sdr_image)
    }
    
    /// Analyze HDR content
    pub fn analyze_hdr_content(&self, hdr_image: &HdrImage) -> Result<HdrAnalysis> {
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
        
        let analysis = HdrAnalysis {
            max_nits,
            avg_nits,
            dynamic_range: max_nits / avg_nits.max(0.1),
            peak_percentage: (max_nits / 10000.0 * 100.0).min(100.0),
            content_type: self.classify_content_type(max_nits, avg_nits),
        };
        
        debug!("HDR analysis: {:?}", analysis);
        
        Ok(analysis)
    }
    
    /// Classify content type based on HDR characteristics
    fn classify_content_type(&self, max_nits: f32, avg_nits: f32) -> HdrContentType {
        if max_nits > 1000.0 {
            HdrContentType::TrueHdr
        } else if max_nits > 400.0 {
            HdrContentType::EnhancedHdr
        } else if max_nits > 200.0 {
            HdrContentType::LimitedHdr
        } else {
            HdrContentType::SdrUpscaled
        }
    }
    
    /// Update HDR configuration
    pub fn update_config(&mut self, config: HdrConfig) -> Result<()> {
        debug!("Updating HDR configuration");
        
        self.config = config.clone();
        self.display_manager.update_config(config.display_config)?;
        self.tone_mapper.update_config(config.tone_mapping_config);
        self.gamut_mapper.update_config(config.gamut_mapping_config);
        
        info!("HDR configuration updated");
        
        Ok(())
    }
    
    /// Get available HDR displays
    pub fn get_available_displays(&self) -> Vec<HdrDisplayType> {
        self.display_manager.get_available_displays()
    }
    
    /// Get available tone mapping algorithms
    pub fn get_available_tone_mappers(&self) -> Vec<ToneMappingAlgorithm> {
        self.tone_mapper.get_available_algorithms()
    }
    
    /// Get available gamut mapping algorithms
    pub fn get_available_gamut_mappers(&self) -> Vec<GamutMappingAlgorithm> {
        self.gamut_mapper.get_available_algorithms()
    }
}

impl Default for HdrProcessor {
    fn default() -> Self {
        Self::new(HdrConfig::default()).unwrap()
    }
}

/// HDR image representation
#[derive(Debug, Clone)]
pub struct HdrImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<HdrPixel>,
    pub color_primaries: ColorPrimaries,
    pub transfer_function: TransferFunction,
    pub max_nits: f32,
}

/// HDR pixel with nits values
#[derive(Debug, Clone, Copy, Default)]
pub struct HdrPixel {
    pub r: f32, // Red in nits
    pub g: f32, // Green in nits
    pub b: f32, // Blue in nits
}

/// HDR configuration
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

/// HDR display configuration
#[derive(Debug, Clone)]
pub struct HdrDisplayConfig {
    pub default_display: HdrDisplayType,
    pub max_display_nits: f32,
    pub min_display_nits: f32,
    pub peak_luminance: f32,
}

impl Default for HdrDisplayConfig {
    fn default() -> Self {
        Self {
            default_display: HdrDisplayType::DolbyVision,
            max_display_nits: 10000.0,
            min_display_nits: 0.001,
            peak_luminance: 1000.0,
        }
    }
}

/// Tone mapping configuration
#[derive(Debug, Clone)]
pub struct ToneMappingConfig {
    pub algorithm: ToneMappingAlgorithm,
    pub shoulder_strength: f32,
    pub mid_tone: f32,
    pub highlight_strength: f32,
    pub contrast: f32,
}

impl Default for ToneMappingConfig {
    fn default() -> Self {
        Self {
            algorithm: ToneMappingAlgorithm::Reinhard,
            shoulder_strength: 0.8,
            mid_tone: 0.5,
            highlight_strength: 0.9,
            contrast: 1.0,
        }
    }
}

/// Gamut mapping configuration
#[derive(Debug, Clone)]
pub struct GamutMappingConfig {
    pub algorithm: GamutMappingAlgorithm,
    pub source_gamut: ColorPrimaries,
    pub target_gamut: ColorPrimaries,
    pub saturation_preservation: f32,
}

impl Default for GamutMappingConfig {
    fn default() -> Self {
        Self {
            algorithm: GamutMappingAlgorithm::Itp,
            source_gamut: ColorPrimaries::Rec2020,
            target_gamut: ColorPrimaries::Rec709,
            saturation_preservation: 0.8,
        }
    }
}

/// HDR display types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdrDisplayType {
    Sdr,
    Hdr10,
    DolbyVision,
    Hdr10Plus,
    Hlg,
}

/// Color primaries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPrimaries {
    Rec709,
    Rec2020,
    P3,
    DciP3,
}

/// Transfer functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferFunction {
    Srgb,
    Rec709,
    Rec2020,
    Pq,      // Perceptual Quantizer (HDR)
    Hlg,     // Hybrid Log-Gamma (HDR)
}

/// Tone mapping algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneMappingAlgorithm {
    Reinhard,
    Filmic,
    Aces,
    Hable,
    Drago,
}

/// Gamut mapping algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamutMappingAlgorithm {
    Itp,     // ICTCP
    Yuv,
    Saturation,
    Perceptual,
}

/// HDR content analysis
#[derive(Debug, Clone)]
pub struct HdrAnalysis {
    pub max_nits: f32,
    pub avg_nits: f32,
    pub dynamic_range: f32,
    pub peak_percentage: f32,
    pub content_type: HdrContentType,
}

/// HDR content types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdrContentType {
    SdrUpscaled,
    LimitedHdr,
    EnhancedHdr,
    TrueHdr,
}

/// HDR display manager
pub struct HdrDisplayManager {
    config: HdrDisplayConfig,
    display_profiles: std::collections::HashMap<HdrDisplayType, HdrDisplayProfile>,
}

impl HdrDisplayManager {
    /// Create new HDR display manager
    pub fn new(config: HdrDisplayConfig) -> Result<Self> {
        let mut manager = Self {
            config: config.clone(),
            display_profiles: std::collections::HashMap::new(),
        };
        
        manager.initialize_display_profiles()?;
        
        Ok(manager)
    }
    
    /// Initialize display profiles
    fn initialize_display_profiles(&mut self) -> Result<()> {
        // SDR display
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
        
        // HDR10 display
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
        
        // Dolby Vision display
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
        
        Ok(())
    }
    
    /// Get display information
    pub fn get_display_info(&self, display_type: HdrDisplayType) -> Result<HdrDisplayProfile> {
        self.display_profiles.get(&display_type)
            .cloned()
            .ok_or_else(|| anyhow!("Display profile not found: {:?}", display_type))
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: HdrDisplayConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
    
    /// Get available displays
    pub fn get_available_displays(&self) -> Vec<HdrDisplayType> {
        self.display_profiles.keys().copied().collect()
    }
}

/// HDR display profile
#[derive(Debug, Clone)]
pub struct HdrDisplayProfile {
    pub display_type: HdrDisplayType,
    pub max_nits: f32,
    pub min_nits: f32,
    pub peak_luminance: f32,
    pub color_gamut: ColorPrimaries,
    pub transfer_function: TransferFunction,
}

/// Tone mapper implementation
pub struct ToneMapper {
    config: ToneMappingConfig,
}

impl ToneMapper {
    /// Create new tone mapper
    pub fn new(config: ToneMappingConfig) -> Self {
        Self { config }
    }
    
    /// Apply tone mapping
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
    
    /// Reinhard tone mapping
    fn reinhard_tone_map(&self, pixel: HdrPixel, display: &HdrDisplayProfile, config: &ToneMappingConfig) -> HdrPixel {
        let scale = 1.0 / display.peak_luminance;
        let r = pixel.r * scale;
        let g = pixel.g * scale;
        let b = pixel.b * scale;
        
        let r_reinhard = r / (1.0 + r);
        let g_reinhard = g / (1.0 + g);
        let b_reinhard = b / (1.0 + b);
        
        HdrPixel {
            r: r_reinhard * display.peak_luminance,
            g: g_reinhard * display.peak_luminance,
            b: b_reinhard * display.peak_luminance,
        }
    }
    
    /// Filmic tone mapping
    fn filmic_tone_map(&self, pixel: HdrPixel, display: &HdrDisplayProfile, config: &ToneMappingConfig) -> HdrPixel {
        let shoulder_strength = config.shoulder_strength;
        let mid_tone = config.mid_tone;
        let highlight_strength = config.highlight_strength;
        
        let r = self.filmic_curve(pixel.r / display.peak_luminance, shoulder_strength, mid_tone, highlight_strength);
        let g = self.filmic_curve(pixel.g / display.peak_luminance, shoulder_strength, mid_tone, highlight_strength);
        let b = self.filmic_curve(pixel.b / display.peak_luminance, shoulder_strength, mid_tone, highlight_strength);
        
        HdrPixel {
            r: r * display.peak_luminance,
            g: g * display.peak_luminance,
            b: b * display.peak_luminance,
        }
    }
    
    /// Filmic curve function
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
    
    /// ACES tone mapping
    fn aces_tone_map(&self, pixel: HdrPixel, display: &HdrDisplayProfile, config: &ToneMappingConfig) -> HdrPixel {
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
        
        HdrPixel {
            r: r_aces * display.peak_luminance,
            g: g_aces * display.peak_luminance,
            b: b_aces * display.peak_luminance,
        }
    }
    
    /// Hable tone mapping
    fn hable_tone_map(&self, pixel: HdrPixel, display: &HdrDisplayProfile, config: &ToneMappingConfig) -> HdrPixel {
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
        
        HdrPixel {
            r: r_hable * display.peak_luminance,
            g: g_hable * display.peak_luminance,
            b: b_hable * display.peak_luminance,
        }
    }
    
    /// Drago tone mapping
    fn drago_tone_map(&self, pixel: HdrPixel, display: &HdrDisplayProfile, config: &ToneMappingConfig) -> HdrPixel {
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
        
        HdrPixel {
            r: r_drago * display.peak_luminance,
            g: g_drago * display.peak_luminance,
            b: b_drago * display.peak_luminance,
        }
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: ToneMappingConfig) {
        self.config = config;
    }
    
    /// Get available algorithms
    pub fn get_available_algorithms(&self) -> Vec<ToneMappingAlgorithm> {
        vec![
            ToneMappingAlgorithm::Reinhard,
            ToneMappingAlgorithm::Filmic,
            ToneMappingAlgorithm::Aces,
            ToneMappingAlgorithm::Hable,
            ToneMappingAlgorithm::Drago,
        ]
    }
}

/// Gamut mapper implementation
pub struct GamutMapper {
    config: GamutMappingConfig,
}

impl GamutMapper {
    /// Create new gamut mapper
    pub fn new(config: GamutMappingConfig) -> Self {
        Self { config }
    }
    
    /// Apply gamut mapping
    pub fn apply_gamut_mapping(
        &self,
        hdr_image: &HdrImage,
        target_color_space: ColorSpace,
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
    
    /// ITP gamut mapping
    fn itp_gamut_map(&self, pixel: HdrPixel, config: &GamutMappingConfig) -> HdrPixel {
        // Convert to ITP color space
        let (l, m, s) = self.rgb_to_itp(pixel.r, pixel.g, pixel.b);
        
        // Apply gamut mapping in ITP space
        let (l_mapped, m_mapped, s_mapped) = self.map_itp_gamut(l, m, s, config);
        
        // Convert back to RGB
        let (r, g, b) = self.itp_to_rgb(l_mapped, m_mapped, s_mapped);
        
        HdrPixel { r, g, b }
    }
    
    /// Convert RGB to ITP
    fn rgb_to_itp(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        // Simplified ITP conversion
        let l = (r + g + b) / 3.0;
        let m = (r - g) / 2.0;
        let s = (r + g - 2.0 * b) / 6.0;
        
        (l, m, s)
    }
    
    /// Convert ITP to RGB
    fn itp_to_rgb(&self, l: f32, m: f32, s: f32) -> (f32, f32, f32) {
        let r = l + m + s;
        let g = l - m + s;
        let b = l - 2.0 * s;
        
        (r, g, b)
    }
    
    /// Map ITP gamut
    fn map_itp_gamut(&self, l: f32, m: f32, s: f32, config: &GamutMappingConfig) -> (f32, f32, f32) {
        let saturation_factor = config.saturation_preservation;
        
        let l_mapped = l;
        let m_mapped = m * saturation_factor;
        let s_mapped = s * saturation_factor;
        
        (l_mapped, m_mapped, s_mapped)
    }
    
    /// YUV gamut mapping
    fn yuv_gamut_map(&self, pixel: HdrPixel, config: &GamutMappingConfig) -> HdrPixel {
        // Convert to YUV
        let y = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
        let u = (pixel.b - y) * 0.5;
        let v = (pixel.r - y) * 0.5;
        
        // Apply gamut mapping
        let saturation_factor = config.saturation_preservation;
        let y_mapped = y;
        let u_mapped = u * saturation_factor;
        let v_mapped = v * saturation_factor;
        
        // Convert back to RGB
        let r = y_mapped + 1.403 * v_mapped;
        let g = y_mapped - 0.344 * u_mapped - 0.714 * v_mapped;
        let b = y_mapped + 1.770 * u_mapped;
        
        HdrPixel { r, g, b }
    }
    
    /// Saturation gamut mapping
    fn saturation_gamut_map(&self, pixel: HdrPixel, config: &GamutMappingConfig) -> HdrPixel {
        let luma = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
        let saturation_factor = config.saturation_preservation;
        
        let r = luma + (pixel.r - luma) * saturation_factor;
        let g = luma + (pixel.g - luma) * saturation_factor;
        let b = luma + (pixel.b - luma) * saturation_factor;
        
        HdrPixel { r, g, b }
    }
    
    /// Perceptual gamut mapping
    fn perceptual_gamut_map(&self, pixel: HdrPixel, config: &GamutMappingConfig) -> HdrPixel {
        // Advanced perceptual gamut mapping
        let luma = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
        let saturation_factor = config.saturation_preservation;
        
        // Perceptual weighting
        let r_weight = 0.299;
        let g_weight = 0.587;
        let b_weight = 0.114;
        
        let r_perceptual = luma + (pixel.r - luma) * saturation_factor * r_weight;
        let g_perceptual = luma + (pixel.g - luma) * saturation_factor * g_weight;
        let b_perceptual = luma + (pixel.b - luma) * saturation_factor * b_weight;
        
        HdrPixel {
            r: r_perceptual,
            g: g_perceptual,
            b: b_perceptual,
        }
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: GamutMappingConfig) {
        self.config = config;
    }
    
    /// Get available algorithms
    pub fn get_available_algorithms(&self) -> Vec<GamutMappingAlgorithm> {
        vec![
            GamutMappingAlgorithm::Itp,
            GamutMappingAlgorithm::Yuv,
            GamutMappingAlgorithm::Saturation,
            GamutMappingAlgorithm::Perceptual,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hdr_processor_creation() {
        let config = HdrConfig::default();
        let processor = HdrProcessor::new(config);
        assert!(processor.is_ok());
    }
    
    #[test]
    fn test_sdr_to_hdr_conversion() {
        let processor = HdrProcessor::default();
        
        // Create a simple SDR image
        let mut sdr_image = RgbImage::new(2, 2);
        sdr_image.put_pixel(0, 0, Rgb([255, 0, 0]));   // Red
        sdr_image.put_pixel(0, 1, Rgb([0, 255, 0]));   // Green
        sdr_image.put_pixel(1, 0, Rgb([0, 0, 255]));   // Blue
        sdr_image.put_pixel(1, 1, Rgb([128, 128, 128])); // Gray
        
        let hdr_image = processor.sdr_to_hdr(&sdr_image, 1000.0);
        assert!(hdr_image.is_ok());
        
        let hdr = hdr_image.unwrap();
        assert_eq!(hdr.width, 2);
        assert_eq!(hdr.height, 2);
        assert_eq!(hdr.data.len(), 4);
        assert_eq!(hdr.max_nits, 1000.0);
    }
    
    #[test]
    fn test_hdr_analysis() {
        let processor = HdrProcessor::default();
        
        // Create test HDR image
        let mut hdr_data = vec![HdrPixel::default(); 4];
        hdr_data[0] = HdrPixel { r: 1000.0, g: 0.0, b: 0.0 };     // Bright red
        hdr_data[1] = HdrPixel { r: 0.0, g: 500.0, b: 0.0 };      // Medium green
        hdr_data[2] = HdrPixel { r: 0.0, g: 0.0, b: 200.0 };      // Dark blue
        hdr_data[3] = HdrPixel { r: 100.0, g: 100.0, b: 100.0 };  // Dim gray
        
        let hdr_image = HdrImage {
            width: 2,
            height: 2,
            data: hdr_data,
            color_primaries: ColorPrimaries::Rec2020,
            transfer_function: TransferFunction::Pq,
            max_nits: 1000.0,
        };
        
        let analysis = processor.analyze_hdr_content(&hdr_image);
        assert!(analysis.is_ok());
        
        let result = analysis.unwrap();
        assert!(result.max_nits > 0.0);
        assert!(result.avg_nits > 0.0);
        assert!(result.dynamic_range > 1.0);
    }
    
    #[test]
    fn test_tone_mapping_algorithms() {
        let tone_mapper = ToneMapper::new(ToneMappingConfig::default());
        let algorithms = tone_mapper.get_available_algorithms();
        
        assert!(algorithms.contains(&ToneMappingAlgorithm::Reinhard));
        assert!(algorithms.contains(&ToneMappingAlgorithm::Filmic));
        assert!(algorithms.contains(&ToneMappingAlgorithm::Aces));
        assert!(algorithms.contains(&ToneMappingAlgorithm::Hable));
        assert!(algorithms.contains(&ToneMappingAlgorithm::Drago));
    }
    
    #[test]
    fn test_gamut_mapping_algorithms() {
        let gamut_mapper = GamutMapper::new(GamutMappingConfig::default());
        let algorithms = gamut_mapper.get_available_algorithms();
        
        assert!(algorithms.contains(&GamutMappingAlgorithm::Itp));
        assert!(algorithms.contains(&GamutMappingAlgorithm::Yuv));
        assert!(algorithms.contains(&GamutMappingAlgorithm::Saturation));
        assert!(algorithms.contains(&GamutMappingAlgorithm::Perceptual));
    }
    
    #[test]
    fn test_hdr_display_manager() {
        let config = HdrDisplayConfig::default();
        let manager = HdrDisplayManager::new(config);
        assert!(manager.is_ok());
        
        let display_manager = manager.unwrap();
        let displays = display_manager.get_available_displays();
        
        assert!(displays.contains(&HdrDisplayType::Sdr));
        assert!(displays.contains(&HdrDisplayType::Hdr10));
        assert!(displays.contains(&HdrDisplayType::DolbyVision));
    }
}
