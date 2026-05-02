use crate::nodes::basic::blur::NextOdd;
use log::debug;

/// Blur kernels generation
pub struct BlurKernels {
    /// Blur radius
    radius: f32,
}

impl BlurKernels {
    /// Create new blur kernels
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
    
    /// Generate Gaussian kernel
    pub fn generate_gaussian_kernel(&self, size: usize) -> Vec<f32> {
        let mut kernel = Vec::with_capacity(size);
        let center = size as f32 / 2.0 - 0.5;
        let sigma = self.radius / 3.0; // Standard deviation
        
        let mut sum = 0.0;
        for i in 0..size {
            let x = i as f32 - center;
            let value = (-x * x / (2.0 * sigma * sigma)).exp();
            kernel.push(value);
            sum += value;
        }
        
        // Normalize kernel
        for value in kernel.iter_mut() {
            *value /= sum;
        }
        
        debug!("Generated {}x1 Gaussian kernel with sigma={:.2}", size, sigma);
        
        kernel
    }
    
    /// Generate 2D Gaussian kernel
    pub fn generate_gaussian_kernel_2d(&self, size: usize) -> Vec<Vec<f32>> {
        let mut kernel = Vec::with_capacity(size);
        let center = size as f32 / 2.0 - 0.5;
        let sigma = self.radius / 3.0;
        
        // Generate 2D kernel
        for y in 0..size {
            let mut row = Vec::with_capacity(size);
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let value = (-(dx * dx + dy * dy) / (2.0 * sigma * sigma)).exp();
                row.push(value);
            }
            kernel.push(row);
        }
        
        // Normalize kernel
        let mut sum = 0.0;
        for row in &kernel {
            for &value in row {
                sum += value;
            }
        }
        
        for row in kernel.iter_mut() {
            for value in row.iter_mut() {
                *value /= sum;
            }
        }
        
        debug!("Generated {}x{} 2D Gaussian kernel with sigma={:.2}", size, size, sigma);
        
        kernel
    }
    
    /// Generate box blur kernel
    pub fn generate_box_kernel(&self, size: usize) -> Vec<f32> {
        let value = 1.0 / size as f32;
        let kernel = vec![value; size];
        
        debug!("Generated {}x1 box kernel", size);
        
        kernel
    }
    
    /// Generate 2D box blur kernel
    pub fn generate_box_kernel_2d(&self, size: usize) -> Vec<Vec<f32>> {
        let value = 1.0 / (size * size) as f32;
        let kernel = vec![vec![value; size]; size];
        
        debug!("Generated {}x{} 2D box kernel", size, size);
        
        kernel
    }
    
    /// Generate motion blur kernel
    pub fn generate_motion_blur_kernel(&self, size: usize, angle: f32) -> Vec<f32> {
        let mut kernel = vec![0.0; size];
        let center = size as f32 / 2.0 - 0.5;
        let angle_rad = angle.to_radians();
        
        // Calculate motion blur line
        let half_length = self.radius;
        let dx = half_length * angle_rad.cos();
        let dy = half_length * angle_rad.sin();
        
        // Sample points along the motion line
        let samples = size;
        for i in 0..samples {
            let t = (i as f32 / (samples - 1) as f32) * 2.0 - 1.0; // -1 to 1
            let x = center + t * dx;
            let y = center + t * dy;
            
            // Find nearest kernel position
            let kernel_x = (x as usize).min(size - 1);
            kernel[kernel_x] += 1.0;
        }
        
        // Normalize kernel
        let sum: f32 = kernel.iter().sum();
        if sum > 0.0 {
            for value in kernel.iter_mut() {
                *value /= sum;
            }
        }
        
        debug!("Generated {}x1 motion blur kernel with angle={:.1}°", size, angle);
        
        kernel
    }
    
    /// Generate radial blur kernel
    pub fn generate_radial_kernel(&self, samples: usize) -> Vec<(f32, f32, f32)> {
        let mut kernel = Vec::with_capacity(samples);
        
        for i in 0..samples {
            let t = i as f32 / (samples - 1) as f32;
            let radius = t * self.radius;
            let weight = 1.0 / samples as f32;
            
            // Radial blur samples from center outward
            kernel.push((radius, weight, 0.0)); // (radius, weight, angle)
        }
        
        debug!("Generated radial blur kernel with {} samples", samples);
        
        kernel
    }
    
    /// Generate zoom blur kernel
    pub fn generate_zoom_kernel(&self, samples: usize) -> Vec<(f32, f32, f32)> {
        let mut kernel = Vec::with_capacity(samples);
        
        for i in 0..samples {
            let t = i as f32 / (samples - 1) as f32;
            let scale = 1.0 + t * self.radius * 0.1; // Scale factor
            let weight = 1.0 / samples as f32;
            
            // Zoom blur samples from center outward
            kernel.push((scale, weight, 0.0)); // (scale, weight, angle)
        }
        
        debug!("Generated zoom blur kernel with {} samples", samples);
        
        kernel
    }
    
    /// Generate separable Gaussian kernels (horizontal and vertical)
    pub fn generate_separable_gaussian_kernels(&self, size: usize) -> (Vec<f32>, Vec<f32>) {
        let horizontal = self.generate_gaussian_kernel(size);
        let vertical = horizontal.clone(); // Same for vertical
        
        debug!("Generated separable Gaussian kernels: {}x1 and {}x1", size, size);
        
        (horizontal, vertical)
    }
    
    /// Calculate optimal kernel size for given radius
    pub fn calculate_optimal_size(&self) -> usize {
        // Rule of thumb: kernel size should be about 3x the radius
        let size = (self.radius * 3.0) as usize;
        size.next_odd().max(3).min(51) // Clamp between 3 and 51
    }
    
    /// Get the current radius
    pub fn get_radius(&self) -> f32 {
        self.radius
    }
    
    /// Update radius
    pub fn update_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
}
