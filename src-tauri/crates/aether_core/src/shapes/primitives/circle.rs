

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle {

    pub cx: f64,

    pub cy: f64,

    pub radius: f64,
}

impl Circle {

    pub fn new(cx: f64, cy: f64, radius: f64) -> Self {
        Self { cx, cy, radius }
    }


    pub fn diameter(&self) -> f64 {
        self.radius * 2.0
    }


    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }


    pub fn is_valid(&self) -> bool {
        self.radius > 0.0
    }
}

impl super::types::ShapePrimitive for Circle {
    fn shape_type(&self) -> super::types::ShapeType {
        super::types::ShapeType::Circle
    }

    fn bounds(&self) -> super::transform::BoundingBox {
        super::transform::BoundingBox::new(
            self.cx - self.radius,
            self.cy - self.radius,
            self.cx + self.radius,
            self.cy + self.radius,
        )
    }

    fn to_path(&self) -> super::super::paths::Path {
        let mut builder = super::super::paths::PathBuilder::new();


        let k = 0.552284749831;

        builder.move_to(self.cx + self.radius, self.cy);
        builder.bezier_to(
            self.cx + self.radius, self.cy - k * self.radius,
            self.cx + k * self.radius, self.cy - self.radius,
            self.cx, self.cy - self.radius,
        );
        builder.bezier_to(
            self.cx - k * self.radius, self.cy - self.radius,
            self.cx - self.radius, self.cy - k * self.radius,
            self.cx - self.radius, self.cy,
        );
        builder.bezier_to(
            self.cx - self.radius, self.cy + k * self.radius,
            self.cx - k * self.radius, self.cy + self.radius,
            self.cx, self.cy + self.radius,
        );
        builder.bezier_to(
            self.cx + k * self.radius, self.cy + self.radius,
            self.cx + self.radius, self.cy + k * self.radius,
            self.cx + self.radius, self.cy,
        );
        builder.close();

        builder.build()
    }

    fn contains_point(&self, x: f64, y: f64) -> bool {
        let dx = x - self.cx;
        let dy = y - self.cy;
        dx * dx + dy * dy <= self.radius * self.radius
    }

    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn perimeter(&self) -> f64 {
        self.circumference()
    }

    fn transform(&mut self, transform: &super::transform::Transform) {
        let (cx, cy) = transform.transform_point(self.cx, self.cy);
        self.cx = cx;
        self.cy = cy;


        self.radius *= (transform.sx * transform.sy).sqrt() / 2.0;
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
        if self.radius <= 0.0 {
            return Err("Circle radius must be positive".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ShapePrimitive;

    #[test]
    fn test_circle_creation() {
        let circle = Circle::new(50.0, 50.0, 25.0);

        assert_eq!(circle.cx, 50.0);
        assert_eq!(circle.cy, 50.0);
        assert_eq!(circle.radius, 25.0);
        assert_eq!(circle.diameter(), 50.0);
        assert!(circle.is_valid());
    }

    #[test]
    fn test_circle_area_perimeter() {
        let circle = Circle::new(0.0, 0.0, 10.0);

        let expected_area = std::f64::consts::PI * 100.0;
        assert!((circle.area() - expected_area).abs() < 0.001);

        let expected_perimeter = 2.0 * std::f64::consts::PI * 10.0;
        assert!((circle.perimeter() - expected_perimeter).abs() < 0.001);
    }

    #[test]
    fn test_circle_contains_point() {
        let circle = Circle::new(50.0, 50.0, 25.0);

        assert!(circle.contains_point(50.0, 50.0));
        assert!(circle.contains_point(70.0, 50.0));
        assert!(circle.contains_point(65.0, 50.0));
        assert!(!circle.contains_point(80.0, 50.0));
        assert!(!circle.contains_point(50.0, 80.0));
    }

    #[test]
    fn test_circle_transform() {
        let mut circle = Circle::new(0.0, 0.0, 10.0);
        let transform = super::transform::Transform::translation(5.0, 10.0);

        circle.transform(&transform);

        assert_eq!(circle.cx, 5.0);
        assert_eq!(circle.cy, 10.0);
    }

    #[test]
    fn test_circle_validation() {
        let valid_circle = Circle::new(0.0, 0.0, 10.0);
        assert!(valid_circle.validate().is_ok());

        let invalid_circle = Circle::new(0.0, 0.0, -10.0);
        assert!(invalid_circle.validate().is_err());
    }

    #[test]
    fn test_circle_bounds() {
        let circle = Circle::new(50.0, 50.0, 25.0);
        let bounds = circle.bounds();

        assert_eq!(bounds.min_x, 25.0);
        assert_eq!(bounds.min_y, 25.0);
        assert_eq!(bounds.max_x, 75.0);
        assert_eq!(bounds.max_y, 75.0);
        assert_eq!(bounds.width(), 50.0);
        assert_eq!(bounds.height(), 50.0);
        assert_eq!(bounds.center(), (50.0, 50.0));
    }

    #[test]
    fn test_circle_to_path() {
        let circle = Circle::new(50.0, 50.0, 25.0);
        let path = circle.to_path();

        assert!(path.closed);
        let expected_area = std::f64::consts::PI * 25.0 * 25.0;
        assert!((path.area() - expected_area).abs() < 100.0);
    }

    #[test]
    fn test_invalid_circle() {
        let invalid_circle = Circle::new(0.0, 0.0, 0.0);
        assert!(!invalid_circle.is_valid());
        assert!(invalid_circle.validate().is_err());
    }
}
