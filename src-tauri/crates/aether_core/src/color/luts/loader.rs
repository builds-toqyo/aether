

use std::io::Read;
use anyhow::{Result, anyhow};
use log::debug;

use super::types::{LutData, LutFormat};


pub struct LutLoader {
    config: crate::color::luts::config::LutConfig,
}

impl LutLoader {

    pub fn new() -> Self {
        Self {
            config: crate::color::luts::config::LutConfig::default(),
        }
    }


    pub fn load_lut(&self, file_path: &std::path::Path) -> Result<LutData> {
        debug!("Loading LUT from: {:?}", file_path);

        let file_extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("Invalid file extension"))?;

        let lut_data = match LutFormat::from_extension(file_extension) {
            Some(LutFormat::Cube) => self.load_cube_file(file_path)?,
            Some(LutFormat::ThreeDL) => self.load_3dl_file(file_path)?,
            Some(LutFormat::Look) => self.load_look_file(file_path)?,
            None => return Err(anyhow!("Unsupported LUT format: {}", file_extension)),
        };

        debug!("LUT loaded successfully: {}", lut_data.name);

        Ok(lut_data)
    }


    fn load_cube_file(&self, file_path: &std::path::Path) -> Result<LutData> {
        debug!("Loading Cube LUT file: {:?}", file_path);

        let mut file = std::fs::File::open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut lut_data = LutData::default();
        let mut size = 33;
        let mut title = String::new();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with("TITLE") {
                title = line.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
            } else if line.starts_with("LUT_3D_SIZE") {
                size = line.split_whitespace()
                    .nth(1)
                    .ok_or_else(|| anyhow!("Invalid LUT_3D_SIZE line"))?
                    .parse()
                    .map_err(|e| anyhow!("Invalid LUT size: {}", e))?;
            } else if line.contains(' ') {

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let r = parts[0].parse::<f32>()?;
                    let g = parts[1].parse::<f32>()?;
                    let b = parts[2].parse::<f32>()?;

                    lut_data.data.push([r, g, b]);
                }
            }
        }

        lut_data.name = title;
        lut_data.size = size;
        lut_data.format = LutFormat::Cube;


        let expected_size = size * size * size;
        if lut_data.data.len() != expected_size as usize {
            debug!("LUT data size mismatch: expected {}, got {}", expected_size, lut_data.data.len());
        }

        Ok(lut_data)
    }


    fn load_3dl_file(&self, file_path: &std::path::Path) -> Result<LutData> {
        debug!("Loading 3DL LUT file: {:?}", file_path);

        let mut file = std::fs::File::open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut lut_data = LutData::default();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.contains(' ') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 {

                    let r_out = parts[3].parse::<f32>()?;
                    let g_out = parts[4].parse::<f32>()?;
                    let b_out = parts[5].parse::<f32>()?;

                    lut_data.data.push([r_out, g_out, b_out]);
                }
            }
        }

        let data_len = lut_data.data.len();
        let size = (data_len as f32).cbrt() as u32;

        lut_data.name = file_path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("3dl_lut")
            .to_string();
        lut_data.size = size;
        lut_data.format = LutFormat::ThreeDL;

        Ok(lut_data)
    }


    fn load_look_file(&self, file_path: &std::path::Path) -> Result<LutData> {
        debug!("Loading LOOK LUT file: {:?}", file_path);

        let mut file = std::fs::File::open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut lut_data = LutData::default();
        let mut size = 33;
        let mut name = String::new();
        let mut in_data_section = false;

        for line in content.lines() {
            let line = line.trim();

            if line.contains("<Name>") {
                name = line.replace("<Name>", "").replace("</Name>", "").trim().to_string();
            } else if line.contains("<Size>") {
                let size_str = line.replace("<Size>", "").replace("</Size>", "");
                size = size_str.trim().parse().unwrap_or(33);
            } else if line.contains("<Data>") {
                in_data_section = true;
            } else if line.contains("</Data>") {
                in_data_section = false;
            } else if in_data_section && line.contains(' ') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let r = parts[0].parse::<f32>()?;
                    let g = parts[1].parse::<f32>()?;
                    let b = parts[2].parse::<f32>()?;

                    lut_data.data.push([r, g, b]);
                }
            }
        }

        lut_data.name = name;
        lut_data.size = size;
        lut_data.format = LutFormat::Look;


        if lut_data.data.is_empty() {
            lut_data.generate_identity_lut(size);
        }

        Ok(lut_data)
    }


    pub fn load_lut_from_memory(&self, data: &[u8], format: LutFormat, name: &str) -> Result<LutData> {
        debug!("Loading LUT from memory: {} ({:?})", name, format);

        let content = String::from_utf8(data.to_vec())?;

        let lut_data = match format {
            LutFormat::Cube => self.load_cube_from_string(&content, name)?,
            LutFormat::ThreeDL => self.load_3dl_from_string(&content, name)?,
            LutFormat::Look => self.load_look_from_string(&content, name)?,
        };

        debug!("LUT loaded from memory successfully: {}", lut_data.name);

        Ok(lut_data)
    }


    fn load_cube_from_string(&self, content: &str, name: &str) -> Result<LutData> {
        let mut lut_data = LutData::default();
        let mut size = 33;

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with("LUT_3D_SIZE") {
                size = line.split_whitespace()
                    .nth(1)
                    .ok_or_else(|| anyhow!("Invalid LUT_3D_SIZE line"))?
                    .parse()
                    .map_err(|e| anyhow!("Invalid LUT size: {}", e))?;
            } else if line.contains(' ') && !line.starts_with("TITLE") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let r = parts[0].parse::<f32>()?;
                    let g = parts[1].parse::<f32>()?;
                    let b = parts[2].parse::<f32>()?;

                    lut_data.data.push([r, g, b]);
                }
            }
        }

        lut_data.name = name.to_string();
        lut_data.size = size;
        lut_data.format = LutFormat::Cube;

        Ok(lut_data)
    }


    fn load_3dl_from_string(&self, content: &str, name: &str) -> Result<LutData> {
        let mut lut_data = LutData::default();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.contains(' ') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 {
                    let r_out = parts[3].parse::<f32>()?;
                    let g_out = parts[4].parse::<f32>()?;
                    let b_out = parts[5].parse::<f32>()?;

                    lut_data.data.push([r_out, g_out, b_out]);
                }
            }
        }


        let data_len = lut_data.data.len();
        let size = (data_len as f32).cbrt() as u32;

        lut_data.name = name.to_string();
        lut_data.size = size;
        lut_data.format = LutFormat::ThreeDL;

        Ok(lut_data)
    }


    fn load_look_from_string(&self, content: &str, name: &str) -> Result<LutData> {
        let mut lut_data = LutData::default();
        let mut size = 33;
        let mut in_data_section = false;

        for line in content.lines() {
            let line = line.trim();

            if line.contains("<Size>") {
                let size_str = line.replace("<Size>", "").replace("</Size>", "");
                size = size_str.trim().parse().unwrap_or(33);
            } else if line.contains("<Data>") {
                in_data_section = true;
            } else if line.contains("</Data>") {
                in_data_section = false;
            } else if in_data_section && line.contains(' ') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let r = parts[0].parse::<f32>()?;
                    let g = parts[1].parse::<f32>()?;
                    let b = parts[2].parse::<f32>()?;

                    lut_data.data.push([r, g, b]);
                }
            }
        }

        lut_data.name = name.to_string();
        lut_data.size = size;
        lut_data.format = LutFormat::Look;


        if lut_data.data.is_empty() {
            lut_data.generate_identity_lut(size);
        }

        Ok(lut_data)
    }


    pub fn validate_lut_file(&self, file_path: &std::path::Path) -> Result<LutValidationResult> {
        debug!("Validating LUT file: {:?}", file_path);

        let lut_data = self.load_lut(file_path)?;

        let validation = lut_data.validate();

        let result = LutValidationResult {
            is_valid: validation.is_ok(),
            errors: validation.err().map(|e| vec![e]).unwrap_or_default(),
            warnings: self.collect_warnings(&lut_data),
            info: crate::color::luts::types::LutInfo {
                name: lut_data.name.clone(),
                size: lut_data.size,
                format: lut_data.format,
                data_points: lut_data.data.len(),
            },
        };

        debug!("LUT validation completed: valid={}", result.is_valid);

        Ok(result)
    }


    fn collect_warnings(&self, lut_data: &LutData) -> Vec<String> {
        let mut warnings = Vec::new();


        if lut_data.size != 33 && lut_data.size != 65 {
            warnings.push(format!("Non-standard LUT size: {} (common sizes are 33 and 65)", lut_data.size));
        }


        let expected_size = lut_data.size * lut_data.size * lut_data.size;
        if lut_data.data.len() != expected_size as usize {
            warnings.push(format!("Data size mismatch: expected {}, got {}", expected_size, lut_data.data.len()));
        }


        let mut out_of_range_count = 0;
        for rgb in &lut_data.data {
            for &value in rgb {
                if value < 0.0 || value > 1.0 {
                    out_of_range_count += 1;
                }
            }
        }

        if out_of_range_count > 0 {
            warnings.push(format!("{} values out of range [0, 1]", out_of_range_count));
        }

        warnings
    }
}

