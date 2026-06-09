use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CurveType {
    Linear,
    Bezier,
    EaseIn,
    EaseOut,
    EaseInOut,
    Step,
    Custom,
}

impl CurveType {
    pub fn name(&self) -> &'static str {
        match self {
            CurveType::Linear => "Linear",
            CurveType::Bezier => "Bezier",
            CurveType::EaseIn => "Ease In",
            CurveType::EaseOut => "Ease Out",
            CurveType::EaseInOut => "Ease In Out",
            CurveType::Step => "Step",
            CurveType::Custom => "Custom",
        }
    }
}

impl Default for CurveType {
    fn default() -> Self {
        CurveType::Linear
    }
}

impl fmt::Display for CurveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BezierControlPoint {
    pub position: (f64, f64),
    pub handle1: Option<(f64, f64)>,
    pub handle2: Option<(f64, f64)>,
}

impl BezierControlPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            position: (x, y),
            handle1: None,
            handle2: None,
        }
    }

    pub fn with_handles(x: f64, y: f64, handle1: (f64, f64), handle2: (f64, f64)) -> Self {
        Self {
            position: (x, y),
            handle1: Some(handle1),
            handle2: Some(handle2),
        }
    }

    pub fn set_handle1(&mut self, handle: (f64, f64)) {
        self.handle1 = Some(handle);
    }

    pub fn set_handle2(&mut self, handle: (f64, f64)) {
        self.handle2 = Some(handle);
    }

    pub fn clear_handles(&mut self) {
        self.handle1 = None;
        self.handle2 = None;
    }

    pub fn has_handles(&self) -> bool {
        self.handle1.is_some() || self.handle2.is_some()
    }
}

