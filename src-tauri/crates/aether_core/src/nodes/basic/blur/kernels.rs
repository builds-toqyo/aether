use crate::nodes::basic::blur::NextOdd;
use log::debug;


pub struct BlurKernels {

    radius: f32,
}

impl BlurKernels {

    pub fn new(radius: f32) -> Self {
        Self { radius }
    }


    pub fn generate_gaussian_kernel(&self, size: usize) -> Vec<f32> {
        let mut kernel = Vec::with_capacity(size);
        let center = size as f32 / 2.0 - 0.5;
        let sigma = self.radius / 3.0;

        let mut sum = 0.0;
        for i in 0..size {
            let x = i as f32 - center;
            let value = (-x * x / (2.0 * sigma * sigma)).exp();
            kernel.push(value);
            sum += value;
        }


        for value in kernel.iter_mut() {
            *value /= sum;
        }

        debug!("Generated {}x1 Gaussian kernel with sigma={:.2}", size, sigma);

        kernel
    }


    pub fn generate_gaussian_kernel_2d(&self, size: usize) -> Vec<Vec<f32>> {
        let mut kernel = Vec::with_capacity(size);
        let center = size as f32 / 2.0 - 0.5;
        let sigma = self.radius / 3.0;


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


    pub fn generate_box_kernel(&self, size: usize) -> Vec<f32> {
        let value = 1.0 / size as f32;
        let kernel = vec![value; size];

        debug!("Generated {}x1 box kernel", size);

        kernel
    }


    pub fn generate_box_kernel_2d(&self, size: usize) -> Vec<Vec<f32>> {
        let value = 1.0 / (size * size) as f32;
        let kernel = vec![vec![value; size]; size];

        debug!("Generated {}x{} 2D box kernel", size, size);

        kernel
    }


    pub fn generate_motion_blur_kernel(&self, size: usize, angle: f32) -> Vec<f32> {
        let mut kernel = vec![0.0; size];
        let center = size as f32 / 2.0 - 0.5;
        let angle_rad = angle.to_radians();


        let half_length = self.radius;
        let dx = half_length * angle_rad.cos();
        let dy = half_length * angle_rad.sin();


        let samples = size;
        for i in 0..samples {
            let t = (i as f32 / (samples - 1) as f32) * 2.0 - 1.0;
            let x = center + t * dx;
            let _y = center + t * dy;


            let kernel_x = (x as usize).min(size - 1);
            kernel[kernel_x] += 1.0;
        }


        let sum: f32 = kernel.iter().sum();
        if sum > 0.0 {
            for value in kernel.iter_mut() {
                *value /= sum;
            }
        }

        debug!("Generated {}x1 motion blur kernel with angle={:.1}°", size, angle);

        kernel
    }


    pub fn generate_radial_kernel(&self, samples: usize) -> Vec<(f32, f32, f32)> {
        let mut kernel = Vec::with_capacity(samples);

        for i in 0..samples {
            let t = i as f32 / (samples - 1) as f32;
            let radius = t * self.radius;
            let weight = 1.0 / samples as f32;


            kernel.push((radius, weight, 0.0));
        }

        debug!("Generated radial blur kernel with {} samples", samples);

        kernel
    }


    pub fn generate_zoom_kernel(&self, samples: usize) -> Vec<(f32, f32, f32)> {
        let mut kernel = Vec::with_capacity(samples);

        for i in 0..samples {
            let t = i as f32 / (samples - 1) as f32;
            let scale = 1.0 + t * self.radius * 0.1;
            let weight = 1.0 / samples as f32;


            kernel.push((scale, weight, 0.0));
        }

        debug!("Generated zoom blur kernel with {} samples", samples);

        kernel
    }


    pub fn generate_separable_gaussian_kernels(&self, size: usize) -> (Vec<f32>, Vec<f32>) {
        let horizontal = self.generate_gaussian_kernel(size);
        let vertical = horizontal.clone();

        debug!("Generated separable Gaussian kernels: {}x1 and {}x1", size, size);

        (horizontal, vertical)
    }


    pub fn calculate_optimal_size(&self) -> usize {

        let size = (self.radius * 3.0) as usize;
        size.next_odd().max(3).min(51)
    }


    pub fn get_radius(&self) -> f32 {
        self.radius
    }


    pub fn update_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
}
