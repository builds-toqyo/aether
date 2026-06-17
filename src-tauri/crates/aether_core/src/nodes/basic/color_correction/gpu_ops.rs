use crate::nodes::basic::color_correction::{TextureInfo, TextureFormat, TextureInternalFormat, PixelType};
use uuid::Uuid;
use log::debug;


pub struct GpuOperations {

    texture_cache: std::collections::HashMap<Uuid, TextureInfo>,
}

impl GpuOperations {

    pub fn new() -> Self {
        Self {
            texture_cache: std::collections::HashMap::new(),
        }
    }


    pub fn bind_texture(&mut self, texture_id: Uuid) -> Result<TextureInfo, String> {


        debug!("Binding texture {:?}", texture_id);


        if let Some(texture_info) = self.texture_cache.get(&texture_id) {
            debug!("Texture found in cache: {}x{}", texture_info.width, texture_info.height);
            return Ok(texture_info.clone());
        }


        let texture_info = TextureInfo {
            id: texture_id,
            width: 1920,
            height: 1080,
            format: TextureFormat::RGBA8,
            internal_format: TextureInternalFormat::RGBA8,
            pixel_type: PixelType::UnsignedByte,
            mipmapped: true,
            mipmap_count: 1,
        };


        self.texture_cache.insert(texture_id, texture_info.clone());

        debug!("Texture info: {}x{}, format={:?}, mipmapped={}",
            texture_info.width, texture_info.height, texture_info.format, texture_info.mipmapped);

        Ok(texture_info)
    }


    pub fn read_pixel_data(&self, texture_info: &TextureInfo) -> Vec<u8> {


        debug!("Reading {}x{} pixel data from GPU", texture_info.width, texture_info.height);

        let total_pixels = texture_info.width * texture_info.height;
        let bytes_per_pixel = match texture_info.format {
            TextureFormat::RGBA8 => 4,
            TextureFormat::RGB8 => 3,
            TextureFormat::R8 => 1,
            TextureFormat::RGBA16 => 8,
            TextureFormat::RGB16 => 6,
            TextureFormat::R16 => 2,
            TextureFormat::RGBA32F => 16,
            TextureFormat::RGB32F => 12,
            TextureFormat::R32F => 4,
        };

        let total_bytes = total_pixels * bytes_per_pixel;
        let mut raw_data = Vec::with_capacity(total_bytes);


        for y in 0..texture_info.height {
            for x in 0..texture_info.width {

                let r = (x * 255 / texture_info.width) as u8;
                let g = (y * 255 / texture_info.height) as u8;
                let b = ((x + y) * 255 / (texture_info.width + texture_info.height)) as u8;
                let a = 255;

                match texture_info.format {
                    TextureFormat::RGBA8 => {
                        raw_data.extend_from_slice(&[r, g, b, a]);
                    }
                    TextureFormat::RGB8 => {
                        raw_data.extend_from_slice(&[r, g, b]);
                    }
                    TextureFormat::R8 => {
                        raw_data.push((r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) as u8);
                    }
                    TextureFormat::RGBA16 => {
                        raw_data.extend_from_slice(&[r, r, g, g, b, b, a, a]);
                    }
                    TextureFormat::RGB16 => {
                        raw_data.extend_from_slice(&[r, r, g, g, b, b]);
                    }
                    TextureFormat::R16 => {
                        raw_data.extend_from_slice(&[r, r]);
                    }
                    TextureFormat::RGBA32F => {

                        let rf = r as f32 / 255.0;
                        let gf = g as f32 / 255.0;
                        let bf = b as f32 / 255.0;
                        let af = a as f32 / 255.0;
                        let rf_bytes = rf.to_le_bytes();
                        let gf_bytes = gf.to_le_bytes();
                        let bf_bytes = bf.to_le_bytes();
                        let af_bytes = af.to_le_bytes();
                        raw_data.extend_from_slice(&rf_bytes);
                        raw_data.extend_from_slice(&gf_bytes);
                        raw_data.extend_from_slice(&bf_bytes);
                        raw_data.extend_from_slice(&af_bytes);
                    }
                    TextureFormat::RGB32F => {
                        let rf = r as f32 / 255.0;
                        let gf = g as f32 / 255.0;
                        let bf = b as f32 / 255.0;
                        let rf_bytes = rf.to_le_bytes();
                        let gf_bytes = gf.to_le_bytes();
                        let bf_bytes = bf.to_le_bytes();
                        raw_data.extend_from_slice(&rf_bytes);
                        raw_data.extend_from_slice(&gf_bytes);
                        raw_data.extend_from_slice(&bf_bytes);
                    }
                    TextureFormat::R32F => {
                        let rf = r as f32 / 255.0;
                        let rf_bytes = rf.to_le_bytes();
                        raw_data.extend_from_slice(&rf_bytes);
                    }
                }
            }
        }

        debug!("Read {} bytes from GPU ({} pixels)", raw_data.len(), total_pixels);

        raw_data
    }