impl fmt::Display for BezierControlPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.handle1, &self.handle2) {
            (Some(h1), Some(h2)) => {
                write!(f, "Point({}, {}) with handles({}, {}) and ({}, {})",
                       self.position.0, self.position.1, h1.0, h1.1, h2.0, h2.1)
            }
            (Some(h1), None) => {
                write!(f, "Point({}, {}) with handle({}, {})",
                       self.position.0, self.position.1, h1.0, h1.1)
            }
            (None, Some(h2)) => {
                write!(f, "Point({}, {}) with handle({}, {})",
                       self.position.0, self.position.1, h2.0, h2.1)
            }
            (None, None) => {
                write!(f, "Point({}, {})",
                       self.position.0, self.position.1)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationCurve {
    pub curve_type: CurveType,
    pub control_points: Vec<BezierControlPoint>,
    pub tension: f64,
    pub bias: f64,
}

impl AnimationCurve {
    pub fn new(curve_type: CurveType) -> Self {
        Self {
            curve_type,
            control_points: Vec::new(),
            tension: 0.5,
            bias: 0.0,
        }
    }

    pub fn linear() -> Self {
        Self::new(CurveType::Linear)
    }

    pub fn bezier(control_points: Vec<BezierControlPoint>) -> Self {
        let mut curve = Self::new(CurveType::Bezier);
        curve.control_points = control_points;
        curve
    }

    pub fn ease_in() -> Self {
        Self::new(CurveType::EaseIn)
    }

    pub fn ease_out() -> Self {
        Self::new(CurveType::EaseOut)
    }

    pub fn ease_in_out() -> Self {
        Self::new(CurveType::EaseInOut)
    }

    pub fn step() -> Self {
        Self::new(CurveType::Step)
    }

    pub fn custom(control_points: Vec<BezierControlPoint>) -> Self {
        let mut curve = Self::new(CurveType::Custom);
        curve.control_points = control_points;
        curve
    }

    pub fn with_tension(mut self, tension: f64) -> Self {
        self.tension = tension.clamp(0.0, 1.0);
        self
    }

    pub fn with_bias(mut self, bias: f64) -> Self {
        self.bias = bias.clamp(-1.0, 1.0);
        self
    }

    pub fn add_control_point(&mut self, point: BezierControlPoint) {
        self.control_points.push(point);
    }

    pub fn remove_control_point(&mut self, index: usize) -> Option<BezierControlPoint> {
        if index < self.control_points.len() {
            Some(self.control_points.remove(index))
        } else {
            None
        }
    }

    pub fn get_control_point(&self, index: usize) -> Option<&BezierControlPoint> {
        self.control_points.get(index)
    }

    pub fn get_control_point_mut(&mut self, index: usize) -> Option<&mut BezierControlPoint> {
        self.control_points.get_mut(index)
    }

    pub fn clear_control_points(&mut self) {
        self.control_points.clear();
    }

    pub fn control_point_count(&self) -> usize {
        self.control_points.len()
    }

    pub fn validate(&self) -> Result<(), String> {
        match self.curve_type {
            CurveType::Bezier | CurveType::Custom => {
                if self.control_points.len() < 2 {
                    return Err(format!("{} curve requires at least 2 control points", self.curve_type.name()));
                }

                for i in 1..self.control_points.len() {
                    if self.control_points[i].position.0 < self.control_points[i - 1].position.0 {
                        return Err("Control points must be ordered by x position".to_string());
                    }
                }
            }
            _ => {
                if !self.control_points.is_empty() {
                    return Err(format!("{} curve should not have control points", self.curve_type.name()));
                }
            }
        }

        Ok(())
    }

    pub fn sample(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);

        match self.curve_type {
            CurveType::Linear => t,
            CurveType::Step => {
                if t < 0.5 { 0.0 } else { 1.0 }
            }
            CurveType::EaseIn => self.ease_in_function(t),
            CurveType::EaseOut => self.ease_out_function(t),
            CurveType::EaseInOut => self.ease_in_out_function(t),
            CurveType::Bezier | CurveType::Custom => {
                self.sample_bezier(t)
            }
        }
    }

    fn ease_in_function(&self, t: f64) -> f64 {
        t * t
    }

    fn ease_out_function(&self, t: f64) -> f64 {
        1.0 - (1.0 - t) * (1.0 - t)
    }

    fn ease_in_out_function(&self, t: f64) -> f64 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            1.0 - 2.0 * (1.0 - t) * (1.0 - t)
        }
    }

    fn sample_bezier(&self, t: f64) -> f64 {
        if self.control_points.len() < 2 {
            return t;
        }

        let segment_count = self.control_points.len() - 1;
        let segment = (t * segment_count as f64).floor() as usize;
        let segment_t = (t * segment_count as f64) - segment as f64;

        if segment >= segment_count {
            return self.control_points.last().unwrap().position.1;
        }

        let p0 = self.control_points[segment];
        let p1 = self.control_points[segment + 1];

        let y0 = p0.position.1;
        let y1 = p1.position.1;

        y0 + (y1 - y0) * segment_t
    }

    pub fn description(&self) -> String {
        match self.curve_type {
            CurveType::Linear => "Linear interpolation".to_string(),
            CurveType::Bezier => format!("Bezier curve with {} control points", self.control_points.len()),
            CurveType::EaseIn => "Ease in (quadratic)".to_string(),
            CurveType::EaseOut => "Ease out (quadratic)".to_string(),
            CurveType::EaseInOut => "Ease in out (quadratic)".to_string(),
            CurveType::Step => "Step function".to_string(),
            CurveType::Custom => format!("Custom curve with {} control points", self.control_points.len()),
        }
    }
}

impl Default for AnimationCurve {
    fn default() -> Self {
        Self::linear()
    }
}

impl fmt::Display for AnimationCurve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

pub struct CurveBuilder {
    curve: AnimationCurve,
}

impl CurveBuilder {
    pub fn new() -> Self {
        Self {
            curve: AnimationCurve::linear(),
        }
    }

    pub fn curve_type(mut self, curve_type: CurveType) -> Self {
        self.curve.curve_type = curve_type;
        self
    }

    pub fn tension(mut self, tension: f64) -> Self {
        self.curve.tension = tension;
        self
    }

    pub fn bias(mut self, bias: f64) -> Self {
        self.curve.bias = bias;
        self
    }

    pub fn add_point(mut self, x: f64, y: f64) -> Self {
        self.curve.add_control_point(BezierControlPoint::new(x, y));
        self
    }

    pub fn add_point_with_handles(mut self, x: f64, y: f64, handle1: (f64, f64), handle2: (f64, f64)) -> Self {
        self.curve.add_control_point(BezierControlPoint::with_handles(x, y, handle1, handle2));
        self
    }

    pub fn build(self) -> AnimationCurve {
        self.curve
    }
}

impl Default for CurveBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curve_creation() {
        let curve = AnimationCurve::linear();

