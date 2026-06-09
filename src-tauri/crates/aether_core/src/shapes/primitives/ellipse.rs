

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ellipse {

    pub cx: f64,

    pub cy: f64,

    pub rx: f64,

    pub ry: f64,

    pub rotation: f64,
}

impl Ellipse {

    pub fn new(cx: f64, cy: f64, rx: f64, ry: f64) -> Self {
        Self {
            cx,
            cy,
            rx,
            ry,
            rotation: 0.0,
        }
    }


    pub fn rotated(cx: f64, cy: f64, rx: f64, ry: f64, rotation: f64) -> Self {
        Self {
            cx,
            cy,
            rx,
            ry,
            rotation,
        }
    }


    pub fn is_circle(&self) -> bool {
        (self.rx - self.ry).abs() < f64::EPSILON
    }


    pub fn is_valid(&self) -> bool {
        self.rx > 0.0 && self.ry > 0.0
    }
}

impl super::types::ShapePrimitive for Ellipse {
    fn shape_type(&self) -> super::types::ShapeType {
        super::types::ShapeType::Ellipse
    }

    fn bounds(&self) -> super::transform::BoundingBox {

        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        let a = self.rx;
        let b = self.ry;


        let max_dx = (a * a * cos_r * cos_r + b * b * sin_r * sin_r).sqrt();
        let max_dy = (a * a * sin_r * sin_r + b * b * cos_r * cos_r).sqrt();

        super::transform::BoundingBox::new(
            self.cx - max_dx,
            self.cy - max_dy,
            self.cx + max_dx,
            self.cy + max_dy,
        )
    }

    fn to_path(&self) -> super::super::paths::Path {
        let mut builder = super::super::paths::PathBuilder::new();


        let k = 0.552284749831;


        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        let transform_point = |x: f64, y: f64| {
            let tx = x * cos_r - y * sin_r + self.cx;
            let ty = x * sin_r + y * cos_r + self.cy;
            (tx, ty)
        };

        let (x0, y0) = transform_point(self.rx, 0.0);
        builder.move_to(x0, y0);

        let (x1, y1) = transform_point(self.rx, -k * self.ry);
        let (x2, y2) = transform_point(k * self.rx, -self.ry);
        let (x3, y3) = transform_point(0.0, -self.ry);
        builder.bezier_to(x1, y1, x2, y2, x3, y3);

        let (x1, y1) = transform_point(-k * self.rx, -self.ry);
        let (x2, y2) = transform_point(-self.rx, -k * self.ry);
        let (x3, y3) = transform_point(-self.rx, 0.0);
        builder.bezier_to(x1, y1, x2, y2, x3, y3);

        let (x1, y1) = transform_point(-self.rx, k * self.ry);
        let (x2, y2) = transform_point(-k * self.rx, self.ry);
        let (x3, y3) = transform_point(0.0, self.ry);
        builder.bezier_to(x1, y1, x2, y2, x3, y3);

        let (x1, y1) = transform_point(k * self.rx, self.ry);
        let (x2, y2) = transform_point(self.rx, k * self.ry);
        let (x3, y3) = transform_point(self.rx, 0.0);
        builder.bezier_to(x1, y1, x2, y2, x3, y3);

        builder.close();

        builder.build()
    }

    fn contains_point(&self, x: f64, y: f64) -> bool {

        let dx = x - self.cx;
        let dy = y - self.cy;

        let cos_r = (-self.rotation).cos();
        let sin_r = (-self.rotation).sin();

        let local_x = dx * cos_r - dy * sin_r;
        let local_y = dx * sin_r + dy * cos_r;


        (local_x * local_x) / (self.rx * self.rx) + (local_y * local_y) / (self.ry * self.ry) <= 1.0
    }

    fn area(&self) -> f64 {
        std::f64::consts::PI * self.rx * self.ry
    }

