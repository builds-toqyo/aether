use uuid::Uuid;

/// Transform parameters
#[derive(Debug, Clone)]
pub struct TransformParams {
    /// Position (x, y)
    pub position: (f32, f32),
    /// Scale (x, y)
    pub scale: (f32, f32),
    /// Rotation in degrees
    pub rotation: f32,
    /// Anchor point (x, y) - origin for transformations
    pub anchor: (f32, f32),
    /// Whether scale is uniform (same for x and y)
    pub uniform_scale: bool,
}

impl Default for TransformParams {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            scale: (1.0, 1.0),
            rotation: 0.0,
            anchor: (0.0, 0.0),
            uniform_scale: true,
        }
    }
}

impl TransformParams {
    /// Create new transform parameters
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set position
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }
    
    /// Get position
    pub fn get_position(&self) -> (f32, f32) {
        self.position
    }
    
    /// Set scale
    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.scale = (x, y);
    }
    
    /// Get scale
    pub fn get_scale(&self) -> (f32, f32) {
        self.scale
    }
    
    /// Set uniform scale
    pub fn set_uniform_scale(&mut self, scale: f32) {
        self.scale = (scale, scale);
    }
    
    /// Get uniform scale (average of x and y)
    pub fn get_uniform_scale(&self) -> f32 {
        (self.scale.0 + self.scale.1) / 2.0
    }
    
    /// Set rotation in degrees
    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation.rem_euclid(360.0);
    }
    
    /// Get rotation in degrees
    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }
    
    /// Get rotation in radians
    pub fn get_rotation_rad(&self) -> f32 {
        self.rotation.to_radians()
    }
    
    /// Set anchor point
    pub fn set_anchor(&mut self, x: f32, y: f32) {
        self.anchor = (x, y);
    }
    
    /// Get anchor point
    pub fn get_anchor(&self) -> (f32, f32) {
        self.anchor
    }
    
    /// Set uniform scale flag
    pub fn set_uniform_scale_flag(&mut self, uniform: bool) {
        self.uniform_scale = uniform;
        if uniform {
            // Make scale uniform by taking average
            let avg_scale = self.get_uniform_scale();
            self.scale = (avg_scale, avg_scale);
        }
    }
    
    /// Check if scale is uniform
    pub fn is_uniform_scale(&self) -> bool {
        self.uniform_scale
    }
    
    /// Check if transform is identity (no change)
    pub fn is_identity(&self) -> bool {
        self.position == (0.0, 0.0) &&
        self.scale == (1.0, 1.0) &&
        self.rotation == 0.0 &&
        self.anchor == (0.0, 0.0)
    }
    
    /// Check if transform is active (has any non-default values)
    pub fn is_active(&self) -> bool {
        !self.is_identity()
    }
    
    /// Reset to identity
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    /// Clamp scale values to prevent issues
    pub fn clamp_scale(&mut self, min_scale: f32, max_scale: f32) {
        self.scale.0 = self.scale.0.clamp(min_scale, max_scale);
        self.scale.1 = self.scale.1.clamp(min_scale, max_scale);
    }
    
    /// Validate parameters
    pub fn validate(&mut self) {
        // Clamp rotation to 0-360 degrees
        self.rotation = self.rotation.rem_euclid(360.0);
        
        // Clamp scale to reasonable values
        self.clamp_scale(0.001, 1000.0);
        
        // Ensure uniform scale if flag is set
        if self.uniform_scale {
            let avg_scale = self.get_uniform_scale();
            self.scale = (avg_scale, avg_scale);
        }
    }
}

/// Transform matrix for 2D transformations
#[derive(Debug, Clone)]
pub struct TransformMatrix {
    /// Matrix elements in row-major order
    pub elements: [[f32; 3]; 3],
}

impl Default for TransformMatrix {
    fn default() -> Self {
        Self {
            elements: [
                [1.0, 0.0, 0.0], // Row 0
                [0.0, 1.0, 0.0], // Row 1
                [0.0, 0.0, 1.0], // Row 2
            ],
        }
    }
}