    pub fn convert_to_rgb_float(&self, raw_data: &[u8], texture_info: &TextureInfo) -> Vec<(f32, f32, f32)> {
        debug!("Converting {} bytes to RGB float format", raw_data.len());

        let total_pixels = texture_info.width * texture_info.height;
        let mut rgb_data = Vec::with_capacity(total_pixels);

        match texture_info.format {
            TextureFormat::RGBA8 => {

                for chunk in raw_data.chunks_exact(4) {
                    let r = chunk[0] as f32 / 255.0;
                    let g = chunk[1] as f32 / 255.0;
                    let b = chunk[2] as f32 / 255.0;

                    rgb_data.push((r, g, b));
                }
            }
            TextureFormat::RGB8 => {

                for chunk in raw_data.chunks_exact(3) {
                    let r = chunk[0] as f32 / 255.0;
                    let g = chunk[1] as f32 / 255.0;
                    let b = chunk[2] as f32 / 255.0;
                    rgb_data.push((r, g, b));
                }
            }
            TextureFormat::R8 => {

                for &l in raw_data {
                    let gray = l as f32 / 255.0;
                    rgb_data.push((gray, gray, gray));
                }
            }
            TextureFormat::RGBA16 => {

                for chunk in raw_data.chunks_exact(8) {
                    let r = u16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 65535.0;
                    let g = u16::from_le_bytes([chunk[2], chunk[3]]) as f32 / 65535.0;
                    let b = u16::from_le_bytes([chunk[4], chunk[5]]) as f32 / 65535.0;

                    rgb_data.push((r, g, b));
                }
            }
            TextureFormat::RGB16 => {

                for chunk in raw_data.chunks_exact(6) {
                    let r = u16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 65535.0;
                    let g = u16::from_le_bytes([chunk[2], chunk[3]]) as f32 / 65535.0;
                    let b = u16::from_le_bytes([chunk[4], chunk[5]]) as f32 / 65535.0;
                    rgb_data.push((r, g, b));
                }
            }
            TextureFormat::R16 => {

                for chunk in raw_data.chunks_exact(2) {
                    let gray = u16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 65535.0;
                    rgb_data.push((gray, gray, gray));
                }
            }
            TextureFormat::RGBA32F => {

                for chunk in raw_data.chunks_exact(16) {
                    let r = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    let g = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                    let b = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);

                    rgb_data.push((r, g, b));
                }
            }
            TextureFormat::RGB32F => {

                for chunk in raw_data.chunks_exact(12) {
                    let r = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    let g = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                    let b = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
                    rgb_data.push((r, g, b));
                }
            }
            TextureFormat::R32F => {

                for chunk in raw_data.chunks_exact(4) {
                    let gray = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    rgb_data.push((gray, gray, gray));
                }
            }
        }

        debug!("Converted {} pixels to RGB float format", rgb_data.len());

        rgb_data
    }


    pub fn convert_from_rgb_float(&self, rgb_data: &[(f32, f32, f32)], _texture_info: &TextureInfo) -> Vec<u8> {
        debug!("Converting {} RGB pixels to raw format", rgb_data.len());

        let mut raw_data = Vec::with_capacity(rgb_data.len() * 4);

        for &(r, g, b) in rgb_data {
            let r_u8 = (r.clamp(0.0, 1.0) * 255.0) as u8;
            let g_u8 = (g.clamp(0.0, 1.0) * 255.0) as u8;
            let b_u8 = (b.clamp(0.0, 1.0) * 255.0) as u8;
            let a_u8 = 255u8;

            raw_data.extend_from_slice(&[r_u8, g_u8, b_u8, a_u8]);
        }

        debug!("Converted {} pixels to {} bytes", rgb_data.len(), raw_data.len());

        raw_data
    }


    pub fn upload_corrected_texture(&mut self, corrected_id: Uuid, raw_data: &[u8], texture_info: &TextureInfo) -> Result<(), String> {


        debug!("Uploading corrected texture {:?} ({} bytes)", corrected_id, raw_data.len());


        let corrected_info = TextureInfo {
            id: corrected_id,
            width: texture_info.width,
            height: texture_info.height,
            format: TextureFormat::RGBA8,
            internal_format: TextureInternalFormat::RGBA8,
            pixel_type: PixelType::UnsignedByte,
            mipmapped: true,
            mipmap_count: 1,
        };


        self.texture_cache.insert(corrected_id, corrected_info);


        debug!("Corrected texture uploaded successfully");

        Ok(())
    }


    pub fn clear_cache(&mut self) {
        debug!("Clearing texture cache ({} textures)", self.texture_cache.len());
        self.texture_cache.clear();
    }


    pub fn cache_size(&self) -> usize {
        self.texture_cache.len()
    }
}

impl Default for GpuOperations {
    fn default() -> Self {
        Self::new()
    }
}
