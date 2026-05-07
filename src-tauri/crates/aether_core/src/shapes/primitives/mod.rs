//! Shape primitives module
//! 
//! This module provides basic shape primitives for vector graphics
//! including rectangles, circles, ellipses, lines, and polygons.

pub mod types;
pub mod transform;
pub mod rectangle;
pub mod circle;
pub mod ellipse;
pub mod line;
pub mod polygon;

// Re-export main types
pub use types::{ShapeType, ShapePrimitive};
pub use transform::{Transform, BoundingBox};
pub use rectangle::Rectangle;
pub use circle::Circle;
pub use ellipse::Ellipse;
pub use line::Line;
pub use polygon::Polygon;
