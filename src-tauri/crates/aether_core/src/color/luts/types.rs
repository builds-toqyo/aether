

#[derive(Debug, Clone)]
pub struct LutData {
    pub name: String,
    pub size: u32,
    pub format: LutFormat,
    pub data: Vec<[f32; 3]>,
}

impl LutData {

    pub fn generate_identity_lut(&mut self, size: u32) {
        self.data.clear();

        for b in 0..size {
            for g in 0..size {
                for r in 0..size {
                    let rf = r as f32 / (size - 1) as f32;
                    let gf = g as f32 / (size - 1) as f32;
                    let bf = b as f32 / (size - 1) as f32;

                    self.data.push([rf, gf, bf]);
                }
            }
        }
    }


    pub fn new(name: String, size: u32, format: LutFormat) -> Self {
        Self {
            name,
            size,
            format,
            data: Vec::new(),
        }
    }


    pub fn dimensions(&self) -> (u32, u32, u32) {
        (self.size, self.size, self.size)
    }


    pub fn total_points(&self) -> usize {
        self.data.len()
    }


    pub fn validate(&self) -> Result<(), String> {
        let expected_size = self.size * self.size * self.size;

        if self.data.len() != expected_size as usize {
            return Err(format!("Invalid data size: expected {}, got {}", expected_size, self.data.len()));
        }


        for (i, rgb) in self.data.iter().enumerate() {
            for (c, &value) in rgb.iter().enumerate() {
                if value < 0.0 || value > 1.0 {
                    return Err(format!("Invalid value at index {}, channel {}: {}", i, c, value));
                }
            }
        }

        Ok(())
    }


    pub fn get_value(&self, r: u32, g: u32, b: u32) -> Option<[f32; 3]> {
        if r >= self.size || g >= self.size || b >= self.size {
            return None;
        }

        let index = ((b * self.size + g) * self.size + r) as usize;
        self.data.get(index).copied()
    }


    pub fn set_value(&mut self, r: u32, g: u32, b: u32, value: [f32; 3]) -> Result<(), String> {
        if r >= self.size || g >= self.size || b >= self.size {
            return Err("Coordinates out of bounds".to_string());
        }

        let index = ((b * self.size + g) * self.size + r) as usize;

        if index >= self.data.len() {
            return Err("Index out of bounds".to_string());
        }

        self.data[index] = value;
        Ok(())
    }
}

impl Default for LutData {
    fn default() -> Self {
        Self {
            name: String::new(),
            size: 33,
            format: LutFormat::Cube,
            data: Vec::new(),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LutFormat {
    Cube,
    ThreeDL,
    Look,
}

impl LutFormat {

    pub fn extension(&self) -> &'static str {
        match self {
            LutFormat::Cube => ".cube",
            LutFormat::ThreeDL => ".3dl",
            LutFormat::Look => ".look",
        }
    }

    /// Get format description
    pub fn description(&self) -> &'static str {
        match self {
            LutFormat::Cube => "Resolve Cube format - Industry standard 3D LUT format",
            LutFormat::ThreeDL => "3DL format - Autodesk 3D LUT format",
            LutFormat::Look => "LOOK format - Adobe SpeedGrade LUT format",
        }
    }


    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "cube" => Some(LutFormat::Cube),
            "3dl" => Some(LutFormat::ThreeDL),
            "look" => Some(LutFormat::Look),
            _ => None,
        }
    }
}

impl Default for LutFormat {
    fn default() -> Self {
        LutFormat::Cube
    }
}


#[derive(Debug, Clone)]
pub struct LutConfig {
    pub interpolation_quality: InterpolationQuality,
    pub color_space: crate::types::ColorSpace,
    pub clamp_output: bool,
}

impl LutConfig {

    pub fn new() -> Self {
        Self::default()
    }


