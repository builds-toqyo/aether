//! LUT file saver
//! 
//! This module provides LUT saving functionality for different file formats
//! including Cube, 3DL, and LOOK formats.

use std::io::Write;
use anyhow::{Result, anyhow};
use log::debug;

use super::types::{LutData, LutFormat};

/// LUT file saver
pub struct LutSaver {
    config: crate::color::luts::config::LutConfig,
}

impl LutSaver {
    /// Create new LUT saver
    pub fn new() -> Self {
        Self {
            config: crate::color::luts::config::LutConfig::default(),
        }
    }
    
    /// Save LUT to file
    pub fn save_lut(&self, lut_data: &LutData, file_path: &std::path::Path, format: LutFormat) -> Result<()> {
        debug!("Saving LUT '{}' to: {:?} ({:?})", lut_data.name, file_path, format);
        
        match format {
            LutFormat::Cube => self.save_cube_file(lut_data, file_path)?,
            LutFormat::ThreeDL => self.save_3dl_file(lut_data, file_path)?,
            LutFormat::Look => self.save_look_file(lut_data, file_path)?,
        }
        
        debug!("LUT saved successfully");
        
        Ok(())
    }
    
    /// Save as Cube format
    fn save_cube_file(&self, lut_data: &LutData, file_path: &std::path::Path) -> Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        
        writeln!(file, "TITLE {}", lut_data.name)?;
        writeln!(file, "LUT_3D_SIZE {}", lut_data.size)?;
        writeln!(file)?;
        
        for rgb in &lut_data.data {
            writeln!(file, "{} {} {}", rgb[0], rgb[1], rgb[2])?;
        }
        
