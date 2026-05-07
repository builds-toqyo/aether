

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::shapes::paths::{Path, PathBuilder};


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
            ShapeType::Rectangle => write!(f, __STRING_0__),
            ShapeType::Circle => write!(f, __STRING_1__),
            ShapeType::Ellipse => write!(f, __STRING_2__),
            ShapeType::Line => write!(f, __STRING_3__),
            ShapeType::Polygon => write!(f, __STRING_4__),
            ShapeType::Path => write!(f, __STRING_5__),
        }
    }
}

/// Base trait for all shape primitives
pub trait ShapePrimitive {
    /// Get the shape type
    fn shape_type(&self) -> ShapeType;

    /// Get the bounding box of the shape
    fn bounds(&self) -> BoundingBox;

    /// Convert shape to path
    fn to_path(&self) -> Path;

    /// Check if point is inside shape
    fn contains_point(&self, x: f64, y: f64) -> bool;

    /// Get area of the shape
    fn area(&self) -> f64;

    /// Get perimeter of the shape
    fn perimeter(&self) -> f64;

    /// Transform the shape with a transformation matrix
    fn transform(&mut self, transform: &Transform);

    /// Get transformed copy of the shape
    fn transformed(&self, transform: &Transform) -> Self where Self: Sized;

    /// Validate shape data
    fn validate(&self) -> Result<(), String>;
}

/// 2D transformation matrix
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// Translation X
    pub tx: f64,
    /// Translation Y
    pub ty: f64,
    /// Scale X
    pub sx: f64,
    /// Scale Y
    pub sy: f64,
    /// Rotation in radians
    pub rotation: f64,
    /// Skew X
    pub kx: f64,
    /// Skew Y
    pub ky: f64,
}

impl Transform {
    /// Create identity transform
    pub fn identity() -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            sx: 1.0,
            sy: 1.0,
            rotation: 0.0,
            kx: 0.0,
            ky: 0.0,
        }
    }

    /// Create translation transform
    pub fn translation(tx: f64, ty: f64) -> Self {
        Self {
            tx,
            ty,
            sx: 1.0,
            sy: 1.0,
            rotation: 0.0,
            kx: 0.0,
            ky: 0.0,
        }
    }

    /// Create scale transform
    pub fn scale(sx: f64, sy: f64) -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            sx,
            sy,
            rotation: 0.0,
            kx: 0.0,
            ky: 0.0,
        }
    }

    /// Create rotation transform
    pub fn rotation(angle: f64) -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            sx: 1.0,
            sy: 1.0,
            rotation: angle,
            kx: 0.0,
            ky: 0.0,
        }
    }

    /// Apply transform to point
    pub fn transform_point(&self, x: f64, y: f64) -> (f64, f64) {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        // Apply scale and rotation
        let x_rot = x * self.sx * cos_r - y * self.sy * sin_r;
        let y_rot = x * self.sx * sin_r + y * self.sy * cos_r;

        // Apply skew
        let x_skew = x_rot + y_rot * self.kx;
        let y_skew = y_rot + x_rot * self.ky;

        // Apply translation
        (x_skew + self.tx, y_skew + self.ty)
    }

    /// Get inverse transform
    pub fn inverse(&self) -> Option<Self> {
        if self.sx == 0.0 || self.sy == 0.0 {
            return None;
        }

        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        // Inverse scale
        let inv_sx = 1.0 / self.sx;
        let inv_sy = 1.0 / self.sy;

        // Inverse rotation
        let inv_cos = cos_r;
        let inv_sin = -sin_r;

        // Calculate inverse translation
        let inv_tx = -(self.tx * inv_cos - self.ty * inv_sin) * inv_sx;
        let inv_ty = -(self.tx * inv_sin + self.ty * inv_cos) * inv_sy;

        Some(Self {
            tx: inv_tx,
            ty: inv_ty,
            sx: inv_sx,
            sy: inv_sy,
            rotation: -self.rotation,
            kx: -self.kx,
            ky: -self.ky,
        })
    }

    /// Combine with another transform
    pub fn combine(&self, other: &Transform) -> Self {
        let cos_r1 = self.rotation.cos();
        let sin_r1 = self.rotation.sin();
        let cos_r2 = other.rotation.cos();
        let sin_r2 = other.rotation.sin();

        // Combine rotations and scales
        let sx = self.sx * other.sx;
        let sy = self.sy * other.sy;
        let rotation = self.rotation + other.rotation;

        // Combine translations
        let tx = self.tx + other.tx * self.sx * cos_r1 - other.ty * self.sy * sin_r1;
        let ty = self.ty + other.tx * self.sx * sin_r1 + other.ty * self.sy * cos_r1;

        // Combine skews
        let kx = self.kx + other.kx;
        let ky = self.ky + other.ky;

        Self {
            tx,
            ty,
            sx,
            sy,
            rotation,
            kx,
            ky,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

/// Bounding box for shapes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum X coordinate
    pub min_x: f64,
    /// Minimum Y coordinate
    pub min_y: f64,
    /// Maximum X coordinate
    pub max_x: f64,
    /// Maximum Y coordinate
    pub max_y: f64,
}

impl BoundingBox {
    /// Create new bounding box
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            max_x: max_x.max(min_x),
            max_y: max_y.max(min_y),
        }
    }

    /// Get width of bounding box
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Get height of bounding box
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Get center point
    pub fn center(&self) -> (f64, f64) {
        ((self.min_x + self.max_x) / 2.0, (self.min_y + self.max_y) / 2.0)
    }

    /// Check if point is inside bounding box
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    /// Check if bounding box intersects with another
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        !(self.max_x < other.min_x || self.min_x > other.max_x ||
          self.max_y < other.min_y || self.min_y > other.max_y)
    }

    /// Union with another bounding box
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
        )
    }

    /// Apply transform to bounding box
    pub fn transform(&self, transform: &Transform) -> BoundingBox {
        let corners = [
            transform.transform_point(self.min_x, self.min_y),
            transform.transform_point(self.max_x, self.min_y),
            transform.transform_point(self.max_x, self.max_y),
            transform.transform_point(self.min_x, self.max_y),
        ];

        let min_x = corners.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
        let max_x = corners.iter().map(|(x, _)| *x).fold(f64::NEG_INFINITY, f64::max);
        let min_y = corners.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
        let max_y = corners.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, f64::max);

        BoundingBox::new(min_x, min_y, max_x, max_y)
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

