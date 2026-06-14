use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Histogram {
    /// Luminance histogram (256 bins)
    pub luminance: Vec<u32>,
    /// Red channel histogram (256 bins)
    pub red: Vec<u32>,
    /// Green channel histogram (256 bins)
    pub green: Vec<u32>,
    /// Blue channel histogram (256 bins)
    pub blue: Vec<u32>,
}

/// Vectorscope data for chrominance analysis
#[derive(Debug, Clone, Default)]
pub struct Vectorscope {
    /// UV color space points (U, V coordinates)
    pub uv_points: Vec<(f32, f32)>,
}

/// Waveform data for luminance per scanline
#[derive(Debug, Clone, Default)]
pub struct Waveform {
    /// Luminance values per scanline (height x width)
    pub luminance_per_scanline: Vec<Vec<u8>>,
}

/// Analyze image data to generate histogram
pub fn analyze_histogram(image_data: &[u8], width: usize, height: usize) -> Histogram {
    let mut histogram = Histogram::default();
    
    // Initialize histograms with 256 bins
    histogram.luminance = vec![0; 256];
    histogram.red = vec![0; 256];
    histogram.green = vec![0; 256];
    histogram.blue = vec![0; 256];
    
    // Process pixels (assuming RGBA format)
    for chunk in image_data.chunks_exact(4) {
        let r = chunk[0] as usize;
        let g = chunk[1] as usize;
        let b = chunk[2] as usize;
        let a = chunk[3];
        
        // Skip fully transparent pixels
        if a == 0 {
            continue;
        }
        
        // Calculate luminance using Rec. 709 coefficients
        let luminance = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) as usize;
        let luminance = luminance.min(255);
        
        histogram.red[r] += 1;
        histogram.green[g] += 1;
        histogram.blue[b] += 1;
        histogram.luminance[luminance] += 1;
    }
    
    histogram
}

/// Analyze image data to generate vectorscope
pub fn analyze_vectorscope(image_data: &[u8], width: usize, height: usize) -> Vectorscope {
    let mut vectorscope = Vectorscope::default();
    let mut uv_points = Vec::new();
    
    // Sample pixels (downsample for performance)
    let sample_step = if width * height > 1_000_000 { 100 } else { 10 };
    
    for (i, chunk) in image_data.chunks_exact(4).enumerate() {
        if i % sample_step != 0 {
            continue;
        }
        
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;
        let a = chunk[3];
        
        // Skip fully transparent pixels
        if a == 0 {
            continue;
        }
        
        // Convert RGB to YUV (Rec. 709)
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let u = 0.5 * (b - y) / (1.0 - 0.114) + 0.5;
        let v = 0.5 * (r - y) / (1.0 - 0.299) + 0.5;
        
        // Normalize UV to -1 to 1 range
        let u_norm = (u - 0.5) * 2.0;
        let v_norm = (v - 0.5) * 2.0;
        
        uv_points.push((u_norm, v_norm));
    }
    
    vectorscope.uv_points = uv_points;
    vectorscope
}

/// Analyze image data to generate waveform
pub fn analyze_waveform(image_data: &[u8], width: usize, height: usize) -> Waveform {
    let mut waveform = Waveform::default();
    let mut luminance_per_scanline = vec![vec![0u8; width]; height];
    
    // Process each scanline
    for y in 0..height {
        let row_start = y * width * 4;
        let row_end = row_start + width * 4;
        
        if row_end > image_data.len() {
            break;
        }
        
        let row_data = &image_data[row_start..row_end];
        
        for (x, chunk) in row_data.chunks_exact(4).enumerate() {
            let r = chunk[0] as f32;
            let g = chunk[1] as f32;
            let b = chunk[2] as f32;
            let a = chunk[3];
            
            // Use luminance for waveform
            let luminance = if a > 0 {
                (0.2126 * r + 0.7152 * g + 0.0722 * b) as u8
            } else {
                0
            };
            
            if x < width {
                luminance_per_scanline[y][x] = luminance;
            }
        }
    }
    
    waveform.luminance_per_scanline = luminance_per_scanline;
    waveform
}

/// Convert histogram to serializable format for frontend
pub fn histogram_to_map(histogram: &Histogram) -> HashMap<String, Vec<u32>> {
    let mut map = HashMap::new();
    map.insert("luminance".to_string(), histogram.luminance.clone());
    map.insert("red".to_string(), histogram.red.clone());
    map.insert("green".to_string(), histogram.green.clone());
    map.insert("blue".to_string(), histogram.blue.clone());
    map
}

/// Convert vectorscope to serializable format for frontend
pub fn vectorscope_to_list(vectorscope: &Vectorscope) -> Vec<(f32, f32)> {
    vectorscope.uv_points.clone()
}

/// Convert waveform to serializable format for frontend
pub fn waveform_to_list(waveform: &Waveform) -> Vec<Vec<u8>> {
    waveform.luminance_per_scanline.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_analysis() {
        // Create a simple red image (RGBA)
        let image_data = vec![255u8, 0, 0, 255; 100 * 100 * 4];
        let histogram = analyze_histogram(&image_data, 100, 100);
        
        // Red channel should have all pixels at 255
        assert_eq!(histogram.red[255], 100 * 100);
        assert_eq!(histogram.green[0], 100 * 100);
        assert_eq!(histogram.blue[0], 100 * 100);
    }

    #[test]
    fn test_vectorscope_analysis() {
        // Create a simple image
        let image_data = vec![255u8, 0, 0, 255; 100 * 100 * 4];
        let vectorscope = analyze_vectorscope(&image_data, 100, 100);
        
        // Should have some UV points
        assert!(!vectorscope.uv_points.is_empty());
    }

    #[test]
    fn test_waveform_analysis() {
        // Create a simple image
        let image_data = vec![255u8, 0, 0, 255; 10 * 10 * 4];
        let waveform = analyze_waveform(&image_data, 10, 10);
        
        // Should have 10 scanlines
        assert_eq!(waveform.luminance_per_scanline.len(), 10);
    }
}
