use crate::nodes::basic::transform::{TransformParams, TransformMatrix, TransformResult};
use aether_types::ParameterValue;
use uuid::Uuid;
use log::debug;


pub struct TransformOperations {
    params: TransformParams,
}

impl TransformOperations {

    pub fn new(params: TransformParams) -> Self {
        Self { params }
    }


    pub fn apply_transform(&self, input_value: ParameterValue) -> ParameterValue {
        match input_value {
            ParameterValue::Image(input_id) => {
                debug!("Applying transform: pos={:?}, scale={:?}, rot={:.1}°, anchor={:?}",
                    self.params.position, self.params.scale, self.params.rotation, self.params.anchor);


                let matrix = TransformMatrix::from_params(&self.params);


                let transformed_id = self.perform_transform(input_id, &matrix);

                ParameterValue::Image(transformed_id)
            }
            _ => {

                ParameterValue::None
            }
        }
    }


    fn perform_transform(&self, texture_id: Uuid, matrix: &TransformMatrix) -> Uuid {
        debug!("Performing transform with matrix: {:?}", matrix.elements);


        let (data, width, height, channels) = self.load_texture_data(texture_id);


        let transformed_data = self.apply_transform_to_pixels(&data, width, height, channels, matrix);


        let transformed_id = self.upload_transformed_texture(&transformed_data, width, height, channels);

        debug!("Transform completed: {}x{} texture with ID {}", width, height, transformed_id);

        transformed_id
    }


    fn load_texture_data(&self, texture_id: Uuid) -> (Vec<u8>, usize, usize, usize) {
        debug!("Loading texture data for transform: {}", texture_id);


        let width = 1920;
        let height = 1080;
        let channels = 3;
        let data = vec![128u8; width * height * channels];

        (data, width, height, channels)
    }


    fn apply_transform_to_pixels(&self, data: &[u8], width: usize, height: usize, channels: usize, matrix: &TransformMatrix) -> Vec<u8> {
        debug!("Applying transform to {}x{} pixels", width, height);

        let mut result = vec![0u8; width * height * channels];


        for y in 0..height {
            for x in 0..width {

                let (src_x, src_y) = matrix.inverse()
                    .and_then(|inv_matrix| Some(inv_matrix.transform_point(x as f32, y as f32)))
                    .unwrap_or((x as f32, y as f32));


                let pixel = self.bilinear_interpolate(data, width, height, channels, src_x, src_y);


                let dst_offset = (y * width + x) * channels;
                for c in 0..channels {
                    result[dst_offset + c] = pixel[c];
                }
            }
        }

        result
    }


    fn bilinear_interpolate(&self, data: &[u8], width: usize, height: usize, channels: usize, x: f32, y: f32) -> Vec<u8> {

        let x = x.clamp(0.0, width as f32 - 1.0);
        let y = y.clamp(0.0, height as f32 - 1.0);


        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(width - 1);
        let y1 = (y0 + 1).min(height - 1);

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;


        let p00 = self.get_pixel(data, width, height, channels, x0, y0);
        let p10 = self.get_pixel(data, width, height, channels, x1, y0);
        let p01 = self.get_pixel(data, width, height, channels, x0, y1);
        let p11 = self.get_pixel(data, width, height, channels, x1, y1);


        let mut result = Vec::with_capacity(channels);
        for c in 0..channels {
            let val = (p00[c] as f32 * (1.0 - fx) * (1.0 - fy) +
                      p10[c] as f32 * fx * (1.0 - fy) +
                      p01[c] as f32 * (1.0 - fx) * fy +
                      p11[c] as f32 * fx * fy).round() as u8;
            result.push(val.clamp(0, 255));
        }

        result
    }


    fn get_pixel(&self, data: &[u8], width: usize, height: usize, channels: usize, x: usize, y: usize) -> Vec<u8> {
        if x >= width || y >= height {
            return vec![0u8; channels];
        }

        let offset = (y * width + x) * channels;
        data[offset..offset + channels].to_vec()
    }


    fn upload_transformed_texture(&self, data: &[u8], width: usize, height: usize, channels: usize) -> Uuid {
        debug!("Uploading transformed texture: {}x{} ({} channels)", width, height, channels);


        let texture_id = Uuid::new_v4();


        debug!("Transformed texture uploaded with ID: {}", texture_id);

        texture_id
    }


    pub fn get_params(&self) -> &TransformParams {
        &self.params
    }


    pub fn update_params(&mut self, params: TransformParams) {
        self.params = params;
    }


    pub fn get_matrix(&self) -> TransformMatrix {
        TransformMatrix::from_params(&self.params)
    }


    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        let matrix = self.get_matrix();
        matrix.transform_point(x, y)
    }


    pub fn transform_vector(&self, x: f32, y: f32) -> (f32, f32) {
        let matrix = self.get_matrix();
        matrix.transform_vector(x, y)
    }


    pub fn get_bounding_box(&self, width: f32, height: f32) -> ((f32, f32), (f32, f32)) {
        let matrix = self.get_matrix();


        let corners = [
            (0.0, 0.0),
            (width, 0.0),
            (0.0, height),
            (width, height),
        ];

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for (x, y) in corners {
            let (tx, ty) = matrix.transform_point(x, y);
            min_x = min_x.min(tx);
            min_y = min_y.min(ty);
            max_x = max_x.max(tx);
            max_y = max_y.max(ty);
        }

        ((min_x, min_y), (max_x, max_y))
    }
}
