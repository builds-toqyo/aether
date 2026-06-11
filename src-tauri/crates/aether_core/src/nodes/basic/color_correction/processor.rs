use crate::nodes::basic::color_correction::{ColorCorrectionParams, CorrectedImage};
use uuid::Uuid;
use log::debug;


pub struct ColorProcessor {
    parameters: ColorCorrectionParams,
}

impl ColorProcessor {

    pub fn new(parameters: ColorCorrectionParams) -> Self {
        Self { parameters }
    }


    pub fn get_parameters(&self) -> &ColorCorrectionParams {
        &self.parameters
    }


    pub fn set_parameters(&mut self, parameters: ColorCorrectionParams) {
        self.parameters = parameters;
    }


    pub fn apply_corrections(&self, pixels: &[(f32, f32, f32)], corrected_id: Uuid) -> CorrectedImage {
        debug!("Applying color corrections to {} pixels", pixels.len());

        let _corrected_pixels: Vec<(f32, f32, f32)> = pixels
            .iter()
            .map(|&(r, g, b)| self.apply_pixel_correction(r, g, b))
            .collect();

        CorrectedImage {
            original_id: Uuid::new_v4(),
            corrected_id,
            brightness: self.parameters.brightness,
            contrast: self.parameters.contrast,
            saturation: self.parameters.saturation,
            gamma: self.parameters.gamma,
            temperature: self.parameters.temperature,
            tint: self.parameters.tint,
            hue: self.parameters.hue,
            lift: self.parameters.lift,
            gamma_gain: self.parameters.gamma_gain,
            gain: self.parameters.gain,
        }
    }


    fn apply_pixel_correction(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {

        let (r, g, b) = self.apply_white_balance(r, g, b);


        let (r, g, b) = self.apply_lift_gamma_gain(r, g, b);


        let (r, g, b) = self.apply_brightness_contrast(r, g, b);


        let (r, g, b) = self.apply_gamma_correction(r, g, b);


        let (r, g, b) = self.apply_hue_saturation(r, g, b);


        (
            r.clamp(0.0, 1.0),
            g.clamp(0.0, 1.0),
            b.clamp(0.0, 1.0)
        )
    }


    fn apply_white_balance(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let (wb_r, wb_g, wb_b) = self.temperature_to_rgb(self.parameters.temperature);


        let r = r * wb_r;
        let g = g * wb_g;
        let b = b * wb_b;


        let tint_factor = self.parameters.tint / 100.0;
        let r = r * (1.0 - tint_factor * 0.5);
        let g = g * (1.0 + tint_factor);
        let b = b * (1.0 - tint_factor * 0.5);

        (r, g, b)
    }


    fn apply_lift_gamma_gain(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {

        let r = r + self.parameters.lift;
        let g = g + self.parameters.lift;
        let b = b + self.parameters.lift;


        if self.parameters.gamma_gain > 0.0 {
            let gamma = 1.0 / self.parameters.gamma_gain;
            let _r = r.powf(gamma);
            let _g = g.powf(gamma);
            let _b = b.powf(gamma);
        }


        let r = r * self.parameters.gain;
        let g = g * self.parameters.gain;
        let b = b * self.parameters.gain;

        (r, g, b)
    }


    fn apply_brightness_contrast(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {

        let r = r + self.parameters.brightness;
        let g = g + self.parameters.brightness;
        let b = b + self.parameters.brightness;


        let r = (r - 0.5) * self.parameters.contrast + 0.5;
        let g = (g - 0.5) * self.parameters.contrast + 0.5;
        let b = (b - 0.5) * self.parameters.contrast + 0.5;

        (r, g, b)
    }


    fn apply_gamma_correction(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        if self.parameters.gamma > 0.0 {
            let gamma = 1.0 / self.parameters.gamma;
            (
                r.powf(gamma),
                g.powf(gamma),
                b.powf(gamma)
            )
        } else {
            (r, g, b)
        }
    }


    fn apply_hue_saturation(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {

        let (h, s, l) = self.rgb_to_hsl(r, g, b);


        let h = (h + self.parameters.hue / 360.0) % 1.0;
        if h < 0.0 {
            let _h = h + 1.0;
        }


        let s = (s * self.parameters.saturation).clamp(0.0, 1.0);


        self.hsl_to_rgb(h, s, l)
    }


    fn rgb_to_hsl(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        if max == min {
            (0.0, 0.0, l)
        } else {
            let d = max - min;
            let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };

            let h = match max {
                x if x == r => ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0,
                x if x == g => ((b - r) / d + 2.0) / 6.0,
                x if x == b => ((r - g) / d + 4.0) / 6.0,
                _ => 0.0,
            };

            (h, s, l)
        }
    }


    fn hsl_to_rgb(&self, h: f32, s: f32, l: f32) -> (f32, f32, f32) {
        if s == 0.0 {
            (l, l, l)
        } else {
            let q = if l < 0.5 {
                l * (1.0 + s)
            } else {
                l + s - l * s
            };
            let p = 2.0 * l - q;

            let r = self.hue_to_rgb(p, q, h + 1.0 / 3.0);
            let g = self.hue_to_rgb(p, q, h);
            let b = self.hue_to_rgb(p, q, h - 1.0 / 3.0);

            (r, g, b)
        }
    }


    fn hue_to_rgb(&self, p: f32, q: f32, t: f32) -> f32 {
        if t < 0.0 {
            let _t = t + 1.0;
        }
        if t > 1.0 {
            let _t = t - 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 1.0 / 2.0 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    }


    fn temperature_to_rgb(&self, temperature: f32) -> (f32, f32, f32) {

        let temp = temperature / 100.0;

        let (r, g, b) = if temp <= 66.0 {

            let r = 1.0;
            let g = if temp <= 19.0 {
                0.0
            } else {
                let g = temp - 19.0;
                let g = 0.390081579 * g.powf(-0.254133);
                g.clamp(0.0, 1.0)
            };
            let b = if temp <= 19.0 {
                0.0
            } else if temp <= 66.0 {
                let b = temp - 10.0;
                let b = 0.543206769 * b.powf(-0.231350);
                b.clamp(0.0, 1.0)
            } else {
                1.0
            };
            (r, g, b)
        } else {

            let r = if temp >= 400.0 {
                0.4355774 * ((temp - 400.0) / 100.0).powf(-0.114912)
            } else {
                1.0
            };
            let g = if temp >= 400.0 {
                0.403308 * ((temp - 400.0) / 100.0).powf(-0.099842)
            } else {
                1.0
            };
            let b = if temp >= 400.0 {
                1.0
            } else {
                0.543206769 * ((temp - 10.0) / 100.0).powf(-0.231350)
            };
            (r, g, b)
        };


        let max_val = r.max(g).max(b);
        if max_val > 0.0 {
            (r / max_val, g / max_val, b / max_val)
        } else {
            (1.0, 1.0, 1.0)
        }
    }
}

impl Default for ColorProcessor {
    fn default() -> Self {
        Self::new(ColorCorrectionParams::default())
    }
}
