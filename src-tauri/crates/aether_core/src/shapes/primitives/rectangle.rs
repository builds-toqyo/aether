

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rectangle {

    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub corner_radius: f64,
}

impl Rectangle {

    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            corner_radius: 0.0,
        }
    }


    pub fn rounded(x: f64, y: f64, width: f64, height: f64, corner_radius: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            corner_radius: corner_radius.max(0.0),
        }
    }


    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }


    pub fn is_square(&self) -> bool {
        (self.width - self.height).abs() < f64::EPSILON
    }


    pub fn is_valid(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

impl super::types::ShapePrimitive for Rectangle {
    fn shape_type(&self) -> super::types::ShapeType {
        super::types::ShapeType::Rectangle
    }

    fn bounds(&self) -> super::transform::BoundingBox {
        super::transform::BoundingBox::new(self.x, self.y, self.x + self.width, self.y + self.height)
    }

    fn to_path(&self) -> super::super::paths::Path {
        let mut builder = super::super::paths::PathBuilder::new();

        if self.corner_radius > 0.0 {

            let r = self.corner_radius.min(self.width / 2.0).min(self.height / 2.0);
            builder.move_to(self.x + r, self.y);
            builder.line_to(self.x + self.width - r, self.y);
            builder.quadratic_to(self.x + self.width, self.y, self.x + self.width, self.y + r);
            builder.line_to(self.x + self.width, self.y + self.height - r);
            builder.quadratic_to(self.x + self.width, self.y + self.height, self.x + self.width - r, self.y + self.height);
            builder.line_to(self.x + r, self.y + self.height);
            builder.quadratic_to(self.x, self.y + self.height, self.x, self.y + self.height - r);
            builder.line_to(self.x, self.y + r);
            builder.quadratic_to(self.x, self.y, self.x + r, self.y);
        } else {

            builder.move_to(self.x, self.y);
            builder.line_to(self.x + self.width, self.y);
            builder.line_to(self.x + self.width, self.y + self.height);
            builder.line_to(self.x, self.y + self.height);
            builder.close();
        }

        builder.build()
    }

    fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }

    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }

    fn transform(&mut self, transform: &super::transform::Transform) {
        let (x1, y1) = transform.transform_point(self.x, self.y);
        let (x2, y2) = transform.transform_point(self.x + self.width, self.y + self.height);

        self.x = x1;
        self.y = y1;
        self.width = (x2 - x1).abs();
        self.height = (y2 - y1).abs();


        let scale = (self.width * self.height).sqrt() / ((self.width * self.height).sqrt());
        self.corner_radius *= scale;
    }

    fn transformed(&self, transform: &super::transform::Transform) -> Self {
        let mut copy = *self;
        copy.transform(transform);
        copy
    }

    fn validate(&self) -> Result<(), String> {
        if self.width <= 0.0 {
            return Err("Rectangle width must be positive".to_string());
        }
        if self.height <= 0.0 {
            return Err("Rectangle height must be positive".to_string());
        }
        if self.corner_radius < 0.0 {
            return Err("Corner radius cannot be negative".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ShapePrimitive;

    #[test]
    fn test_rectangle_creation() {
        let rect = Rectangle::new(10.0, 20.0, 100.0, 50.0);

        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
        assert_eq!(rect.corner_radius, 0.0);
        assert_eq!(rect.center(), (60.0, 45.0));
        assert!(rect.is_valid());
    }

    #[test]
    fn test_rounded_rectangle() {
        let rect = Rectangle::rounded(10.0, 20.0, 100.0, 50.0, 10.0);

        assert_eq!(rect.corner_radius, 10.0);
        assert!(rect.is_valid());
    }

    #[test]
    fn test_rectangle_area_perimeter() {
        let rect = Rectangle::new(0.0, 0.0, 10.0, 5.0);

        assert_eq!(rect.area(), 50.0);
        assert_eq!(rect.perimeter(), 30.0);
    }

    #[test]
    fn test_rectangle_contains_point() {
        let rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);

        assert!(rect.contains_point(5.0, 5.0));
        assert!(rect.contains_point(0.0, 0.0));
        assert!(rect.contains_point(10.0, 10.0));
        assert!(!rect.contains_point(-1.0, 5.0));
        assert!(!rect.contains_point(11.0, 5.0));
    }

    #[test]
    fn test_rectangle_transform() {
        let mut rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        let transform = super::transform::Transform::translation(5.0, 10.0);

        rect.transform(&transform);

        assert_eq!(rect.x, 5.0);
        assert_eq!(rect.y, 10.0);
        assert_eq!(rect.width, 10.0);
        assert_eq!(rect.height, 10.0);
    }

    #[test]
    fn test_rectangle_validation() {
        let valid_rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        assert!(valid_rect.validate().is_ok());

        let invalid_width = Rectangle::new(0.0, 0.0, -10.0, 10.0);
        assert!(invalid_width.validate().is_err());

        let invalid_height = Rectangle::new(0.0, 0.0, 10.0, -10.0);
        assert!(invalid_height.validate().is_err());

        let invalid_radius = Rectangle::rounded(0.0, 0.0, 10.0, 10.0, -5.0);
        assert!(invalid_radius.validate().is_err());
    }

    #[test]
    fn test_rectangle_bounds() {
        let rect = Rectangle::new(10.0, 20.0, 100.0, 50.0);
        let bounds = rect.bounds();

        assert_eq!(bounds.min_x, 10.0);
        assert_eq!(bounds.min_y, 20.0);
        assert_eq!(bounds.max_x, 110.0);
        assert_eq!(bounds.max_y, 70.0);
        assert_eq!(bounds.width(), 100.0);
        assert_eq!(bounds.height(), 50.0);
    }

    #[test]
    fn test_rectangle_to_path() {
        let rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        let path = rect.to_path();

        assert!(path.closed);
        assert!((path.area() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_rounded_rectangle_to_path() {
        let rect = Rectangle::rounded(0.0, 0.0, 10.0, 10.0, 2.0);
        let path = rect.to_path();

        assert!(path.closed);
        assert!(path.area() < 100.0);
    }
}
