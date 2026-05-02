use uuid::Uuid;

/// Metadata for a blurred image
#[derive(Debug, Clone)]
pub struct BlurResult {
    /// Original image ID
    pub original_id: Uuid,
    /// Blurred image ID
    pub blurred_id: Uuid,
    /// Type of blur applied
    pub blur_type: BlurType,
    /// Blur radius
    pub radius: f32,
    /// Kernel size
    pub kernel_size: usize,
    /// Blur angle (for motion blur)
    pub angle: f32,
    /// Number of iterations
    pub iterations: u32,
}

/// Types of blur algorithms
#[derive(Debug, Clone, PartialEq)]
pub enum BlurType {
    /// Gaussian blur
    Gaussian,
    /// Box blur (average)
    Box,
    /// Motion blur
    Motion,
    /// Radial blur
    Radial,
    /// Zoom blur
    Zoom,
}

/// Blur parameters
#[derive(Debug, Clone)]
pub struct BlurParams {
    /// Blur radius (0.0 to 100.0)
    pub radius: f32,
    /// Type of blur
    pub blur_type: BlurType,
    /// Number of iterations (1 to 10)
    pub iterations: u32,
    /// Directional blur
    pub directional: bool,
    /// Blur angle in degrees (for directional/motion blur)
    pub angle: f32,
}

impl Default for BlurParams {
    fn default() -> Self {
        Self {
            radius: 5.0,
            blur_type: BlurType::Gaussian,
            iterations: 1,
            directional: false,
            angle: 0.0,
        }
    }
}

impl BlurParams {
    /// Create new blur parameters
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Validate and clamp parameters to valid ranges
    pub fn validate(&mut self) {
        self.radius = self.radius.clamp(0.0, 100.0);
        self.iterations = self.iterations.clamp(1, 10);
        self.angle = self.angle.rem_euclid(360.0);
    }
    
    /// Check if blur is active (radius > 0)
    pub fn is_active(&self) -> bool {
        self.radius > 0.0
    }
}

/// Helper trait for extending usize to get next odd number
pub trait NextOdd {
    fn next_odd(self) -> usize;
}

impl NextOdd for usize {
    fn next_odd(self) -> usize {
        if self % 2 == 0 {
            self + 1
        } else {
            self
        }
    }
}
