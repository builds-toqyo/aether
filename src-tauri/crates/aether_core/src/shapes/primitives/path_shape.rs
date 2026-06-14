use serde::{Deserialize, Serialize};
use crate::shapes::paths::Path;
use crate::shapes::primitives::transform::{BoundingBox, Transform};
use crate::shapes::primitives::types::{ShapePrimitive, ShapeType};

/// A generic shape that wraps a Path, allowing any path-based geometry
/// to be treated as a ShapePrimitive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathShape {
    pub path: Path,
}

impl PathShape {
    pub fn new(path: Path) -> Self {
        Self { path }
    }

    pub fn from_path(path: Path) -> Self {
        Self::new(path)
    }

    pub fn empty() -> Self {
        Self::new(Path::new())
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    pub fn vertices(&self) -> Vec<(f64, f64)> {
        self.path.vertices(0.1)
    }
}

impl ShapePrimitive for PathShape {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Path
    }

    fn bounds(&self) -> BoundingBox {
        self.path.bounds()
    }

    fn to_path(&self) -> Path {
        self.path.clone()
    }

    fn contains_point(&self, x: f64, y: f64) -> bool {
        self.path.contains_point(x, y)
    }

    fn area(&self) -> f64 {
        self.path.area()
    }

    fn perimeter(&self) -> f64 {
        self.path.length()
    }

    fn transform(&mut self, transform: &Transform) {
        self.path.transform(transform);
    }

    fn transformed(&self, transform: &Transform) -> Self
    where
        Self: Sized,
    {
        Self {
            path: self.path.transformed(transform),
        }
    }

    fn clone_box(&self) -> Box<dyn ShapePrimitive> {
        Box::new(self.clone())
    }

    fn validate(&self) -> Result<(), String> {
        if self.path.is_empty() {
            return Err("PathShape has no segments".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::paths::PathSegment;

    #[test]
    fn test_path_shape_from_square() {
        let mut path = Path::new();
        path.add_segment(PathSegment::move_to(0.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 10.0));
        path.add_segment(PathSegment::line_to(0.0, 10.0));
        path.add_segment(PathSegment::close());

        let shape = PathShape::from_path(path);

        assert_eq!(shape.shape_type(), ShapeType::Path);
        assert_eq!(shape.area(), 100.0);
        assert!(shape.contains_point(5.0, 5.0));
        assert!(!shape.contains_point(15.0, 5.0));

        let bounds = shape.bounds();
        assert_eq!(bounds.min_x, 0.0);
        assert_eq!(bounds.min_y, 0.0);
        assert_eq!(bounds.max_x, 10.0);
        assert_eq!(bounds.max_y, 10.0);
    }

    #[test]
    fn test_path_shape_transform() {
        let mut path = Path::new();
        path.add_segment(PathSegment::move_to(0.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 10.0));
        path.add_segment(PathSegment::line_to(0.0, 10.0));
        path.add_segment(PathSegment::close());

        let shape = PathShape::from_path(path);
        let translated = shape.transformed(&Transform::translation(5.0, 5.0));

        assert!(translated.contains_point(5.0, 5.0));
        assert!(!translated.contains_point(0.0, 0.0));
    }
}