    pub fn with_interpolation_quality(mut self, quality: InterpolationQuality) -> Self {
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
}

impl Default for LutConfig {
    fn default() -> Self {
        Self {
            interpolation_quality: InterpolationQuality::Trilinear,
            color_space: crate::types::ColorSpace::Rec709,
            clamp_output: true,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationQuality {
    Nearest,
    Linear,
    Trilinear,
}

impl InterpolationQuality {

    pub fn description(&self) -> &'static str {
        match self {
            InterpolationQuality::Nearest => "Nearest neighbor - Fast but low quality",
            InterpolationQuality::Linear => "Linear interpolation - Good balance of speed and quality",
            InterpolationQuality::Trilinear => "Trilinear interpolation - Highest quality",
        }
    }


    pub fn computational_cost(&self) -> f32 {
        match self {
            InterpolationQuality::Nearest => 1.0,
            InterpolationQuality::Linear => 2.0,
            InterpolationQuality::Trilinear => 3.0,
        }
    }
}

impl Default for InterpolationQuality {
    fn default() -> Self {
        InterpolationQuality::Trilinear
    }
}


#[derive(Debug, Clone)]
pub struct ColorCorrection {
    pub gamma: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub color_balance: [f32; 3],
}

impl ColorCorrection {

    pub fn new() -> Self {
        Self::default()
    }


    pub fn with_gamma(mut self, gamma: f32) -> Self {
        self.gamma = gamma;
        self
    }


    pub fn with_contrast(mut self, contrast: f32) -> Self {
        self.contrast = contrast;
        self
    }


    pub fn with_saturation(mut self, saturation: f32) -> Self {
        self.saturation = saturation;
        self
    }


    pub fn with_color_balance(mut self, balance: [f32; 3]) -> Self {
        self.color_balance = balance;
        self
    }


    pub fn validate(&self) -> Result<(), String> {
        if self.gamma <= 0.0 {
            return Err("Gamma must be positive".to_string());
        }

        if self.contrast < 0.0 {
            return Err("Contrast cannot be negative".to_string());
        }

        if self.saturation < 0.0 {
            return Err("Saturation cannot be negative".to_string());
        }

        for (i, &value) in self.color_balance.iter().enumerate() {
            if value < 0.0 {
                return Err(format!("Color balance channel {} cannot be negative", i));
            }
        }

        Ok(())
    }


    pub fn apply(&self, rgb: [f32; 3]) -> [f32; 3] {
        let [r, g, b] = rgb;


        let r_gamma = r.powf(self.gamma);
        let g_gamma = g.powf(self.gamma);
        let b_gamma = b.powf(self.gamma);


        let r_contrast = ((r_gamma - 0.5) * self.contrast + 0.5).clamp(0.0, 1.0);
        let g_contrast = ((g_gamma - 0.5) * self.contrast + 0.5).clamp(0.0, 1.0);
        let b_contrast = ((b_gamma - 0.5) * self.contrast + 0.5).clamp(0.0, 1.0);


        let luma = 0.2126 * r_contrast + 0.7152 * g_contrast + 0.0722 * b_contrast;
        let r_saturated = luma + (r_contrast - luma) * self.saturation;
        let g_saturated = luma + (g_contrast - luma) * self.saturation;
        let b_saturated = luma + (b_contrast - luma) * self.saturation;


        let r_balanced = r_saturated * self.color_balance[0];
        let g_balanced = g_saturated * self.color_balance[1];
        let b_balanced = g_saturated * self.color_balance[2];

        [r_balanced, g_balanced, b_balanced]
    }
}

impl Default for ColorCorrection {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            color_balance: [1.0, 1.0, 1.0],
        }
    }
}


#[derive(Debug, Clone)]
pub struct LutInfo {
    pub name: String,
    pub size: u32,
    pub format: LutFormat,
    pub data_points: usize,
}

impl LutInfo {

    pub fn memory_usage(&self) -> usize {
        self.data_points * 3 * std::mem::size_of::<f32>()
    }


