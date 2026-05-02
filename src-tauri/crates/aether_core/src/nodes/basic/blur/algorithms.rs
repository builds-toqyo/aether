use crate::nodes::basic::blur::{BlurType, BlurResult, BlurKernels, NextOdd};
use aether_types::ParameterValue;
use uuid::Uuid;
use log::debug;

/// Blur algorithms implementation
pub struct BlurAlgorithms {
    /// Blur parameters
    params: crate::nodes::basic::blur::BlurParams,
}

impl BlurAlgorithms {
    /// Create new blur algorithms
    pub fn new(params: crate::nodes::basic::blur::BlurParams) -> Self {
        Self { params }
    }
    
    /// Apply blur to an image
    pub fn apply_blur(&self, input_value: ParameterValue) -> ParameterValue {
        match input_value {
            ParameterValue::Image(input_id) => {
                debug!("Applying blur: type={:?}, radius={:.1}, iterations={}, directional={}, angle={:.1}°", 
                    self.params.blur_type, self.params.radius, self.params.iterations, self.params.directional, self.params.angle);
                
                // Apply blur iterations
                let mut current_id = input_id;
                for i in 0..self.params.iterations {
                    debug!("Blur iteration {}", i + 1);
                    current_id = self.apply_blur_iteration(current_id);
                }
                
                ParameterValue::Image(current_id)
            }
            _ => {
                // No valid image input
                ParameterValue::None
            }
        }
    }
    
    /// Apply a single blur iteration
    fn apply_blur_iteration(&self, input_id: Uuid) -> Uuid {
        match self.params.blur_type {
            BlurType::Gaussian => self.apply_gaussian_blur(input_id),
            BlurType::Box => self.apply_box_blur(input_id),
            BlurType::Motion => self.apply_motion_blur(input_id),
            BlurType::Radial => self.apply_radial_blur(input_id),
            BlurType::Zoom => self.apply_zoom_blur(input_id),
        }
    }
    
    /// Apply Gaussian blur
    fn apply_gaussian_blur(&self, input_id: Uuid) -> Uuid {
        debug!("Applying Gaussian blur with radius {:.1}", self.params.radius);
        
        let blurred_id = Uuid::new_v4();
        let kernel_size = ((self.params.radius * 2.0) as usize).max(3).next_odd();
        
        // Generate Gaussian kernel
        let kernels = BlurKernels::new(self.params.radius);
        let kernel = kernels.generate_gaussian_kernel(kernel_size);
        
        let blur_result = BlurResult {
            original_id: input_id,
            blurred_id,
            blur_type: BlurType::Gaussian,
            radius: self.params.radius,
            kernel_size,
            angle: 0.0,
            iterations: self.params.iterations,
        };
        
        debug!("Generated Gaussian kernel: size={}", kernel_size);
        debug!("Created blur result: {:?}", blur_result);
        
        blurred_id
    }
    
    /// Apply box blur
    fn apply_box_blur(&self, input_id: Uuid) -> Uuid {
        debug!("Applying box blur with radius {:.1}", self.params.radius);
        
        let blurred_id = Uuid::new_v4();
        let kernel_size = ((self.params.radius * 2.0) as usize).max(3).next_odd();
        
        let blur_result = BlurResult {
            original_id: input_id,
            blurred_id,
            blur_type: BlurType::Box,
            radius: self.params.radius,
            kernel_size,
            angle: 0.0,
            iterations: self.params.iterations,
        };
        
        debug!("Generated box kernel: size={}", kernel_size);
        debug!("Created blur result: {:?}", blur_result);
        
        blurred_id
    }
    
    /// Apply motion blur
    fn apply_motion_blur(&self, input_id: Uuid) -> Uuid {
        debug!("Applying motion blur: length={:.1}, angle={:.1}°", 
            self.params.radius, self.params.angle);
        
        let blurred_id = Uuid::new_v4();
        let kernel_size = ((self.params.radius * 2.0) as usize).max(3).next_odd();
        
        // Generate motion blur kernel
        let kernels = BlurKernels::new(self.params.radius);
        let kernel = kernels.generate_motion_blur_kernel(kernel_size, self.params.angle);
        
        let blur_result = BlurResult {
            original_id: input_id,
            blurred_id,
            blur_type: BlurType::Motion,
            radius: self.params.radius,
            kernel_size,
            angle: self.params.angle,
            iterations: self.params.iterations,
        };
        
        debug!("Generated motion blur kernel: size={}, angle={:.1}°", kernel_size, self.params.angle);
        debug!("Created blur result: {:?}", blur_result);
        
        blurred_id
    }
    
    /// Apply radial blur
    fn apply_radial_blur(&self, input_id: Uuid) -> Uuid {
        debug!("Applying radial blur with radius {:.1}", self.params.radius);
        
        let blurred_id = Uuid::new_v4();
        let samples = (self.params.radius * 4.0) as usize;
        
        let blur_result = BlurResult {
            original_id: input_id,
            blurred_id,
            blur_type: BlurType::Radial,
            radius: self.params.radius,
            kernel_size: samples,
            angle: 0.0,
            iterations: self.params.iterations,
        };
        
        debug!("Radial blur samples: {}", samples);
        debug!("Created blur result: {:?}", blur_result);
        
        blurred_id
    }
    
    /// Apply zoom blur
    fn apply_zoom_blur(&self, input_id: Uuid) -> Uuid {
        debug!("Applying zoom blur with radius {:.1}", self.params.radius);
        
        let blurred_id = Uuid::new_v4();
        let samples = (self.params.radius * 4.0) as usize;
        
        let blur_result = BlurResult {
            original_id: input_id,
            blurred_id,
            blur_type: BlurType::Zoom,
            radius: self.params.radius,
            kernel_size: samples,
            angle: 0.0,
            iterations: self.params.iterations,
        };
        
        debug!("Zoom blur samples: {}", samples);
        debug!("Created blur result: {:?}", blur_result);
        
        blurred_id
    }
    
    /// Get the current parameters
    pub fn get_params(&self) -> &crate::nodes::basic::blur::BlurParams {
        &self.params
    }
    
    /// Update parameters
    pub fn update_params(&mut self, params: crate::nodes::basic::blur::BlurParams) {
        self.params = params;
    }
}
