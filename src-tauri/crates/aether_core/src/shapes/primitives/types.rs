use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Ellipse,
    Line,
    Polygon,
    Path,
}

impl fmt::Display for ShapeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShapeType::Rectangle => write!(f, "Rectangle"),
            ShapeType::Circle => write!(f, "Circle"),
            ShapeType::Ellipse => write!(f, "Ellipse"),
            ShapeType::Line => write!(f, "Line"),
            ShapeType::Polygon => write!(f, "Polygon"),
            ShapeType::Path => write!(f, "Path"),
        }
    }
}

pub trait ShapePrimitive {
    fn shape_type(&self) -> ShapeType;
    fn bounds(&self) -> super::transform::BoundingBox;
    fn to_path(&self) -> super::super::paths::Path;
    fn contains_point(&self, x: f64, y: f64) -> bool;
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn transform(&mut self, transform: &super::transform::Transform);
    fn transformed(&self, transform: &super::transform::Transform) -> Self where Self: Sized;
    fn clone_box(&self) -> Box<dyn ShapePrimitive>;
    fn validate(&self) -> Result<(), String>;
}