/// Rectangle shape primitive
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rectangle {
    /// X coordinate of top-left corner
    pub x: f64,
    /// Y coordinate of top-left corner
    pub y: f64,
    /// Width of rectangle
    pub width: f64,
    /// Height of rectangle
    pub height: f64,
    /// Corner radius for rounded rectangles
    pub corner_radius: f64,
}

impl Rectangle {
    /// Create new rectangle
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            corner_radius: 0.0,
        }
    }

    /// Create rounded rectangle
    pub fn rounded(x: f64, y: f64, width: f64, height: f64, corner_radius: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            corner_radius: corner_radius.max(0.0),
        }
    }

    /// Get center point
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Check if rectangle is square
    pub fn is_square(&self) -> bool {
        (self.width - self.height).abs() < f64::EPSILON
    }

    /// Check if rectangle is valid (positive dimensions)
    pub fn is_valid(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

impl ShapePrimitive for Rectangle {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Rectangle
    }

    fn bounds(&self) -> BoundingBox {
        BoundingBox::new(self.x, self.y, self.x + self.width, self.y + self.height)
    }

    fn to_path(&self) -> Path {
        let mut builder = PathBuilder::new();

        if self.corner_radius > 0.0 {
            // Rounded rectangle
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
            // Regular rectangle
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

    fn transform(&mut self, transform: &Transform) {
        let (x1, y1) = transform.transform_point(self.x, self.y);
        let (x2, y2) = transform.transform_point(self.x + self.width, self.y + self.height);

        self.x = x1;
        self.y = y1;
        self.width = (x2 - x1).abs();
        self.height = (y2 - y1).abs();

        // Transform corner radius proportionally
        let scale = (self.width * self.height).sqrt() / ((self.width * self.height).sqrt());
        self.corner_radius *= scale;
    }

    fn transformed(&self, transform: &Transform) -> Self {
        let mut copy = *self;
        copy.transform(transform);
        copy
    }

    fn validate(&self) -> Result<(), String> {
        if self.width <= 0.0 {
            return Err(__STRING_6__.to_string());
        }
        if self.height <= 0.0 {
            return Err(__STRING_7__.to_string());
        }
        if self.corner_radius < 0.0 {
            return Err(__STRING_8__.to_string());
        }
        Ok(())
    }
}

/// Circle shape primitive
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    /// Center X coordinate
    pub cx: f64,
    /// Center Y coordinate
    pub cy: f64,
    /// Radius
    pub radius: f64,
}

impl Circle {
    /// Create new circle
    pub fn new(cx: f64, cy: f64, radius: f64) -> Self {
        Self { cx, cy, radius }
    }

    /// Get diameter
    pub fn diameter(&self) -> f64 {
        self.radius * 2.0
    }

    /// Get circumference
    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    /// Check if circle is valid (positive radius)
    pub fn is_valid(&self) -> bool {
        self.radius > 0.0
    }
}

impl ShapePrimitive for Circle {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Circle
    }

    fn bounds(&self) -> BoundingBox {
        BoundingBox::new(
            self.cx - self.radius,
            self.cy - self.radius,
            self.cx + self.radius,
            self.cy + self.radius,
        )
    }

    fn to_path(&self) -> Path {
        let mut builder = PathBuilder::new();

        // Approximate circle with bezier curves
        let k = 0.552284749831; // Magic number for circle approximation

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

    fn transform(&mut self, transform: &Transform) {
        let (cx, cy) = transform.transform_point(self.cx, self.cy);
        self.cx = cx;
        self.cy = cy;

        // Scale radius by average scale factor
        self.radius *= (transform.sx * transform.sy).sqrt() / 2.0;
    }

    fn transformed(&self, transform: &Transform) -> Self {
        let mut copy = *self;
        copy.transform(transform);
        copy
    }

    fn validate(&self) -> Result<(), String> {
        if self.radius <= 0.0 {
            return Err(__STRING_9__.to_string());
        }
        Ok(())
    }
}

/// Ellipse shape primitive
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ellipse {
    /// Center X coordinate
    pub cx: f64,
    /// Center Y coordinate
    pub cy: f64,
    /// Horizontal radius
    pub rx: f64,
    /// Vertical radius
    pub ry: f64,
    /// Rotation angle in radians
    pub rotation: f64,
}

impl Ellipse {
    /// Create new ellipse
    pub fn new(cx: f64, cy: f64, rx: f64, ry: f64) -> Self {
        Self {
            cx,
            cy,
            rx,
            ry,
            rotation: 0.0,
        }
    }

    /// Create rotated ellipse
    pub fn rotated(cx: f64, cy: f64, rx: f64, ry: f64, rotation: f64) -> Self {
        Self {
            cx,
            cy,
            rx,
            ry,
            rotation,
        }
    }

    /// Check if ellipse is a circle
    pub fn is_circle(&self) -> bool {
        (self.rx - self.ry).abs() < f64::EPSILON
    }

    /// Check if ellipse is valid (positive radii)
    pub fn is_valid(&self) -> bool {
        self.rx > 0.0 && self.ry > 0.0
    }
}

impl ShapePrimitive for Ellipse {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Ellipse
    }

    fn bounds(&self) -> BoundingBox {
        // Calculate bounding box of rotated ellipse
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        let a = self.rx;
        let b = self.ry;

        // Maximum extents of rotated ellipse
        let max_dx = (a * a * cos_r * cos_r + b * b * sin_r * sin_r).sqrt();
        let max_dy = (a * a * sin_r * sin_r + b * b * cos_r * cos_r).sqrt();

        BoundingBox::new(
            self.cx - max_dx,
            self.cy - max_dy,
            self.cx + max_dx,
            self.cy + max_dy,
        )
    }

    fn to_path(&self) -> Path {
        let mut builder = PathBuilder::new();

        // Approximate ellipse with bezier curves
        let k = 0.552284749831; // Magic number for ellipse approximation

        // Transform control points for rotation
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
        // Transform point to ellipse coordinate system
        let dx = x - self.cx;
        let dy = y - self.cy;

        let cos_r = (-self.rotation).cos();
        let sin_r = (-self.rotation).sin();

        let local_x = dx * cos_r - dy * sin_r;
        let local_y = dx * sin_r + dy * cos_r;

        // Check if point is inside ellipse
        (local_x * local_x) / (self.rx * self.rx) + (local_y * local_y) / (self.ry * self.ry) <= 1.0
    }

    fn area(&self) -> f64 {
        std::f64::consts::PI * self.rx * self.ry
    }

    fn perimeter(&self) -> f64 {
        // Approximation of ellipse perimeter (Ramanujan's formula)
        let a = self.rx;
        let b = self.ry;
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));
        std::f64::consts::PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
    }

    fn transform(&mut self, transform: &Transform) {
        let (cx, cy) = transform.transform_point(self.cx, self.cy);
        self.cx = cx;
        self.cy = cy;


        self.rx *= transform.sx;
        self.ry *= transform.sy;


        self.rotation += transform.rotation;
    }

    fn transformed(&self, transform: &Transform) -> Self {
        let mut copy = *self;
        copy.transform(transform);
        copy
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


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Line {

    pub x1: f64,

    pub y1: f64,

    pub x2: f64,

    pub y2: f64,

    pub thickness: f64,
}

impl Line {

    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            thickness: 1.0,
        }
    }


    pub fn with_thickness(x1: f64, y1: f64, x2: f64, y2: f64, thickness: f64) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            thickness: thickness.max(0.0),
        }
    }


    pub fn length(&self) -> f64 {
        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        (dx * dx + dy * dy).sqrt()
    }


    pub fn angle(&self) -> f64 {
        (self.y2 - self.y1).atan2(self.x2 - self.x1)
    }


    pub fn is_point(&self) -> bool {
        (self.x1 - self.x2).abs() < f64::EPSILON && (self.y1 - self.y2).abs() < f64::EPSILON
    }
}

