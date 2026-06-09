use uuid::Uuid;


#[derive(Debug, Clone)]
pub struct BlurResult {

    pub original_id: Uuid,

    pub blurred_id: Uuid,

    pub blur_type: BlurType,

    pub radius: f32,

    pub kernel_size: usize,

    pub angle: f32,

    pub iterations: u32,
}


#[derive(Debug, Clone, PartialEq)]
pub enum BlurType {

    Gaussian,

    Box,

    Motion,

    Radial,

    Zoom,
}


#[derive(Debug, Clone)]
pub struct BlurParams {

    pub radius: f32,

    pub blur_type: BlurType,

    pub iterations: u32,

    pub directional: bool,

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

    pub fn new() -> Self {
        Self::default()
    }


    pub fn validate(&mut self) {
        self.radius = self.radius.clamp(0.0, 100.0);
        self.iterations = self.iterations.clamp(1, 10);
        self.angle = self.angle.rem_euclid(360.0);
    }


    pub fn is_active(&self) -> bool {
        self.radius > 0.0
    }
}


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