        assert_eq!(curve.curve_type, CurveType::Linear);
        assert_eq!(curve.tension, 0.5);
        assert_eq!(curve.bias, 0.0);
    }

    #[test]
    fn test_bezier_curve() {
        let points = vec![
            BezierControlPoint::new(0.0, 0.0),
            BezierControlPoint::new(1.0, 1.0),
        ];

        let curve = AnimationCurve::bezier(points);

        assert_eq!(curve.curve_type, CurveType::Bezier);
        assert_eq!(curve.control_point_count(), 2);
    }

    #[test]
    fn test_curve_sampling() {
        let linear_curve = AnimationCurve::linear();

        assert_eq!(linear_curve.sample(0.0), 0.0);
        assert_eq!(linear_curve.sample(0.5), 0.5);
        assert_eq!(linear_curve.sample(1.0), 1.0);
    }

    #[test]
    fn test_ease_curves() {
        let ease_in = AnimationCurve::ease_in();
        let ease_out = AnimationCurve::ease_out();
        let ease_in_out = AnimationCurve::ease_in_out();

        let t = 0.5;
        assert_ne!(ease_in.sample(t), ease_out.sample(t));
        assert_ne!(ease_in.sample(t), ease_in_out.sample(t));

        assert!(ease_in.sample(0.25) <= ease_in.sample(0.5));
        assert!(ease_in.sample(0.5) <= ease_in.sample(0.75));
    }

    #[test]
    fn test_step_curve() {
        let step_curve = AnimationCurve::step();

        assert_eq!(step_curve.sample(0.25), 0.0);
        assert_eq!(step_curve.sample(0.5), 0.0);
        assert_eq!(step_curve.sample(0.75), 1.0);
        assert_eq!(step_curve.sample(1.0), 1.0);
    }

    #[test]
    fn test_control_point_operations() {
        let mut curve = AnimationCurve::bezier(Vec::new());

        curve.add_control_point(BezierControlPoint::new(0.0, 0.0));
        curve.add_control_point(BezierControlPoint::new(1.0, 1.0));

        assert_eq!(curve.control_point_count(), 2);

        let point = curve.get_control_point(0);
        assert!(point.is_some());
        assert_eq!(point.unwrap().position, (0.0, 0.0));

        let removed = curve.remove_control_point(0);
        assert!(removed.is_some());
        assert_eq!(curve.control_point_count(), 1);
    }

    #[test]
    fn test_curve_validation() {
        let valid_bezier = AnimationCurve::bezier(vec![
            BezierControlPoint::new(0.0, 0.0),
            BezierControlPoint::new(1.0, 1.0),
        ]);

        assert!(valid_bezier.validate().is_ok());

        let invalid_bezier = AnimationCurve::bezier(vec![
            BezierControlPoint::new(0.0, 0.0),
        ]);

        assert!(invalid_bezier.validate().is_err());

        let invalid_linear = AnimationCurve::linear();
        let mut invalid_with_points = invalid_linear.clone();
        invalid_with_points.add_control_point(BezierControlPoint::new(0.0, 0.0));

        assert!(invalid_with_points.validate().is_err());
    }

    #[test]
    fn test_curve_builder() {
        let curve = CurveBuilder::new()
            .curve_type(CurveType::Bezier)
            .tension(0.7)
            .bias(0.1)
            .add_point(0.0, 0.0)
            .add_point(1.0, 1.0)
            .build();

        assert_eq!(curve.curve_type, CurveType::Bezier);
        assert_eq!(curve.tension, 0.7);
        assert_eq!(curve.bias, 0.1);
        assert_eq!(curve.control_point_count(), 2);
    }

    #[test]
    fn test_bezier_control_point() {
        let point = BezierControlPoint::new(0.5, 0.5);

        assert_eq!(point.position, (0.5, 0.5));
        assert!(!point.has_handles());

        let mut point_with_handles = point;
        point_with_handles.set_handle1((0.3, 0.3));
        point_with_handles.set_handle2((0.7, 0.7));

        assert!(point_with_handles.has_handles());
        assert_eq!(point_with_handles.handle1, Some((0.3, 0.3)));
        assert_eq!(point_with_handles.handle2, Some((0.7, 0.7)));
    }

    #[test]
    fn test_curve_descriptions() {
        let linear = AnimationCurve::linear();
        let bezier = AnimationCurve::bezier(vec![BezierControlPoint::new(0.0, 0.0), BezierControlPoint::new(1.0, 1.0)]);

        assert!(linear.description().contains("Linear"));
        assert!(bezier.description().contains("Bezier"));
        assert!(bezier.description().contains("2 control points"));
    }
}