impl TransformMatrix {
    /// Create identity matrix
    pub fn identity() -> Self {
        Self::default()
    }
    
    /// Create translation matrix
    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            elements: [
                [1.0, 0.0, x],
                [0.0, 1.0, y],
                [0.0, 0.0, 1.0],
            ],
        }
    }
    
    /// Create scale matrix
    pub fn scale(x: f32, y: f32) -> Self {
        Self {
            elements: [
                [x, 0.0, 0.0],
                [0.0, y, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }
    
    /// Create rotation matrix (angle in radians)
    pub fn rotation(angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        Self {
            elements: [
                [cos_a, -sin_a, 0.0],
                [sin_a, cos_a, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }
    
    /// Create matrix from transform parameters
    pub fn from_params(params: &TransformParams) -> Self {
        // Start with identity
        let mut matrix = Self::identity();
        
        // Apply translation to move anchor to origin
        matrix = matrix * Self::translation(-params.anchor.0, -params.anchor.1);
        
        // Apply scale
        matrix = matrix * Self::scale(params.scale.0, params.scale.1);
        
        // Apply rotation
        if params.rotation != 0.0 {
            matrix = matrix * Self::rotation(params.get_rotation_rad());
        }
        
        // Apply translation to move back from anchor
        matrix = matrix * Self::translation(params.anchor.0, params.anchor.1);
        
        // Apply final position
        matrix = matrix * Self::translation(params.position.0, params.position.1);
        
        matrix
    }
    
    /// Transform a point
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        let w = self.elements[2][0] * x + self.elements[2][1] * y + self.elements[2][2];
        if w != 0.0 {
            let inv_w = 1.0 / w;
            (
                (self.elements[0][0] * x + self.elements[0][1] * y + self.elements[0][2]) * inv_w,
                (self.elements[1][0] * x + self.elements[1][1] * y + self.elements[1][2]) * inv_w,
            )
        } else {
            (x, y)
        }
    }
    
    /// Transform a vector (ignoring translation)
    pub fn transform_vector(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.elements[0][0] * x + self.elements[0][1] * y,
            self.elements[1][0] * x + self.elements[1][1] * y,
        )
    }
    
    /// Get inverse matrix
    pub fn inverse(&self) -> Option<Self> {
        // Calculate determinant of 2x2 submatrix
        let det = self.elements[0][0] * self.elements[1][1] - self.elements[0][1] * self.elements[1][0];
        
        if det.abs() < 1e-6 {
            return None; // Matrix is not invertible
        }
        
        let inv_det = 1.0 / det;
        
        Some(Self {
            elements: [
                [
                    self.elements[1][1] * inv_det,
                    -self.elements[0][1] * inv_det,
                    (self.elements[0][1] * self.elements[1][2] - self.elements[1][1] * self.elements[0][2]) * inv_det,
                ],
                [
                    -self.elements[1][0] * inv_det,
                    self.elements[0][0] * inv_det,
                    (self.elements[1][0] * self.elements[0][2] - self.elements[0][0] * self.elements[1][2]) * inv_det,
                ],
                [0.0, 0.0, 1.0],
            ],
        })
    }
}

impl std::ops::Mul for TransformMatrix {
    type Output = TransformMatrix;
    
    fn mul(self, other: TransformMatrix) -> TransformMatrix {
        let mut result = TransformMatrix::default();
        
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0;
                for k in 0..3 {
                    sum += self.elements[i][k] * other.elements[k][j];
                }
                result.elements[i][j] = sum;
            }
        }
        
        result
    }
}

/// Transform result metadata
#[derive(Debug, Clone)]
pub struct TransformResult {
    /// Original image ID
    pub original_id: Uuid,
    /// Transformed image ID
    pub transformed_id: Uuid,
    /// Transform parameters applied
    pub params: TransformParams,
    /// Transform matrix used
    pub matrix: TransformMatrix,
}
