use uuid::Uuid;


#[derive(Debug, Clone)]
pub struct TransformParams {

    pub position: (f32, f32),

    pub scale: (f32, f32),

    pub rotation: f32,

    pub anchor: (f32, f32),

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

    pub fn new() -> Self {
        Self::default()
    }


    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }


    pub fn get_position(&self) -> (f32, f32) {
        self.position
    }


    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.scale = (x, y);
    }


    pub fn get_scale(&self) -> (f32, f32) {
        self.scale
    }


    pub fn set_uniform_scale(&mut self, scale: f32) {
        self.scale = (scale, scale);
    }


    pub fn get_uniform_scale(&self) -> f32 {
        (self.scale.0 + self.scale.1) / 2.0
    }


    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation.rem_euclid(360.0);
    }


    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }


    pub fn get_rotation_rad(&self) -> f32 {
        self.rotation.to_radians()
    }


    pub fn set_anchor(&mut self, x: f32, y: f32) {
        self.anchor = (x, y);
    }


    pub fn get_anchor(&self) -> (f32, f32) {
        self.anchor
    }


    pub fn set_uniform_scale_flag(&mut self, uniform: bool) {
        self.uniform_scale = uniform;
        if uniform {

            let avg_scale = self.get_uniform_scale();
            self.scale = (avg_scale, avg_scale);
        }
    }


    pub fn is_uniform_scale(&self) -> bool {
        self.uniform_scale
    }


    pub fn is_identity(&self) -> bool {
        self.position == (0.0, 0.0) &&
        self.scale == (1.0, 1.0) &&
        self.rotation == 0.0 &&
        self.anchor == (0.0, 0.0)
    }


    pub fn is_active(&self) -> bool {
        !self.is_identity()
    }


    pub fn reset(&mut self) {
        *self = Self::default();
    }


    pub fn clamp_scale(&mut self, min_scale: f32, max_scale: f32) {
        self.scale.0 = self.scale.0.clamp(min_scale, max_scale);
        self.scale.1 = self.scale.1.clamp(min_scale, max_scale);
    }


    pub fn validate(&mut self) {

        self.rotation = self.rotation.rem_euclid(360.0);


        self.clamp_scale(0.001, 1000.0);


        if self.uniform_scale {
            let avg_scale = self.get_uniform_scale();
            self.scale = (avg_scale, avg_scale);
        }
    }
}


#[derive(Debug, Clone)]
pub struct TransformMatrix {

    pub elements: [[f32; 3]; 3],
}

impl Default for TransformMatrix {
    fn default() -> Self {
        Self {
            elements: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }
}

impl TransformMatrix {

    pub fn identity() -> Self {
        Self::default()
    }


    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            elements: [
                [1.0, 0.0, x],
                [0.0, 1.0, y],
                [0.0, 0.0, 1.0],
            ],
        }
    }


    pub fn scale(x: f32, y: f32) -> Self {
        Self {
            elements: [
                [x, 0.0, 0.0],
                [0.0, y, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }


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


    pub fn from_params(params: &TransformParams) -> Self {

        let mut matrix = Self::identity();


        matrix = matrix * Self::translation(-params.anchor.0, -params.anchor.1);


        matrix = matrix * Self::scale(params.scale.0, params.scale.1);


        if params.rotation != 0.0 {
            matrix = matrix * Self::rotation(params.get_rotation_rad());
        }


        matrix = matrix * Self::translation(params.anchor.0, params.anchor.1);


        matrix = matrix * Self::translation(params.position.0, params.position.1);

        matrix
    }


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


    pub fn transform_vector(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.elements[0][0] * x + self.elements[0][1] * y,
            self.elements[1][0] * x + self.elements[1][1] * y,
        )
    }


    pub fn inverse(&self) -> Option<Self> {

        let det = self.elements[0][0] * self.elements[1][1] - self.elements[0][1] * self.elements[1][0];

        if det.abs() < 1e-6 {
            return None;
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


#[derive(Debug, Clone)]
pub struct TransformResult {

    pub original_id: Uuid,

    pub transformed_id: Uuid,

    pub params: TransformParams,

    pub matrix: TransformMatrix,
}
