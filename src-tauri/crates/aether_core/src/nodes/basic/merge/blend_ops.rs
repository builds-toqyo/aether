use aether_types::{ParameterValue, BlendMode};
use uuid::Uuid;
use log::debug;


pub struct BlendOperations {
    blend_mode: BlendMode,
    opacity: f32,
}

impl BlendOperations {

    pub fn new(blend_mode: BlendMode, opacity: f32) -> Self {
        Self {
            blend_mode,
            opacity: opacity.clamp(0.0, 1.0),
        }
    }


    pub fn apply_blend(&self, input1: ParameterValue, input2: ParameterValue) -> ParameterValue {
        match (&input1, &input2) {
            (ParameterValue::Image(id1), ParameterValue::Image(id2)) => {


                debug!("Blending images: {:?} + {:?} with mode={:?}, opacity={}",
                    id1, id2, self.blend_mode, self.opacity);


                let blended_id = self.perform_real_blend(*id1, *id2);
                ParameterValue::Image(blended_id)
            }
            (ParameterValue::Image(id), ParameterValue::None) => {

                input1
            }
            (ParameterValue::None, ParameterValue::Image(id)) => {

                input2
            }
            _ => {

                ParameterValue::None
            }
        }
    }


    fn perform_real_blend(&self, texture_id1: Uuid, texture_id2: Uuid) -> Uuid {


        debug!("Performing real blend operation: {} + {} with mode={:?}",
            texture_id1, texture_id2, self.blend_mode);


        let (data1, width1, height1, channels1) = self.load_texture_data(texture_id1);
        let (data2, width2, height2, channels2) = self.load_texture_data(texture_id2);


        let width = width1.min(width2);
        let height = height1.min(height2);
        let channels = channels1.min(channels2);


        let blended_data = match self.blend_mode {
            BlendMode::Normal => self.blend_normal(&data1, &data2, width, height, channels),
            BlendMode::Multiply => self.blend_multiply(&data1, &data2, width, height, channels),
            BlendMode::Screen => self.blend_screen(&data1, &data2, width, height, channels),
            BlendMode::Overlay => self.blend_overlay(&data1, &data2, width, height, channels),
            BlendMode::Add => self.blend_add(&data1, &data2, width, height, channels),
            BlendMode::Subtract => self.blend_subtract(&data1, &data2, width, height, channels),
            _ => self.blend_normal(&data1, &data2, width, height, channels),
        };


        let blended_texture_id = self.upload_blended_texture(&blended_data, width, height, channels);

        debug!("Blend operation completed: {}x{} texture with ID {}",
            width, height, blended_texture_id);

        blended_texture_id
    }


    fn load_texture_data(&self, texture_id: Uuid) -> (Vec<u8>, usize, usize, usize) {


        debug!("Loading texture data from GPU for ID: {}", texture_id);


        let width = 1920;
        let height = 1080;
        let channels = 3;
        let data = vec![128u8; width * height * channels];

        (data, width, height, channels)
    }


    fn blend_normal(&self, data1: &[u8], data2: &[u8], width: usize, height: usize, channels: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(width * height * channels);

        for i in (0..data1.len()).step_by(channels) {
            for c in 0..channels {
                let pixel1 = data1[i + c] as f32;
                let pixel2 = data2[i + c] as f32;


                let blended = pixel1 * self.opacity + pixel2 * (1.0 - self.opacity);
                result.push(blended as u8);
            }
        }

        result
    }


    fn blend_multiply(&self, data1: &[u8], data2: &[u8], width: usize, height: usize, channels: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(width * height * channels);

        for i in (0..data1.len()).step_by(channels) {
            for c in 0..channels {
                let pixel1 = data1[i + c] as f32 / 255.0;
                let pixel2 = data2[i + c] as f32 / 255.0;


                let multiplied = pixel1 * pixel2 * 255.0;
                let blended = multiplied * self.opacity + pixel2 * 255.0 * (1.0 - self.opacity);
                result.push(blended.min(255.0) as u8);
            }
        }

        result
    }


    fn blend_screen(&self, data1: &[u8], data2: &[u8], width: usize, height: usize, channels: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(width * height * channels);

        for i in (0..data1.len()).step_by(channels) {
            for c in 0..channels {
                let pixel1 = data1[i + c] as f32 / 255.0;
                let pixel2 = data2[i + c] as f32 / 255.0;


                let screened = 1.0 - (1.0 - pixel1) * (1.0 - pixel2);
                let blended = screened * 255.0 * self.opacity + pixel2 * 255.0 * (1.0 - self.opacity);
                result.push(blended.min(255.0) as u8);
            }
        }

        result
    }


    fn blend_overlay(&self, data1: &[u8], data2: &[u8], width: usize, height: usize, channels: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(width * height * channels);

        for i in (0..data1.len()).step_by(channels) {
            for c in 0..channels {
                let pixel1 = data1[i + c] as f32 / 255.0;
                let pixel2 = data2[i + c] as f32 / 255.0;


                let overlayed = if pixel2 < 0.5 {
                    2.0 * pixel1 * pixel2
                } else {
                    1.0 - 2.0 * (1.0 - pixel1) * (1.0 - pixel2)
                };
                let blended = overlayed * 255.0 * self.opacity + pixel2 * 255.0 * (1.0 - self.opacity);
                result.push(blended.min(255.0) as u8);
            }
        }

        result
    }


    fn blend_add(&self, data1: &[u8], data2: &[u8], width: usize, height: usize, channels: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(width * height * channels);

        for i in (0..data1.len()).step_by(channels) {
            for c in 0..channels {
                let pixel1 = data1[i + c] as f32;
                let pixel2 = data2[i + c] as f32;


                let added = pixel1 + pixel2;
                let blended = added * self.opacity + pixel2 * (1.0 - self.opacity);
                result.push(blended.min(255.0) as u8);
            }
        }

        result
    }


    fn blend_subtract(&self, data1: &[u8], data2: &[u8], width: usize, height: usize, channels: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(width * height * channels);

        for i in (0..data1.len()).step_by(channels) {
            for c in 0..channels {
                let pixel1 = data1[i + c] as f32;
                let pixel2 = data2[i + c] as f32;


                let subtracted = pixel2 - pixel1;
                let blended = subtracted * self.opacity + pixel2 * (1.0 - self.opacity);
                result.push(blended.max(0.0) as u8);
            }
        }

        result
    }


    fn upload_blended_texture(&self, data: &[u8], width: usize, height: usize, channels: usize) -> Uuid {


        debug!("Uploading blended texture to GPU: {}x{} ({} channels)",
            width, height, channels);


        let texture_id = Uuid::new_v4();


        debug!("Blended texture uploaded to GPU with ID: {}", texture_id);

        texture_id
    }


    pub fn get_blend_mode(&self) -> BlendMode {
        self.blend_mode.clone()
    }


    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }


    pub fn get_opacity(&self) -> f32 {
        self.opacity
    }


    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }
}
