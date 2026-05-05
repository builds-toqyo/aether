//! Gamma correction functions for ACES color pipeline
//! 
//! This module provides gamma encode/decode functions for different
//! color spaces used in the ACES color pipeline.

/// Gamma functions for different color spaces
pub mod gamma {
    /// Rec709 gamma decode (linearize)
    pub fn rec709_gamma_decode(value: f32) -> f32 {
        if value < 0.0812 {
            value / 4.5
        } else {
            ((value + 0.099) / 1.099).powf(1.0 / 0.45)
        }
    }
    
    /// Rec709 gamma encode
    pub fn rec709_gamma_encode(value: f32) -> f32 {
        if value < 0.0181 {
            value * 4.5
        } else {
            1.099 * value.powf(0.45) - 0.099
        }
    }
    
    /// Rec2020 gamma decode (linearize)
    pub fn rec2020_gamma_decode(value: f32) -> f32 {
        if value < 0.0812 {
            value / 4.5
        } else {
            ((value + 0.099) / 1.099).powf(1.0 / 0.45)
        }
    }
    
    /// Rec2020 gamma encode
    pub fn rec2020_gamma_encode(value: f32) -> f32 {
        if value < 0.0181 {
            value * 4.5
        } else {
            1.099 * value.powf(0.45) - 0.099
        }
    }
    
    /// sRGB gamma decode (linearize)
    pub fn srgb_gamma_decode(value: f32) -> f32 {
        if value < 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
    
    /// sRGB gamma encode
    pub fn srgb_gamma_encode(value: f32) -> f32 {
        if value < 0.0031308 {
            value * 12.92
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        }
    }
}

#[cfg(test)]
mod tests {
    use super::gamma::*;
    
    #[test]
    fn test_gamma_roundtrip() {
        // Test gamma encode/decode roundtrip for sRGB
        let original = 0.5;
        let decoded = srgb_gamma_decode(original);
        let encoded = srgb_gamma_encode(decoded);
        
        assert!((original - encoded).abs() < 0.01);
    }
    
    #[test]
    fn test_rec709_gamma() {
        // Test Rec709 gamma functions
        let linear = 0.18; // Middle gray
        let encoded = rec709_gamma_encode(linear);
        let decoded = rec709_gamma_decode(encoded);
        
        assert!((linear - decoded).abs() < 0.01);
    }
    
    #[test]
    fn test_rec2020_gamma() {
        // Test Rec2020 gamma functions
        let linear = 0.18; // Middle gray
        let encoded = rec2020_gamma_encode(linear);
        let decoded = rec2020_gamma_decode(encoded);
        
        assert!((linear - decoded).abs() < 0.01);
    }
}
