use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GradientType {
    Linear,
    Radial,
    Angular,
    Diamond,
    Conic,
    Noise,
    Fractal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    pub position: f64,
    pub color: (f64, f64, f64, f64),
    pub mid_point: f64,
}

impl GradientStop {
    pub fn new(position: f64, color: (f64, f64, f64, f64)) -> Self {
        Self {
            position: position.clamp(0.0, 1.0),
            color,
            mid_point: 0.5,
        }
    }

    pub fn with_mid_point(mut self, mid_point: f64) -> Self {
        self.mid_point = mid_point.clamp(0.0, 1.0);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.position < 0.0 || self.position > 1.0 {
            return Err("Gradient stop position must be between 0.0 and 1.0".to_string());
        }
        if self.mid_point < 0.0 || self.mid_point > 1.0 {
            return Err("Gradient stop mid-point must be between 0.0 and 1.0".to_string());
        }
        if self.color.0 < 0.0 || self.color.0 > 1.0 ||
           self.color.1 < 0.0 || self.color.1 > 1.0 ||
           self.color.2 < 0.0 || self.color.2 > 1.0 ||
           self.color.3 < 0.0 || self.color.3 > 1.0 {
            return Err("Gradient color values must be between 0.0 and 1.0".to_string());
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradientMask {
    pub properties: MaskProperties,
    pub gradient_type: GradientType,
    pub stops: Vec<GradientStop>,
    pub start_point: (f64, f64),
    pub end_point: (f64, f64),
    pub center_point: (f64, f64),
    pub radius: f64,
    pub angle: f64,
    pub repeat: bool,
    pub reverse: bool,
    pub noise_scale: f64,
    pub noise_octaves: u32,
    pub noise_persistence: f64,
    pub fractal_depth: u32,
    pub interpolation: GradientInterpolation,
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GradientInterpolation {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    Cubic,
    Exponential,
}

impl GradientMask {
    pub fn new(id: String, name: String, gradient_type: GradientType) -> Self {
        let properties = MaskProperties::new(id.clone(), name, MaskType::Gradient);
        Self {
            properties,
            gradient_type,
            stops: vec![
                GradientStop::new(0.0, (0.0, 0.0, 0.0, 0.0)),
                GradientStop::new(1.0, (1.0, 1.0, 1.0, 1.0)),
            ],
            start_point: (-50.0, 0.0),
            end_point: (50.0, 0.0),
            center_point: (0.0, 0.0),
            radius: 50.0,
            angle: 0.0,
            repeat: false,
            reverse: false,
            noise_scale: 1.0,
            noise_octaves: 4,
            noise_persistence: 0.5,
            fractal_depth: 5,
            interpolation: GradientInterpolation::Linear,
        }
    }

    pub fn with_stops(mut self, stops: Vec<GradientStop>) -> Self {
        self.stops = stops;
        self
    }

    pub fn with_start_point(mut self, x: f64, y: f64) -> Self {
        self.start_point = (x, y);
        self
    }

    pub fn with_end_point(mut self, x: f64, y: f64) -> Self {
        self.end_point = (x, y);
        self
    }

    pub fn with_center_point(mut self, x: f64, y: f64) -> Self {
        self.center_point = (x, y);
        self
    }

    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius.max(0.0);
        self
    }

    pub fn with_angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    pub fn with_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn with_reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    pub fn with_noise_params(mut self, scale: f64, octaves: u32, persistence: f64) -> Self {
        self.noise_scale = scale.max(0.1);
        self.noise_octaves = octaves.max(1);
        self.noise_persistence = persistence.clamp(0.0, 1.0);
        self
    }

    pub fn with_fractal_depth(mut self, depth: u32) -> Self {
        self.fractal_depth = depth.max(1);
        self
    }

    pub fn with_interpolation(mut self, interpolation: GradientInterpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    pub fn add_stop(&mut self, stop: GradientStop) {
        self.stops.push(stop);
        self.stops.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
        self.properties.update_modified_time();
    }

    pub fn remove_stop(&mut self, position: f64) -> bool {
        let initial_len = self.stops.len();
        self.stops.retain(|stop| (stop.position - position).abs() > f64::EPSILON);
        let removed = self.stops.len() < initial_len;
        if removed {
            self.properties.update_modified_time();
        }
        removed
    }

    pub fn clear_stops(&mut self) {
        self.stops.clear();
        self.properties.update_modified_time();
    }

    pub fn get_bounds(&self) -> Option<(f64, f64, f64, f64)> {
        let (x, y) = self.properties.position;

        match self.gradient_type {
            GradientType::Linear => {
                let (sx, sy) = self.start_point;
                let (ex, ey) = self.end_point;
                let min_x = x + sx.min(ex);
                let min_y = y + sy.min(ey);
                let max_x = x + sx.max(ex);
                let max_y = y + sy.max(ey);
                Some((min_x, min_y, max_x, max_y))
            }
            GradientType::Radial | GradientType::Diamond => {
                let r = self.radius;
                Some((x - r, y - r, x + r, y + r))
            }
            GradientType::Angular | GradientType::Conic => {
                let r = self.radius;
                Some((x - r, y - r, x + r, y + r))
            }
            GradientType::Noise | GradientType::Fractal => {


                Some((x - 100.0, y - 100.0, x + 100.0, y + 100.0))
            }
        }
    }

    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        if !self.properties.enabled {
            return false;
        }

        if let Some((min_x, min_y, max_x, max_y)) = self.get_bounds() {
            px >= min_x && px <= max_x && py >= min_y && py <= max_y
        } else {
            false
        }
    }

    pub fn evaluate_at(&self, x: f64, y: f64) -> MaskEvaluation {
        let mut value = 0.0;

        if self.properties.enabled {
            let local_x = x - self.properties.position.0;
            let local_y = y - self.properties.position.1;

            value = match self.gradient_type {
                GradientType::Linear => self.evaluate_linear(local_x, local_y),
                GradientType::Radial => self.evaluate_radial(local_x, local_y),
                GradientType::Angular => self.evaluate_angular(local_x, local_y),
                GradientType::Diamond => self.evaluate_diamond(local_x, local_y),
                GradientType::Conic => self.evaluate_conic(local_x, local_y),
                GradientType::Noise => self.evaluate_noise(local_x, local_y),
                GradientType::Fractal => self.evaluate_fractal(local_x, local_y),
            };

            if self.reverse {
                value = 1.0 - value;
            }
        }

        let mut eval = MaskEvaluation::new(self.properties.id.clone(), value, (x, y));


        eval.gradient_value = Some(value);

        eval.apply_opacity(self.properties.opacity)
            .apply_invert(self.properties.invert_mode)
    }

    fn evaluate_linear(&self, x: f64, y: f64) -> f64 {
        let (sx, sy) = self.start_point;
        let (ex, ey) = self.end_point;

        let dx = ex - sx;
        let dy = ey - sy;
        let length = (dx * dx + dy * dy).sqrt();

        if length == 0.0 {
            return 0.0;
        }

        let nx = (x - sx) * dx / length + (y - sy) * dy / length;
        let mut t = nx / length;

        if self.repeat {
            t = t - t.floor();
        }

        t.clamp(0.0, 1.0)
    }

    fn evaluate_radial(&self, x: f64, y: f64) -> f64 {
        let (cx, cy) = self.center_point;
        let dx = x - cx;
        let dy = y - cy;
        let distance = (dx * dx + dy * dy).sqrt();

        let mut t = distance / self.radius;

        if self.repeat {
            t = t - t.floor();
        }

        t.clamp(0.0, 1.0)
    }

    fn evaluate_angular(&self, x: f64, y: f64) -> f64 {
        let (cx, cy) = self.center_point;
        let dx = x - cx;
        let dy = y - cy;
        let angle = dy.atan2(dx) + std::f64::consts::PI;
        let mut t = angle / (2.0 * std::f64::consts::PI);

        if self.repeat {
            t = t - t.floor();
        }

        t.clamp(0.0, 1.0)
    }

    fn evaluate_diamond(&self, x: f64, y: f64) -> f64 {
        let (cx, cy) = self.center_point;
        let dx = (x - cx).abs();
        let dy = (y - cy).abs();
        let distance = (dx + dy) / std::f64::consts::SQRT_2;

        let mut t = distance / self.radius;

        if self.repeat {
            t = t - t.floor();
        }

        t.clamp(0.0, 1.0)
    }

    fn evaluate_conic(&self, x: f64, y: f64) -> f64 {
        let (cx, cy) = self.center_point;
        let dx = x - cx;
        let dy = y - cy;
        let angle = dy.atan2(dx) + self.angle.to_radians() + std::f64::consts::PI;
        let mut t = angle / (2.0 * std::f64::consts::PI);

        if self.repeat {
            t = t - t.floor();
        }

        t.clamp(0.0, 1.0)
    }

    fn evaluate_noise(&self, x: f64, y: f64) -> f64 {

        let nx = x * self.noise_scale;
        let ny = y * self.noise_scale;

        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..self.noise_octaves {
            value += self.simplex_noise(nx * frequency, ny * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= self.noise_persistence;
            frequency *= 2.0;
        }

        ((value / max_value) + 1.0) / 2.0
    }

    fn evaluate_fractal(&self, x: f64, y: f64) -> f64 {

        self.fractal_noise(x, y, self.fractal_depth, 1.0)
    }

    fn simplex_noise(&self, x: f64, y: f64) -> f64 {

        let s = (x + y) * 0.3660254037844386;
        let i = (x + s).floor();
        let j = (y + s).floor();

        let t = (i + j) * 0.211324865405187;
        let x0 = x - (i - t);
        let y0 = y - (j - t);


        let hash = ((i as u32).wrapping_mul(374761393).wrapping_add((j as u32).wrapping_mul(668265263))) as f64;
        (hash % 1000.0) / 500.0 - 1.0
    }

    fn fractal_noise(&self, x: f64, y: f64, depth: u32, amplitude: f64) -> f64 {
        if depth == 0 {
            0.0
        } else {
            let noise = self.simplex_noise(x, y) * amplitude;
            noise + self.fractal_noise(x * 2.0, y * 2.0, depth - 1, amplitude * 0.5)
        }
    }

    fn interpolate_gradient(&self, t: f64) -> (f64, f64, f64, f64) {
        if self.stops.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        if self.stops.len() == 1 {
            return self.stops[0].color;
        }


        let mut lower_stop = &self.stops[0];
        let mut upper_stop = &self.stops[self.stops.len() - 1];

        for i in 0..self.stops.len() - 1 {
            if self.stops[i].position <= t && self.stops[i + 1].position >= t {
                lower_stop = &self.stops[i];
                upper_stop = &self.stops[i + 1];
                break;
            }
        }

        if lower_stop.position == upper_stop.position {
            return lower_stop.color;
        }

        let local_t = (t - lower_stop.position) / (upper_stop.position - lower_stop.position);
        let eased_t = self.apply_easing(local_t, lower_stop.mid_point);

        (
            lower_stop.color.0 + (upper_stop.color.0 - lower_stop.color.0) * eased_t,
            lower_stop.color.1 + (upper_stop.color.1 - lower_stop.color.1) * eased_t,
            lower_stop.color.2 + (upper_stop.color.2 - lower_stop.color.2) * eased_t,
            lower_stop.color.3 + (upper_stop.color.3 - lower_stop.color.3) * eased_t,
        )
    }

    fn apply_easing(&self, t: f64, mid_point: f64) -> f64 {
        let adjusted_t = if t < mid_point {
            t / mid_point
        } else {
            (t - mid_point) / (1.0 - mid_point)
        };

        let eased_t = match self.interpolation {
            GradientInterpolation::Linear => adjusted_t,
            GradientInterpolation::Ease => {
                if adjusted_t < 0.5 {
                    2.0 * adjusted_t * adjusted_t
                } else {
                    1.0 - 2.0 * (1.0 - adjusted_t) * (1.0 - adjusted_t)
                }
            }
            GradientInterpolation::EaseIn => adjusted_t * adjusted_t,
            GradientInterpolation::EaseOut => 1.0 - (1.0 - adjusted_t) * (1.0 - adjusted_t),
            GradientInterpolation::EaseInOut => {
                if adjusted_t < 0.5 {
                    2.0 * adjusted_t * adjusted_t
                } else {
                    1.0 - 2.0 * (1.0 - adjusted_t) * (1.0 - adjusted_t)
                }
            }
            GradientInterpolation::Cubic => adjusted_t * adjusted_t * adjusted_t,
            GradientInterpolation::Exponential => {
                if adjusted_t <= 0.0 {
                    0.0
                } else {
                    2.0f64.powf(10.0 * (adjusted_t - 1.0))
                }
            }
        };

        if t < mid_point {
            eased_t * mid_point
        } else {
            mid_point + eased_t * (1.0 - mid_point)
        }
    }

    pub fn create_linear_gradient(id: String, name: String, start: (f64, f64), end: (f64, f64)) -> Self {
        Self::new(id, name, GradientType::Linear)
            .with_start_point(start.0, start.1)
            .with_end_point(end.0, end.1)
    }

    pub fn create_radial_gradient(id: String, name: String, center: (f64, f64), radius: f64) -> Self {
        Self::new(id, name, GradientType::Radial)
            .with_center_point(center.0, center.1)
            .with_radius(radius)
    }

    pub fn create_angular_gradient(id: String, name: String, center: (f64, f64), radius: f64, angle: f64) -> Self {
        Self::new(id, name, GradientType::Angular)
            .with_center_point(center.0, center.1)
            .with_radius(radius)
            .with_angle(angle)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.properties.validate()?;

        if self.stops.len() < 2 {
            return Err("Gradient must have at least 2 stops".to_string());
        }

        if self.radius < 0.0 {
            return Err("Gradient radius must be non-negative".to_string());
        }

        if self.noise_scale <= 0.0 {
            return Err("Noise scale must be positive".to_string());
        }

        if self.noise_octaves == 0 {
            return Err("Noise octaves must be at least 1".to_string());
        }

        if self.noise_persistence < 0.0 || self.noise_persistence > 1.0 {
            return Err("Noise persistence must be between 0.0 and 1.0".to_string());
        }

        if self.fractal_depth == 0 {
            return Err("Fractal depth must be at least 1".to_string());
        }

        for (i, stop) in self.stops.iter().enumerate() {
            stop.validate().map_err(|e| format!("Stop {}: {}", i, e))?;
        }

        Ok(())
    }
}

impl Default for GradientMask {
    fn default() -> Self {
        Self::new("default".to_string(), "Default Gradient".to_string(), GradientType::Linear)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_mask_creation() {
        let mask = GradientMask::new("grad1".to_string(), "Test Gradient".to_string(), GradientType::Linear);

        assert_eq!(mask.properties.id, "grad1");
        assert_eq!(mask.properties.name, "Test Gradient");
        assert_eq!(mask.gradient_type, GradientType::Linear);
        assert_eq!(mask.stops.len(), 2);
        assert_eq!(mask.start_point, (-50.0, 0.0));
        assert_eq!(mask.end_point, (50.0, 0.0));
    }

    #[test]
    fn test_gradient_stop_creation() {
        let stop = GradientStop::new(0.5, (1.0, 0.0, 0.0, 0.8));

        assert_eq!(stop.position, 0.5);
        assert_eq!(stop.color, (1.0, 0.0, 0.0, 0.8));
        assert_eq!(stop.mid_point, 0.5);
    }

    #[test]
    fn test_gradient_stop_builder() {
        let stop = GradientStop::new(0.3, (0.5, 0.7, 0.2, 1.0))
            .with_mid_point(0.7);

        assert_eq!(stop.position, 0.3);
        assert_eq!(stop.mid_point, 0.7);
    }

    #[test]
    fn test_gradient_mask_builder() {
        let stops = vec![
            GradientStop::new(0.0, (1.0, 0.0, 0.0, 0.0)),
            GradientStop::new(0.5, (0.0, 1.0, 0.0, 0.5)),
            GradientStop::new(1.0, (0.0, 0.0, 1.0, 1.0)),
        ];

        let mask = GradientMask::new("grad1".to_string(), "Test".to_string(), GradientType::Radial)
            .with_stops(stops.clone())
            .with_center_point(100.0, 100.0)
            .with_radius(75.0)
            .with_repeat(true)
            .with_reverse(true)
            .with_interpolation(GradientInterpolation::Ease);

        assert_eq!(mask.stops, stops);
        assert_eq!(mask.center_point, (100.0, 100.0));
        assert_eq!(mask.radius, 75.0);
        assert!(mask.repeat);
        assert!(mask.reverse);
        assert_eq!(mask.interpolation, GradientInterpolation::Ease);
    }

    #[test]
    fn test_linear_gradient_evaluation() {
        let mask = GradientMask::create_linear_gradient(
            "linear".to_string(),
            "Linear".to_string(),
            (-50.0, 0.0),
            (50.0, 0.0)
        );

        let eval_start = mask.evaluate_at(-50.0, 0.0);
        assert_eq!(eval_start.value, 0.0);

        let eval_end = mask.evaluate_at(50.0, 0.0);
        assert_eq!(eval_end.value, 1.0);

        let eval_middle = mask.evaluate_at(0.0, 0.0);
        assert_eq!(eval_middle.value, 0.5);
    }

    #[test]
    fn test_radial_gradient_evaluation() {
        let mask = GradientMask::create_radial_gradient(
            "radial".to_string(),
            "Radial".to_string(),
            (0.0, 0.0),
            50.0
        );

        let eval_center = mask.evaluate_at(0.0, 0.0);
        assert_eq!(eval_center.value, 0.0);

        let eval_edge = mask.evaluate_at(50.0, 0.0);
        assert_eq!(eval_edge.value, 1.0);

        let eval_outside = mask.evaluate_at(75.0, 0.0);
        assert_eq!(eval_outside.value, 1.0);
    }

    #[test]
    fn test_angular_gradient_evaluation() {
        let mask = GradientMask::create_angular_gradient(
            "angular".to_string(),
            "Angular".to_string(),
            (0.0, 0.0),
            50.0,
            0.0
        );

        let eval_right = mask.evaluate_at(50.0, 0.0);
        let eval_top = mask.evaluate_at(0.0, 50.0);
        let eval_left = mask.evaluate_at(-50.0, 0.0);


        assert!(eval_right.value != eval_top.value);
        assert!(eval_top.value != eval_left.value);
        assert!(eval_left.value != eval_right.value);
    }

    #[test]
    fn test_gradient_stops_management() {
        let mut mask = GradientMask::default();

        assert_eq!(mask.stops.len(), 2);

        mask.add_stop(GradientStop::new(0.25, (1.0, 1.0, 0.0, 0.5)));
        assert_eq!(mask.stops.len(), 3);

        assert!(mask.remove_stop(0.25));
        assert_eq!(mask.stops.len(), 2);
        assert!(!mask.remove_stop(0.25));

        mask.clear_stops();
        assert!(mask.stops.is_empty());
    }

    #[test]
    fn test_gradient_validation() {
        let valid_mask = GradientMask::default();
        assert!(valid_mask.validate().is_ok());

        let mut invalid_mask = valid_mask.clone();
        invalid_mask.stops.clear();
        assert!(invalid_mask.validate().is_err());

        invalid_mask.stops = vec![GradientStop::new(0.0, (1.0, 0.0, 0.0, 1.0))];
        assert!(invalid_mask.validate().is_err());

        invalid_mask.stops.push(GradientStop::new(1.0, (0.0, 1.0, 0.0, 1.0)));
        invalid_mask.radius = -1.0;
        assert!(invalid_mask.validate().is_err());

        invalid_mask.radius = 50.0;
        invalid_mask.noise_scale = 0.0;
        assert!(invalid_mask.validate().is_err());
    }

    #[test]
    fn test_gradient_interpolation() {
        let mask = GradientMask::new("test".to_string(), "Test".to_string(), GradientType::Linear)
            .with_interpolation(GradientInterpolation::Ease);

        let color = mask.interpolate_gradient(0.5);

        assert!(color.3 > 0.0 && color.3 < 1.0);
    }

    #[test]
    fn test_noise_gradient() {
        let mask = GradientMask::new("noise".to_string(), "Noise".to_string(), GradientType::Noise)
            .with_noise_params(0.1, 4, 0.5);

        let eval1 = mask.evaluate_at(0.0, 0.0);
        let eval2 = mask.evaluate_at(10.0, 10.0);


        assert!(eval1.value != eval2.value);
        assert!(eval1.value >= 0.0 && eval1.value <= 1.0);
        assert!(eval2.value >= 0.0 && eval2.value <= 1.0);
    }

    #[test]
    fn test_repeat_gradient() {
        let mask = GradientMask::create_linear_gradient(
            "repeat".to_string(),
            "Repeat".to_string(),
            (-50.0, 0.0),
            (50.0, 0.0)
        );
        mask.repeat = true;

        let eval_inside = mask.evaluate_at(0.0, 0.0);
        let eval_outside = mask.evaluate_at(150.0, 0.0);


        assert!(eval_outside.value >= 0.0 && eval_outside.value <= 1.0);
    }

    #[test]
    fn test_reverse_gradient() {
        let mask = GradientMask::create_linear_gradient(
            "reverse".to_string(),
            "Reverse".to_string(),
            (-50.0, 0.0),
            (50.0, 0.0)
        );
        mask.reverse = true;

        let eval_start = mask.evaluate_at(-50.0, 0.0);
        let eval_end = mask.evaluate_at(50.0, 0.0);


        assert_eq!(eval_start.value, 1.0);
        assert_eq!(eval_end.value, 0.0);
    }

    #[test]
    fn test_gradient_bounds() {
        let linear_mask = GradientMask::create_linear_gradient(
            "linear".to_string(),
            "Linear".to_string(),
            (-50.0, 0.0),
            (50.0, 0.0)
        );

        let bounds = linear_mask.get_bounds();
        assert!(bounds.is_some());
        let (min_x, min_y, max_x, max_y) = bounds.unwrap();
        assert_eq!(min_x, -50.0);
        assert_eq!(max_x, 50.0);
        assert_eq!(min_y, 0.0);
        assert_eq!(max_y, 0.0);

        let radial_mask = GradientMask::create_radial_gradient(
            "radial".to_string(),
            "Radial".to_string(),
            (0.0, 0.0),
            50.0
        );

        let bounds = radial_mask.get_bounds();
        assert!(bounds.is_some());
        let (min_x, min_y, max_x, max_y) = bounds.unwrap();
        assert_eq!(min_x, -50.0);
        assert_eq!(max_x, 50.0);
        assert_eq!(min_y, -50.0);
        assert_eq!(max_y, 50.0);
    }
}