        Ok(())
    }
    
    /// Save as 3DL format
    fn save_3dl_file(&self, lut_data: &LutData, file_path: &std::path::Path) -> Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        
        let mut index = 0;
        for b in 0..lut_data.size {
            for g in 0..lut_data.size {
                for r in 0..lut_data.size {
                    if index < lut_data.data.len() {
                        let rgb = &lut_data.data[index];
                        let r_in = r as f32 / (lut_data.size - 1) as f32;
                        let g_in = g as f32 / (lut_data.size - 1) as f32;
                        let b_in = b as f32 / (lut_data.size - 1) as f32;
                        
                        writeln!(file, "{} {} {} {} {} {}", 
                            r_in, g_in, b_in, rgb[0], rgb[1], rgb[2])?;
                        index += 1;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Save as LOOK format
    fn save_look_file(&self, lut_data: &LutData, file_path: &std::path::Path) -> Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        
        writeln!(file, "<Look>")?;
        writeln!(file, "  <Name>{}</Name>", lut_data.name)?;
        writeln!(file, "  <Size>{}</Size>", lut_data.size)?;
        writeln!(file, "  <Data>")?;
        
        for rgb in &lut_data.data {
            writeln!(file, "    {} {} {}", rgb[0], rgb[1], rgb[2])?;
        }
        
        writeln!(file, "  </Data>")?;
        writeln!(file, "</Look>")?;
        
        Ok(())
    }
    
    /// Save LUT to memory
    pub fn save_lut_to_memory(&self, lut_data: &LutData, format: LutFormat) -> Result<Vec<u8>> {
        debug!("Saving LUT '{}' to memory ({:?})", lut_data.name, format);
        
        let content = match format {
            LutFormat::Cube => self.save_cube_to_string(lut_data)?,
            LutFormat::ThreeDL => self.save_3dl_to_string(lut_data)?,
            LutFormat::Look => self.save_look_to_string(lut_data)?,
        };
        
        Ok(content.into_bytes())
    }
    
    /// Save Cube format to string
    fn save_cube_to_string(&self, lut_data: &LutData) -> Result<String> {
        let mut content = String::new();
        
        content.push_str(&format!("TITLE {}\n", lut_data.name));
        content.push_str(&format!("LUT_3D_SIZE {}\n\n", lut_data.size));
        
        for rgb in &lut_data.data {
            content.push_str(&format!("{} {} {}\n", rgb[0], rgb[1], rgb[2]));
        }
        
        Ok(content)
    }
    
    /// Save 3DL format to string
    fn save_3dl_to_string(&self, lut_data: &LutData) -> Result<String> {
        let mut content = String::new();
        
        let mut index = 0;
        for b in 0..lut_data.size {
            for g in 0..lut_data.size {
                for r in 0..lut_data.size {
                    if index < lut_data.data.len() {
                        let rgb = &lut_data.data[index];
                        let r_in = r as f32 / (lut_data.size - 1) as f32;
                        let g_in = g as f32 / (lut_data.size - 1) as f32;
                        let b_in = b as f32 / (lut_data.size - 1) as f32;
                        
                        content.push_str(&format!("{} {} {} {} {} {}\n", 
                            r_in, g_in, b_in, rgb[0], rgb[1], rgb[2]));
                        index += 1;
                    }
                }
            }
        }
        
        Ok(content)
    }
    
    /// Save LOOK format to string
    fn save_look_to_string(&self, lut_data: &LutData) -> Result<String> {
        let mut content = String::new();
        
        content.push_str("<Look>\n");
        content.push_str(&format!("  <Name>{}</Name>\n", lut_data.name));
        content.push_str(&format!("  <Size>{}</Size>\n", lut_data.size));
        content.push_str("  <Data>\n");
        
        for rgb in &lut_data.data {
            content.push_str(&format!("    {} {} {}\n", rgb[0], rgb[1], rgb[2]));
        }
        
        content.push_str("  </Data>\n");
        content.push_str("</Look>\n");
        
        Ok(content)
    }
    
    /// Export LUT with metadata
    pub fn export_lut_with_metadata(&self, lut_data: &LutData, file_path: &std::path::Path, format: LutFormat, metadata: &LutMetadata) -> Result<()> {
        debug!("Exporting LUT with metadata: {} -> {:?}", lut_data.name, file_path);
        
        match format {
            LutFormat::Cube => self.export_cube_with_metadata(lut_data, file_path, metadata)?,
            LutFormat::ThreeDL => self.export_3dl_with_metadata(lut_data, file_path, metadata)?,
            LutFormat::Look => self.export_look_with_metadata(lut_data, file_path, metadata)?,
        }
        
        debug!("LUT exported with metadata successfully");
        
        Ok(())
    }
    
    /// Export Cube format with metadata
    fn export_cube_with_metadata(&self, lut_data: &LutData, file_path: &std::path::Path, metadata: &LutMetadata) -> Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        
        // Write header with metadata
        writeln!(file, "# LUT exported by Aether")?;
        writeln!(file, "# Created: {}", metadata.created_at)?;
        if !metadata.author.is_empty() {
            writeln!(file, "# Author: {}", metadata.author)?;
        }
        if !metadata.description.is_empty() {
            writeln!(file, "# Description: {}", metadata.description)?;
        }
        writeln!(file, "# Input Color Space: {:?}", metadata.input_color_space)?;
        writeln!(file, "# Output Color Space: {:?}", metadata.output_color_space)?;
        writeln!(file)?;
        
        // Write LUT data
        writeln!(file, "TITLE {}", lut_data.name)?;
        writeln!(file, "LUT_3D_SIZE {}", lut_data.size)?;
        writeln!(file)?;
        
        for rgb in &lut_data.data {
            writeln!(file, "{} {} {}", rgb[0], rgb[1], rgb[2])?;
        }
        
        Ok(())
    }
    
    /// Export 3DL format with metadata
    fn export_3dl_with_metadata(&self, lut_data: &LutData, file_path: &std::path::Path, metadata: &LutMetadata) -> Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        
        // Write header with metadata
        writeln!(file, "# 3DL LUT exported by Aether")?;
        writeln!(file, "# Created: {}", metadata.created_at)?;
        if !metadata.author.is_empty() {
            writeln!(file, "# Author: {}", metadata.author)?;
        }
        writeln!(file)?;
        
        // Write LUT data
        let mut index = 0;
        for b in 0..lut_data.size {
            for g in 0..lut_data.size {
                for r in 0..lut_data.size {
                    if index < lut_data.data.len() {
                        let rgb = &lut_data.data[index];
                        let r_in = r as f32 / (lut_data.size - 1) as f32;
                        let g_in = g as f32 / (lut_data.size - 1) as f32;
                        let b_in = b as f32 / (lut_data.size - 1) as f32;
                        
                        writeln!(file, "{} {} {} {} {} {}", 
                            r_in, g_in, b_in, rgb[0], rgb[1], rgb[2])?;
                        index += 1;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Export LOOK format with metadata
    fn export_look_with_metadata(&self, lut_data: &LutData, file_path: &std::path::Path, metadata: &LutMetadata) -> Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        
        // Write LOOK format with metadata in comments
        writeln!(file, "<Look>")?;
        writeln!(file, "  <Name>{}</Name>", lut_data.name)?;
        writeln!(file, "  <Size>{}</Size>", lut_data.size)?;
        
        // Add metadata as comments
        writeln!(file, "  <!--")?;
        writeln!(file, "    Exported by Aether")?;
        writeln!(file, "    Created: {}", metadata.created_at)?;
        if !metadata.author.is_empty() {
            writeln!(file, "    Author: {}", metadata.author)?;
        }
        if !metadata.description.is_empty() {
            writeln!(file, "    Description: {}", metadata.description)?;
        }
        writeln!(file, "    Input Color Space: {:?}", metadata.input_color_space)?;
        writeln!(file, "    Output Color Space: {:?}", metadata.output_color_space)?;
        writeln!(file, "  -->")?;
        
        writeln!(file, "  <Data>")?;
        
        for rgb in &lut_data.data {
            writeln!(file, "    {} {} {}", rgb[0], rgb[1], rgb[2])?;
        }
        
        writeln!(file, "  </Data>")?;
        writeln!(file, "</Look>")?;
        
        Ok(())
    }
    
    /// Batch export LUTs
    pub fn batch_export(&self, luts: &[(&LutData, &str)], output_dir: &std::path::Path, format: LutFormat) -> Result<Vec<std::path::PathBuf>> {
        debug!("Batch exporting {} LUTs to {:?} ({:?})", luts.len(), output_dir, format);
        
        std::fs::create_dir_all(output_dir)?;
        
        let mut exported_files = Vec::new();
        
        for (lut_data, filename) in luts {
            let file_path = output_dir.join(format!("{}.{}", filename, format.extension()));
            self.save_lut(lut_data, &file_path, format)?;
            exported_files.push(file_path);
        }
        
        debug!("Batch export completed: {} files exported", exported_files.len());
        
        Ok(exported_files)
    }
}

impl Default for LutSaver {
    fn default() -> Self {
        Self::new()
    }
}

/// LUT metadata for export
#[derive(Debug, Clone)]
pub struct LutMetadata {
    pub created_at: String,
    pub author: String,
    pub description: String,
    pub input_color_space: crate::types::ColorSpace,
    pub output_color_space: crate::types::ColorSpace,
}

impl LutMetadata {
    /// Create new metadata
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set author
    pub fn with_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }
    
    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
    
    /// Set color spaces
    pub fn with_color_spaces(mut self, input: crate::types::ColorSpace, output: crate::types::ColorSpace) -> Self {
        self.input_color_space = input;
        self.output_color_space = output;
        self
    }
}

impl Default for LutMetadata {
    fn default() -> Self {
        Self {
            created_at: chrono::Utc::now().to_rfc3339(),
            author: String::new(),
            description: String::new(),
            input_color_space: crate::types::ColorSpace::Rec709,
            output_color_space: crate::types::ColorSpace::Rec709,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_cube_file_saving() {
        let mut lut_data = LutData::new("Test LUT".to_string(), 2, LutFormat::Cube);
        lut_data.data = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ];
        
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.cube");
        
        let saver = LutSaver::new();
        saver.save_lut(&lut_data, &file_path, LutFormat::Cube).unwrap();
        
        assert!(file_path.exists());
        
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("TITLE Test LUT"));
        assert!(content.contains("LUT_3D_SIZE 2"));
    }
    
    #[test]
    fn test_3dl_file_saving() {
        let mut lut_data = LutData::new("Test 3DL".to_string(), 2, LutFormat::ThreeDL);
        lut_data.data = vec![
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.5, 0.5, 0.5],
            [0.25, 0.75, 0.5],
        ];
        
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.3dl");
        
        let saver = LutSaver::new();
        saver.save_lut(&lut_data, &file_path, LutFormat::ThreeDL).unwrap();
        
        assert!(file_path.exists());
        
        let content = std::fs::read_to_string(&file_path).unwrap();
        // Check that content has 3DL format (input/output pairs)
        let lines: Vec<&str> = content.lines().collect();
        assert!(lines.len() >= 4);
        for line in lines.iter().take(4) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            assert_eq!(parts.len(), 6); // r_in g_in b_in r_out g_out b_out
        }
    }
    
    #[test]
    fn test_look_file_saving() {
        let mut lut_data = LutData::new("Test LOOK".to_string(), 2, LutFormat::Look);
        lut_data.data = vec![
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
        ];
        
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.look");
        
        let saver = LutSaver::new();
        saver.save_lut(&lut_data, &file_path, LutFormat::Look).unwrap();
        
        assert!(file_path.exists());
        
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("<Look>"));
        assert!(content.contains("<Name>Test LOOK</Name>"));
        assert!(content.contains("<Size>2</Size>"));
        assert!(content.contains("<Data>"));
    }
    
    #[test]
    fn test_memory_saving() {
        let mut lut_data = LutData::new("Memory Test".to_string(), 2, LutFormat::Cube);
        lut_data.data = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        
        let saver = LutSaver::new();
        let bytes = saver.save_lut_to_memory(&lut_data, LutFormat::Cube).unwrap();
        
        let content = String::from_utf8(bytes).unwrap();
        assert!(content.contains("TITLE Memory Test"));
        assert!(content.contains("LUT_3D_SIZE 2"));
    }
    
    #[test]
    fn test_batch_export() {
        let mut lut_data1 = LutData::new("LUT1".to_string(), 2, LutFormat::Cube);
        lut_data1.data = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        
        let mut lut_data2 = LutData::new("LUT2".to_string(), 2, LutFormat::Cube);
        lut_data2.data = vec![[0.5, 0.5, 0.5], [0.25, 0.75, 0.5]];
        
        let luts = vec![(&lut_data1, "lut1"), (&lut_data2, "lut2")];
        
        let dir = tempdir().unwrap();
        let saver = LutSaver::new();
        let exported_files = saver.batch_export(&luts, dir.path(), LutFormat::Cube).unwrap();
        
        assert_eq!(exported_files.len(), 2);
        assert!(exported_files[0].exists());
        assert!(exported_files[1].exists());
    }
    
    #[test]
    fn test_metadata_export() {
        let mut lut_data = LutData::new("Meta Test".to_string(), 2, LutFormat::Cube);
        lut_data.data = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        
        let metadata = LutMetadata::new()
            .with_author("Test Author".to_string())
            .with_description("Test Description".to_string());
        
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("meta_test.cube");
        
        let saver = LutSaver::new();
        saver.export_lut_with_metadata(&lut_data, &file_path, LutFormat::Cube, &metadata).unwrap();
        
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("# Author: Test Author"));
        assert!(content.contains("# Description: Test Description"));
    }
}
