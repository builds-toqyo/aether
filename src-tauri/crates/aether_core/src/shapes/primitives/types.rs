//! Shape primitive types
//! 
//! This module contains the basic type definitions for shape primitives.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Basic shape primitive types
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

/// Base trait for all shape primitives
pub trait ShapePrimitive {
    /// Get the shape type
    fn shape_type(&self) -> ShapeType;
    
    /// Get the bounding box of the shape
    fn bounds(&self) -> super::transform::BoundingBox;
    
    /// Convert shape to path
    fn to_path(&self) -> super::super::paths::Path;
    
    /// Check if point is inside shape
    fn contains_point(&self, x: f64, y: f64) -> bool;
    
    /// Get area of the shape
    fn area(&self) -> f64;
    
    /// Get perimeter of the shape
    fn perimeter(&self) -> f64;
    
    /// Transform the shape with a transformation matrix
    fn transform(&mut self, transform: &super::transform::Transform);
    
    /// Get transformed copy of the shape
    fn transformed(&self, transform: &super::transform::Transform) -> Self where Self: Sized;
    
    /// Validate shape data
    fn validate(&self) -> Result<(), String>;
}