    fn perimeter(&self) -> f64 {

        let a = self.rx;
        let b = self.ry;
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));
        std::f64::consts::PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
    }

    fn transform(&mut self, transform: &super::transform::Transform) {
        let (cx, cy) = transform.transform_point(self.cx, self.cy);
        self.cx = cx;
        self.cy = cy;

        self.rx *= transform.sx;
        self.ry *= transform.sy;

        self.rotation += transform.rotation;
    }

    fn transformed(&self, transform: &super::transform::Transform) -> Self {
        let mut copy = *self;
        copy.transform(transform);
        copy
    }

    fn clone_box(&self) -> Box<dyn super::types::ShapePrimitive> {
        Box::new(self.clone())
    }

    fn validate(&self) -> Result<(), String> {
        if self.rx <= 0.0 {
            return Err("Ellipse horizontal radius must be positive".to_string());
        }
        if self.ry <= 0.0 {
            return Err("Ellipse vertical radius must be positive".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ShapePrimitive;

    #[test]
    fn test_ellipse_creation() {
        let ellipse = Ellipse::new(50.0, 50.0, 40.0, 20.0);

        assert_eq!(ellipse.cx, 50.0);
        assert_eq!(ellipse.cy, 50.0);
        assert_eq!(ellipse.rx, 40.0);
        assert_eq!(ellipse.ry, 20.0);
        assert_eq!(ellipse.rotation, 0.0);
        assert!(!ellipse.is_circle());
        assert!(ellipse.is_valid());
    }

    #[test]
    fn test_rotated_ellipse() {
        let ellipse = Ellipse::rotated(50.0, 50.0, 40.0, 20.0, std::f64::consts::PI / 4.0);

        assert_eq!(ellipse.rotation, std::f64::consts::PI / 4.0);
        assert!(ellipse.is_valid());
    }

    #[test]
    fn test_circle_ellipse() {
        let circle_ellipse = Ellipse::new(50.0, 50.0, 25.0, 25.0);
        assert!(circle_ellipse.is_circle());
    }

    #[test]
    fn test_ellipse_area_perimeter() {
        let ellipse = Ellipse::new(0.0, 0.0, 40.0, 20.0);

        let expected_area = std::f64::consts::PI * 40.0 * 20.0;
        assert!((ellipse.area() - expected_area).abs() < 0.001);


        let perimeter = ellipse.perimeter();
        assert!(perimeter > 180.0 && perimeter < 200.0);
    }

    #[test]
    fn test_ellipse_contains_point() {
        let ellipse = Ellipse::new(50.0, 50.0, 40.0, 20.0);

        assert!(ellipse.contains_point(50.0, 50.0));
        assert!(ellipse.contains_point(85.0, 50.0));
        assert!(ellipse.contains_point(70.0, 50.0));
        assert!(!ellipse.contains_point(95.0, 50.0));
        assert!(!ellipse.contains_point(50.0, 75.0));
    }

    #[test]
    fn test_rotated_ellipse_contains_point() {
        let ellipse = Ellipse::rotated(50.0, 50.0, 40.0, 20.0, std::f64::consts::PI / 2.0);

        assert!(ellipse.contains_point(50.0, 50.0));
        assert!(ellipse.contains_point(50.0, 85.0));
        assert!(!ellipse.contains_point(85.0, 50.0));
    }

    #[test]
    fn test_ellipse_transform() {
        let mut ellipse = Ellipse::new(0.0, 0.0, 40.0, 20.0);
        let transform = super::transform::Transform::translation(5.0, 10.0);

        ellipse.transform(&transform);

        assert_eq!(ellipse.cx, 5.0);
        assert_eq!(ellipse.cy, 10.0);
        assert_eq!(ellipse.rx, 40.0);
        assert_eq!(ellipse.ry, 20.0);
    }

    #[test]
    fn test_ellipse_validation() {
        let valid_ellipse = Ellipse::new(0.0, 0.0, 40.0, 20.0);
        assert!(valid_ellipse.validate().is_ok());

        let invalid_rx = Ellipse::new(0.0, 0.0, -40.0, 20.0);
        assert!(invalid_rx.validate().is_err());

        let invalid_ry = Ellipse::new(0.0, 0.0, 40.0, -20.0);
        assert!(invalid_ry.validate().is_err());
    }

    #[test]
    fn test_ellipse_bounds() {
        let ellipse = Ellipse::new(50.0, 50.0, 40.0, 20.0);
        let bounds = ellipse.bounds();

        assert_eq!(bounds.min_x, 10.0);
        assert_eq!(bounds.min_y, 30.0);
        assert_eq!(bounds.max_x, 90.0);
        assert_eq!(bounds.max_y, 70.0);
        assert_eq!(bounds.center(), (50.0, 50.0));
    }

    #[test]
    fn test_rotated_ellipse_bounds() {
        let ellipse = Ellipse::rotated(50.0, 50.0, 40.0, 20.0, std::f64::consts::PI / 4.0);
        let bounds = ellipse.bounds();


        assert!(bounds.width() > 80.0);
        assert!(bounds.height() > 40.0);
        assert_eq!(bounds.center(), (50.0, 50.0));
    }

    #[test]
    fn test_ellipse_to_path() {
        let ellipse = Ellipse::new(50.0, 50.0, 40.0, 20.0);
        let path = ellipse.to_path();

        assert!(path.closed);
        let expected_area = std::f64::consts::PI * 40.0 * 20.0;
        assert!((path.area() - expected_area).abs() < 100.0);
    }

    #[test]
    fn test_invalid_ellipse() {
        let invalid_ellipse = Ellipse::new(0.0, 0.0, 0.0, 20.0);
        assert!(!invalid_ellipse.is_valid());
        assert!(invalid_ellipse.validate().is_err());
    }
}
