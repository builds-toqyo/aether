use std::collections::HashMap;
use anyhow::{Result, anyhow};
use log::debug;

#[derive(Debug, Clone)]
pub struct LookTransform {
    name: String,
    saturation: f32,
    contrast: f32,
    pivot: f32,
    color_matrix: [[f32; 3]; 3],
}

impl LookTransform {
    pub fn new(name: String) -> Self {
        Self {
            name,
            saturation: 1.0,
            contrast: 1.0,
            pivot: 0.18,
            color_matrix: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn apply(&self, rgb: [u8; 3]) -> [u8; 3] {
        let rf = rgb[0] as f32 / 65535.0;
        let gf = rgb[1] as f32 / 65535.0;
        let bf = rgb[2] as f32 / 65535.0;

        let r = self.color_matrix[0][0] * rf + self.color_matrix[0][1] * gf + self.color_matrix[0][2] * bf;
        let g = self.color_matrix[1][0] * rf + self.color_matrix[1][1] * gf + self.color_matrix[1][2] * bf;
        let b = self.color_matrix[2][0] * rf + self.color_matrix[2][1] * gf + self.color_matrix[2][2] * bf;

        let r = (r - self.pivot) * self.contrast + self.pivot;
        let g = (g - self.pivot) * self.contrast + self.pivot;
        let b = (b - self.pivot) * self.contrast + self.pivot;

        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let r = luma + (r - luma) * self.saturation;
        let g = luma + (g - luma) * self.saturation;
        let b = luma + (b - luma) * self.saturation;

        [
            (r * 65535.0).clamp(0.0, 65535.0) as u8,
            (g * 65535.0).clamp(0.0, 65535.0) as u8,
            (b * 65535.0).clamp(0.0, 65535.0) as u8,
        ]
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new_neutral() -> Self {
        Self::new("neutral".to_string())
    }

    pub fn new_teal_orange() -> Self {
        let mut look = Self::new("teal_orange".to_string());
        look.color_matrix = [
            [1.1, -0.05, -0.05],
            [-0.05, 1.0, -0.05],
            [-0.05, -0.05, 0.9],
        ];
        look.saturation = 1.2;
        look.contrast = 1.1;
        look
    }

    pub fn new_vintage() -> Self {
        let mut look = Self::new("vintage".to_string());
        look.color_matrix = [
            [1.2, 0.1, 0.0],
            [0.0, 0.9, 0.1],
            [0.0, 0.0, 0.8],
        ];
        look.saturation = 0.8;
        look.contrast = 0.9;
        look
    }

    pub fn new_dramatic() -> Self {
        let mut look = Self::new("dramatic".to_string());
        look.color_matrix = [
            [1.3, -0.1, -0.1],
            [-0.1, 1.2, -0.1],
            [-0.1, -0.1, 1.1],
        ];
        look.saturation = 1.3;
        look.contrast = 1.2;
        look
    }
}

pub struct LookManager {
    looks: HashMap<String, LookTransform>,
}

impl LookManager {

    pub fn new() -> Self {
        Self {
            looks: HashMap::new(),
        }
    }

    pub fn initialize_default_looks(&mut self) {
        debug!("Initializing default looks");

        self.looks.insert("neutral".to_string(), LookTransform::new_neutral());
        self.looks.insert("teal_orange".to_string(), LookTransform::new_teal_orange());
        self.looks.insert("vintage".to_string(), LookTransform::new_vintage());
        self.looks.insert("dramatic".to_string(), LookTransform::new_dramatic());
    }

    pub fn get_look(&self, look_name: &str) -> Result<&LookTransform> {
        self.looks.get(look_name)
            .ok_or_else(|| anyhow!("Look not found: {}", look_name))
    }

    pub fn add_look(&mut self, look: LookTransform) {
        let name = look.name().to_string();
        self.looks.insert(name, look);
    }

    pub fn remove_look(&mut self, look_name: &str) -> Result<()> {
        self.looks.remove(look_name)
            .ok_or_else(|| anyhow!("Look not found: {}", look_name))?;
        Ok(())
    }

    pub fn get_available_looks(&self) -> Vec<String> {
        self.looks.keys().cloned().collect()
    }

    pub fn list_looks(&self) -> Vec<&str> {
        self.looks.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for LookManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_look_transforms() {
        let look = LookTransform::new_teal_orange();
        let rgb = [32768, 32768, 32768];

        let result = look.apply(rgb);
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|&v| v <= 65535));
    }

    #[test]
    fn test_look_manager() {
        let mut manager = LookManager::new();
        manager.initialize_default_looks();

        let looks = manager.get_available_looks();
        assert!(looks.contains(&"neutral".to_string()));
        assert!(looks.contains(&"teal_orange".to_string()));

        let look = manager.get_look("neutral");
        assert!(look.is_ok());
    }
}