impl ShapePrimitive for Line {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Line
    }

    fn bounds(&self) -> BoundingBox {
        let half_thickness = self.thickness / 2.0;
        BoundingBox::new(
            self.x1.min(self.x2) - half_thickness,
            self.y1.min(self.y2) - half_thickness,
            self.x1.max(self.x2) + half_thickness,
            self.y1.max(self.y2) + half_thickness,
        )
    }

    fn to_path(&self) -> Path {
        let mut builder = PathBuilder::new();

        if self.thickness > 0.0 {

            let dx = self.x2 - self.x1;
            let dy = self.y2 - self.y1;
            let length = (dx * dx + dy * dy).sqrt();

            if length > 0.0 {
                let nx = -dy / length;
                let ny = dx / length;

                let half_thickness = self.thickness / 2.0;


                let x1 = self.x1 + nx * half_thickness;
                let y1 = self.y1 + ny * half_thickness;
                let x2 = self.x2 + nx * half_thickness;
                let y2 = self.y2 + ny * half_thickness;
                let x3 = self.x2 - nx * half_thickness;
                let y3 = self.y2 - ny * half_thickness;
                let x4 = self.x1 - nx * half_thickness;
                let y4 = self.y1 - ny * half_thickness;

                builder.move_to(x1, y1);
                builder.line_to(x2, y2);
                builder.line_to(x3, y3);
                builder.line_to(x4, y4);
                builder.close();
            }
        } else {

            builder.move_to(self.x1, self.y1);
            builder.line_to(self.x2, self.y2);
        }

        builder.build()
    }

    fn contains_point(&self, x: f64, y: f64) -> bool {
        if self.thickness == 0.0 {
            return false;
        }


        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        let length_sq = dx * dx + dy * dy;

        if length_sq == 0.0 {

            let dist_sq = (x - self.x1) * (x - self.x1) + (y - self.y1) * (y - self.y1);
            return dist_sq <= (self.thickness / 2.0).powi(2);
        }


        let t = ((x - self.x1) * dx + (y - self.y1) * dy) / length_sq;
        let t_clamped = t.clamp(0.0, 1.0);


        let closest_x = self.x1 + t_clamped * dx;
        let closest_y = self.y1 + t_clamped * dy;


        let dist_sq = (x - closest_x) * (x - closest_x) + (y - closest_y) * (y - closest_y);
        dist_sq <= (self.thickness / 2.0).powi(2)
    }

    fn area(&self) -> f64 {
        if self.thickness == 0.0 {
            0.0
        } else {
            self.length() * self.thickness
        }
    }

    fn perimeter(&self) -> f64 {
        if self.thickness == 0.0 {
            self.length()
        } else {
            2.0 * self.length() + 2.0 * self.thickness
        }
    }

    fn transform(&mut self, transform: &Transform) {
        let (x1, y1) = transform.transform_point(self.x1, self.y1);
        let (x2, y2) = transform.transform_point(self.x2, self.y2);

        self.x1 = x1;
        self.y1 = y1;
        self.x2 = x2;
        self.y2 = y2;


        self.thickness *= (transform.sx * transform.sy).sqrt() / 2.0;
    }

    fn transformed(&self, transform: &Transform) -> Self {
        let mut copy = *self;
        copy.transform(transform);
        copy
    }

    fn validate(&self) -> Result<(), String> {
        if self.thickness < 0.0 {
            return Err("Line thickness cannot be negative".to_string());
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {

    pub vertices: Vec<(f64, f64)>,

    pub closed: bool,
}

impl Polygon {

    pub fn new(vertices: Vec<(f64, f64)>) -> Self {
        Self {
            vertices,
            closed: true,
        }
    }


    pub fn open(vertices: Vec<(f64, f64)>) -> Self {
        Self {
            vertices,
            closed: false,
        }
    }


    pub fn regular(cx: f64, cy: f64, radius: f64, sides: usize) -> Self {
        let mut vertices = Vec::with_capacity(sides);
        let angle_step = 2.0 * std::f64::consts::PI / sides as f64;

        for i in 0..sides {
            let angle = i as f64 * angle_step - std::f64::consts::PI / 2.0;
            let x = cx + radius * angle.cos();
            let y = cy + radius * angle.sin();
            vertices.push((x, y));
        }

        Self::new(vertices)
    }


    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }


    pub fn is_valid(&self) -> bool {
        if self.closed {
            self.vertices.len() >= 3
        } else {
            self.vertices.len() >= 2
        }
    }


    pub fn is_convex(&self) -> bool {
        if self.vertices.len() < 3 {
            return false;
        }

        let n = self.vertices.len();
        let mut sign = 0;

        for i in 0..n {
            let p1 = self.vertices[i];
            let p2 = self.vertices[(i + 1) % n];
            let p3 = self.vertices[(i + 2) % n];

            let cross = (p2.0 - p1.0) * (p3.1 - p2.1) - (p2.1 - p1.1) * (p3.0 - p2.0);

            if cross != 0.0 {
                if sign == 0 {
                    sign = if cross > 0.0 { 1 } else { -1 };
                } else if cross.signum() != sign {
                    return false;
                }
            }
        }

        true
    }
}

impl ShapePrimitive for Polygon {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Polygon
    }

    fn bounds(&self) -> BoundingBox {
        if self.vertices.is_empty() {
            return BoundingBox::default();
        }

        let mut min_x = self.vertices[0].0;
        let mut min_y = self.vertices[0].1;
        let mut max_x = self.vertices[0].0;
        let mut max_y = self.vertices[0].1;

        for (x, y) in &self.vertices {
            min_x = min_x.min(*x);
            min_y = min_y.min(*y);
            max_x = max_x.max(*x);
            max_y = max_y.max(*y);
        }

        BoundingBox::new(min_x, min_y, max_x, max_y)
    }

    fn to_path(&self) -> Path {
        let mut builder = PathBuilder::new();

        if let Some((x, y)) = self.vertices.first() {
            builder.move_to(*x, *y);

            for (x, y) in self.vertices.iter().skip(1) {
                builder.line_to(*x, *y);
            }

            if self.closed && self.vertices.len() > 2 {
                builder.close();
            }
        }

        builder.build()
    }

    fn contains_point(&self, x: f64, y: f64) -> bool {
        if !self.closed || self.vertices.len() < 3 {
            return false;
        }


        let mut inside = false;
        let n = self.vertices.len();

        for i in 0..n {
            let p1 = self.vertices[i];
            let p2 = self.vertices[(i + 1) % n];

            if ((p1.1 > y) != (p2.1 > y)) &&
               (x < (p2.0 - p1.0) * (y - p1.1) / (p2.1 - p1.1) + p1.0) {
                inside = !inside;
            }
        }

        inside
    }

    fn area(&self) -> f64 {
        if !self.closed || self.vertices.len() < 3 {
            return 0.0;
        }


        let mut area = 0.0;
        let n = self.vertices.len();

        for i in 0..n {
            let p1 = self.vertices[i];
            let p2 = self.vertices[(i + 1) % n];
            area += p1.0 * p2.1 - p2.0 * p1.1;
        }

        area.abs() / 2.0
    }

    fn perimeter(&self) -> f64 {
        if self.vertices.len() < 2 {
            return 0.0;
        }

        let mut perimeter = 0.0;
        let n = self.vertices.len();

        for i in 0..n {
            let p1 = self.vertices[i];
            let p2 = self.vertices[(i + 1) % n];

            if !self.closed && i == n - 1 {
                break;
            }

            let dx = p2.0 - p1.0;
            let dy = p2.1 - p1.1;
            perimeter += (dx * dx + dy * dy).sqrt();
        }

        perimeter
    }

    fn transform(&mut self, transform: &Transform) {
        for (x, y) in &mut self.vertices {
            let (tx, ty) = transform.transform_point(*x, *y);
            *x = tx;
            *y = ty;
        }
    }

    fn transformed(&self, transform: &Transform) -> Self {
        let mut copy = self.clone();
        copy.transform(transform);
        copy
    }

    fn validate(&self) -> Result<(), String> {
        if self.vertices.is_empty() {
            return Err("Polygon must have at least one vertex".to_string());
        }

        if self.closed && self.vertices.len() < 3 {
            return Err("Closed polygon must have at least 3 vertices".to_string());
        }

        if !self.closed && self.vertices.len() < 2 {
            return Err("Open polygon must have at least 2 vertices".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle() {
        let rect = Rectangle::new(10.0, 20.0, 100.0, 50.0);

        assert_eq!(rect.shape_type(), ShapeType::Rectangle);
        assert_eq!(rect.area(), 5000.0);
        assert_eq!(rect.perimeter(), 300.0);
        assert!(rect.contains_point(60.0, 45.0));
        assert!(!rect.contains_point(5.0, 45.0));

        let bounds = rect.bounds();
        assert_eq!(bounds.min_x, 10.0);
        assert_eq!(bounds.min_y, 20.0);
        assert_eq!(bounds.max_x, 110.0);
        assert_eq!(bounds.max_y, 70.0);
    }

    #[test]
    fn test_circle() {
        let circle = Circle::new(50.0, 50.0, 25.0);

        assert_eq!(circle.shape_type(), ShapeType::Circle);
        assert!((circle.area() - std::f64::consts::PI * 625.0).abs() < 0.001);
        assert!((circle.perimeter() - 2.0 * std::f64::consts::PI * 25.0).abs() < 0.001);
        assert!(circle.contains_point(50.0, 50.0));
        assert!(circle.contains_point(70.0, 50.0));
        assert!(!circle.contains_point(80.0, 50.0));
    }

    #[test]
    fn test_ellipse() {
        let ellipse = Ellipse::new(50.0, 50.0, 40.0, 20.0);

        assert_eq!(ellipse.shape_type(), ShapeType::Ellipse);
        assert!(ellipse.contains_point(50.0, 50.0));
        assert!(ellipse.contains_point(85.0, 50.0));
        assert!(!ellipse.contains_point(95.0, 50.0));
    }

    #[test]
    fn test_line() {
        let line = Line::new(0.0, 0.0, 100.0, 100.0);

        assert_eq!(line.shape_type(), ShapeType::Line);
        assert!((line.length() - 141.421).abs() < 0.001);
        assert!((line.angle() - std::f64::consts::PI / 4.0).abs() < 0.001);
    }

    #[test]
    fn test_polygon() {
        let triangle = Polygon::regular(50.0, 50.0, 30.0, 3);

        assert_eq!(triangle.shape_type(), ShapeType::Polygon);
        assert_eq!(triangle.vertex_count(), 3);
        assert!(triangle.is_convex());
        assert!(triangle.contains_point(50.0, 50.0));

        let square = Polygon::new(vec![
            (0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)
        ]);
        assert_eq!(square.area(), 10000.0);
        assert_eq!(square.perimeter(), 400.0);
    }

    #[test]
    fn test_transform() {
        let rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        let transform = Transform::translation(5.0, 10.0);

        let transformed = rect.transformed(&transform);
        assert_eq!(transformed.x, 5.0);
        assert_eq!(transformed.y, 10.0);
        assert_eq!(transformed.width, 10.0);
        assert_eq!(transformed.height, 10.0);
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(10.0, 20.0, 100.0, 80.0);

        assert_eq!(bbox.width(), 90.0);
        assert_eq!(bbox.height(), 60.0);
        assert_eq!(bbox.center(), (55.0, 50.0));
        assert!(bbox.contains_point(50.0, 50.0));
        assert!(!bbox.contains_point(5.0, 50.0));

        let other = BoundingBox::new(50.0, 40.0, 150.0, 90.0);
        assert!(bbox.intersects(&other));

        let union = bbox.union(&other);
        assert_eq!(union.min_x, 10.0);
        assert_eq!(union.max_x, 150.0);
    }

    #[test]
    fn test_shape_validation() {
        let valid_rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        assert!(valid_rect.validate().is_ok());

        let invalid_rect = Rectangle::new(0.0, 0.0, -10.0, 10.0);
        assert!(invalid_rect.validate().is_err());

        let valid_circle = Circle::new(0.0, 0.0, 10.0);
        assert!(valid_circle.validate().is_ok());

        let invalid_circle = Circle::new(0.0, 0.0, -10.0);
        assert!(invalid_circle.validate().is_err());
    }
}
