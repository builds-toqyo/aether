//! ACES color transformation management
//! 
//! This module handles input and output color space transformations
//! for the ACES color pipeline.

use std::collections::HashMap;
use anyhow::{Result, anyhow};

/// Input transform types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputTransform {
    Rec709ToAces,
    Rec2020ToAces,
    SrgbToAces,
    RawToAces,
}

/// Output transform types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputTransform {
    AcesToRec709,
    AcesToRec2020,
    AcesToSrgb,
    AcesToHdr10,
}

/// Manager for color space transformations
pub struct TransformManager {
    input_transforms: HashMap<InputTransform, [[f32; 3]; 3]>,
    output_transforms: HashMap<OutputTransform, [[f32; 3]; 3]>,
}

impl TransformManager {
    /// Create new transform manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            input_transforms: HashMap::new(),
            output_transforms: HashMap::new(),
        })
    }
    
    /// Initialize all transformation matrices
    pub fn initialize_transforms(&mut self) -> Result<()> {
        debug!("Initializing transformation matrices");
        
        // Input transforms
        self.input_transforms.insert(InputTransform::Rec709ToAces, Self::rec709_to_aces_matrix());
        self.input_transforms.insert(InputTransform::Rec2020ToAces, Self::rec2020_to_aces_matrix());
        self.input_transforms.insert(InputTransform::SrgbToAces, Self::srgb_to_aces_matrix());
        self.input_transforms.insert(InputTransform::RawToAces, Self::raw_to_aces_matrix());
        
        // Output transforms
        self.output_transforms.insert(OutputTransform::AcesToRec709, Self::aces_to_rec709_matrix());
        self.output_transforms.insert(OutputTransform::AcesToRec2020, Self::aces_to_rec2020_matrix());
        self.output_transforms.insert(OutputTransform::AcesToSrgb, Self::aces_to_srgb_matrix());
        self.output_transforms.insert(OutputTransform::AcesToHdr10, Self::aces_to_hdr10_matrix());
        
        Ok(())
    }
    
    /// Apply input transformation
    pub fn apply_input_transform(&self, rgb: [u8; 3], transform: InputTransform) -> Result<[u8; 3]> {
        let matrix = self.input_transforms.get(&transform)
            .ok_or_else(|| anyhow!("Input transform not found: {:?}", transform))?;
        
        Ok(self.apply_matrix(rgb, *matrix))
    }
    
    /// Apply output transformation
    pub fn apply_output_transform(&self, rgb: [u8; 3], transform: OutputTransform) -> Result<[u8; 3]> {
        let matrix = self.output_transforms.get(&transform)
            .ok_or_else(|| anyhow!("Output transform not found: {:?}", transform))?;
        
        Ok(self.apply_matrix(rgb, *matrix))
    }
    
    /// Apply transformation matrix to RGB values
    fn apply_matrix(&self, rgb: [u8; 3], matrix: [[f32; 3]; 3]) -> [u8; 3] {
        let rf = rgb[0] as f32 / 255.0;
        let gf = rgb[1] as f32 / 255.0;
        let bf = rgb[2] as f32 / 255.0;
        
        let r = matrix[0][0] * rf + matrix[0][1] * gf + matrix[0][2] * bf;
        let g = matrix[1][0] * rf + matrix[1][1] * gf + matrix[1][2] * bf;
        let b = matrix[2][0] * rf + matrix[2][1] * gf + matrix[2][2] * bf;
        
        [
            (r * 65535.0).clamp(0.0, 65535.0) as u8, // ACES uses 16-bit
            (g * 65535.0).clamp(0.0, 65535.0) as u8,
            (b * 65535.0).clamp(0.0, 65535.0) as u8,
        ]
    }
    
    /// Get available input transforms
    pub fn get_available_input_transforms(&self) -> Vec<InputTransform> {
        self.input_transforms.keys().copied().collect()
    }
    
    /// Get available output transforms
    pub fn get_available_output_transforms(&self) -> Vec<OutputTransform> {
        self.output_transforms.keys().copied().collect()
    }
    
    // Color space transformation matrices
    fn rec709_to_aces_matrix() -> [[f32; 3]; 3] {
        [
            [0.439632, 0.382975, 0.177393],
            [0.089788, 0.813423, 0.096789],
            [0.017544, 0.111544, 0.870912],
        ]
    }
    
    fn rec2020_to_aces_matrix() -> [[f32; 3]; 3] {
        [
            [0.627404, 0.329283, 0.043313],
            [0.069097, 0.919540, 0.011363],
            [0.016391, 0.087013, 0.896596],
        ]
    }
    
    fn srgb_to_aces_matrix() -> [[f32; 3]; 3] {
        [
            [0.439632, 0.382975, 0.177393],
            [0.089788, 0.813423, 0.096789],
            [0.017544, 0.111544, 0.870912],
        ]
    }
    
    fn raw_to_aces_matrix() -> [[f32; 3]; 3] {
        // Identity matrix for raw data (assumed to be linear)
        [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
    }
    
    fn aces_to_rec709_matrix() -> [[f32; 3]; 3] {
        [
            [1.704748, -0.198989, -0.505759],
            [-0.262351, 1.078549, 0.183802],
            [0.023711, -0.242931, 1.219220],
        ]
    }
    
    fn aces_to_rec2020_matrix() -> [[f32; 3]; 3] {
        [
            [1.641023, -0.324803, -0.316216],
            [-0.418194, 1.279050, 0.139144],
            [-0.016279, -0.042748, 1.059027],
        ]
    }
    
    fn aces_to_srgb_matrix() -> [[f32; 3]; 3] {
        [
            [2.521391, -0.275201, -0.246190],
            [-0.699826, 1.789077, -0.089251],
            [0.045584, -0.324636, 1.279052],
        ]
    }
    
    fn aces_to_hdr10_matrix() -> [[f32; 3]; 3] {
        [
            [1.344361, -0.154775, -0.189586],
            [-0.354425, 1.204946, 0.149479],
            [-0.016279, -0.042748, 1.059027],
        ]
    }
}

impl Default for TransformManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