impl Default for LutLoader {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Clone)]
pub struct LutValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: crate::color::luts::types::LutInfo,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_cube_file_loading() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.cube");

        let cube_content = r#"TITLE Test LUT
LUT_3D_SIZE 2

0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0
0.0 0.0 1.0
1.0 1.0 0.0
1.0 0.0 1.0
0.0 1.0 1.0
1.0 1.0 1.0
"#;

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(cube_content.as_bytes()).unwrap();

        let loader = LutLoader::new();
        let lut_data = loader.load_lut(&file_path).unwrap();

        assert_eq!(lut_data.name, "Test LUT");
        assert_eq!(lut_data.size, 2);
        assert_eq!(lut_data.format, LutFormat::Cube);
        assert_eq!(lut_data.data.len(), 8);
    }

    #[test]
    fn test_3dl_file_loading() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.3dl");

        let _3dl_content = r#"0.0 0.0 0.0 0.0 0.0 0.0
1.0 0.0 0.0 1.0 0.0 0.0
0.0 1.0 0.0 0.0 1.0 0.0
0.0 0.0 1.0 0.0 0.0 1.0
"#;

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(_3dl_content.as_bytes()).unwrap();

        let loader = LutLoader::new();
        let lut_data = loader.load_lut(&file_path).unwrap();

        assert_eq!(lut_data.format, LutFormat::ThreeDL);
        assert_eq!(lut_data.data.len(), 4);
    }

    #[test]
    fn test_memory_loading() {
        let cube_content = r#"TITLE Memory Test
LUT_3D_SIZE 2

0.0 0.0 0.0
1.0 1.0 1.0
"#;

        let loader = LutLoader::new();
        let lut_data = loader.load_lut_from_memory(cube_content.as_bytes(), LutFormat::Cube, "memory_test").unwrap();

        assert_eq!(lut_data.name, "memory_test");
        assert_eq!(lut_data.size, 2);
        assert_eq!(lut_data.format, LutFormat::Cube);
    }

    #[test]
    fn test_lut_validation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("invalid.cube");


        let cube_content = r#"TITLE Invalid LUT
LUT_3D_SIZE 3

0.0 0.0 0.0
1.0 1.0 1.0
"#;

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(cube_content.as_bytes()).unwrap();

        let loader = LutLoader::new();
        let result = loader.validate_lut_file(&file_path).unwrap();

        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert!(!result.warnings.is_empty());
    }
}