    pub fn memory_usage_string(&self) -> String {
        let bytes = self.memory_usage();

        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f32 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f32 / (1024.0 * 1024.0))
        }
    }


    pub fn is_valid(&self) -> bool {
        self.data_points == (self.size * self.size * self.size) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lut_data_creation() {
        let mut lut_data = LutData::new("test".to_string(), 33, LutFormat::Cube);

        assert_eq!(lut_data.name, "test");
        assert_eq!(lut_data.size, 33);
        assert_eq!(lut_data.format, LutFormat::Cube);
        assert_eq!(lut_data.data.len(), 0);

        lut_data.generate_identity_lut(33);
        assert_eq!(lut_data.data.len(), 33 * 33 * 33);

        assert!(lut_data.validate().is_ok());
    }

    #[test]
    fn test_lut_data_access() {
        let mut lut_data = LutData::new("test".to_string(), 3, LutFormat::Cube);
        lut_data.generate_identity_lut(3);


        let value = lut_data.get_value(0, 0, 0);
        assert!(value.is_some());
        assert_eq!(value.unwrap(), [0.0, 0.0, 0.0]);

        let value = lut_data.get_value(2, 2, 2);
        assert!(value.is_some());
        assert_eq!(value.unwrap(), [1.0, 1.0, 1.0]);


        let result = lut_data.set_value(1, 1, 1, [0.5, 0.5, 0.5]);
        assert!(result.is_ok());

        let value = lut_data.get_value(1, 1, 1);
        assert!(value.is_some());
        assert_eq!(value.unwrap(), [0.5, 0.5, 0.5]);


        let value = lut_data.get_value(3, 0, 0);
        assert!(value.is_none());

        let result = lut_data.set_value(3, 0, 0, [0.0, 0.0, 0.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_lut_format() {
        assert_eq!(LutFormat::Cube.extension(), "cube");
        assert_eq!(LutFormat::ThreeDL.extension(), "3dl");
        assert_eq!(LutFormat::Look.extension(), "look");

        assert_eq!(LutFormat::from_extension("cube"), Some(LutFormat::Cube));
        assert_eq!(LutFormat::from_extension("3dl"), Some(LutFormat::ThreeDL));
        assert_eq!(LutFormat::from_extension("look"), Some(LutFormat::Look));
        assert_eq!(LutFormat::from_extension("invalid"), None);
    }

    #[test]
    fn test_color_correction() {
        let correction = ColorCorrection::new()
            .with_gamma(2.2)
            .with_contrast(1.2)
            .with_saturation(1.1)
            .with_color_balance([1.0, 0.9, 0.8]);

        assert!(correction.validate().is_ok());

        let input = [0.5, 0.5, 0.5];
        let output = correction.apply(input);


        assert_ne!(input, output);


        for &value in &output {
            assert!(value >= 0.0);
            assert!(value <= 1.0);
        }
    }

    #[test]
    fn test_lut_info() {
        let info = LutInfo {
            name: "test".to_string(),
            size: 33,
            format: LutFormat::Cube,
            data_points: 33 * 33 * 33,
        };

        assert!(info.is_valid());

        let memory = info.memory_usage();
        assert!(memory > 0);

        let memory_str = info.memory_usage_string();
        assert!(!memory_str.is_empty());
        assert!(memory_str.contains("KB") || memory_str.contains("MB"));
    }

    #[test]
    fn test_interpolation_quality() {
        assert_eq!(InterpolationQuality::Nearest.computational_cost(), 1.0);
        assert_eq!(InterpolationQuality::Linear.computational_cost(), 2.0);
        assert_eq!(InterpolationQuality::Trilinear.computational_cost(), 3.0);

        assert!(!InterpolationQuality::Nearest.description().is_empty());
        assert!(!InterpolationQuality::Linear.description().is_empty());
        assert!(!InterpolationQuality::Trilinear.description().is_empty());
    }
}
