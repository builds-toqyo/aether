#[derive(Debug, Clone)]
pub struct HdrImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<HdrPixel>,
    pub color_primaries: ColorPrimaries,
    pub transfer_function: TransferFunction,
    pub max_nits: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HdrPixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HdrDisplayType {
    Sdr,
    Hdr10,
    DolbyVision,
    Hdr10Plus,
    Hlg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPrimaries {
    Rec709,
    Rec2020,
    P3,
    DciP3,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferFunction {
    Srgb,
    Rec709,
    Rec2020,
    Pq,
    Hlg,
}

impl HdrImage {

    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![HdrPixel::default(); (width * height) as usize],
            color_primaries: ColorPrimaries::Rec709,
            transfer_function: TransferFunction::Rec709,
            max_nits: 100.0,
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&HdrPixel> {
        if x < self.width && y < self.height {
            self.data.get((y * self.width + x) as usize)
        } else {
            None
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: HdrPixel) -> Result<(), &'static str> {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = pixel;
            Ok(())
        } else {
            Err("Pixel coordinates out of bounds")
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn total_pixels(&self) -> usize {
        (self.width * self.height) as usize
    }
}

impl HdrPixel {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    pub fn clamp(&self, min_nits: f32, max_nits: f32) -> Self {
        Self {
            r: self.r.clamp(min_nits, max_nits),
            g: self.g.clamp(min_nits, max_nits),
            b: self.b.clamp(min_nits, max_nits),
        }
    }

    pub fn to_sdr_rgb(&self, reference_nits: f32) -> [u8; 3] {
        let scale = 255.0 / reference_nits;
        [
            (self.r * scale).clamp(0.0, 255.0) as u8,
            (self.g * scale).clamp(0.0, 255.0) as u8,
            (self.b * scale).clamp(0.0, 255.0) as u8,
        ]
    }
}

impl Default for HdrDisplayType {
    fn default() -> Self {
        Self::Sdr
    }
}

impl Default for ColorPrimaries {
    fn default() -> Self {
        Self::Rec709
    }
}

impl Default for TransferFunction {
    fn default() -> Self {
        Self::Rec709
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdr_image_creation() {
        let image = HdrImage::new(100, 100);

        assert_eq!(image.width, 100);
        assert_eq!(image.height, 100);
        assert_eq!(image.total_pixels(), 10000);
        assert_eq!(image.data.len(), 10000);
    }

    #[test]
    fn test_hdr_pixel_operations() {
        let pixel = HdrPixel::new(1000.0, 500.0, 200.0);

        assert_eq!(pixel.luminance(), 212.6 + 357.6 + 14.44);

        let clamped = pixel.clamp(0.0, 800.0);
        assert_eq!(clamped.r, 800.0);
        assert_eq!(clamped.g, 500.0);
        assert_eq!(clamped.b, 200.0);

        let sdr = pixel.to_sdr_rgb(1000.0);
        assert_eq!(sdr[0], 255);
        assert_eq!(sdr[1], 128);
        assert_eq!(sdr[2], 51);
    }

    #[test]
    fn test_hdr_image_pixel_access() {
        let mut image = HdrImage::new(10, 10);
        let pixel = HdrPixel::new(500.0, 500.0, 500.0);


        assert!(image.set_pixel(5, 5, pixel).is_ok());
        assert!(image.set_pixel(10, 10, pixel).is_err());

        let retrieved = image.get_pixel(5, 5);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().r, 500.0);

        let out_of_bounds = image.get_pixel(10, 10);
        assert!(out_of_bounds.is_none());
    }
}
