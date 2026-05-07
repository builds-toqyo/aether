//! Path and bezier curve operations
//! 
//! This module provides path building and manipulation capabilities
//! including bezier curves, path segments, and path operations.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Path segment types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathSegmentType {
    /// Move to point
    MoveTo,
    /// Line to point
    LineTo,
    /// Quadratic bezier curve
    QuadraticTo,
    /// Cubic bezier curve
    CubicTo,
    /// Close path
    Close,
}

impl fmt::Display for PathSegmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathSegmentType::MoveTo => write!(f, "MoveTo"),
            PathSegmentType::LineTo => write!(f, "LineTo"),
            PathSegmentType::QuadraticTo => write!(f, "QuadraticTo"),
            PathSegmentType::CubicTo => write!(f, "CubicTo"),
            PathSegmentType::Close => write!(f, "Close"),
        }
    }
}

/// Path segment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathSegment {
    /// Segment type
    pub segment_type: PathSegmentType,
    /// End point coordinates
    pub x: f64,
    pub y: f64,
    /// Control point 1 (for cubic bezier)
    pub cp1_x: f64,
    pub cp1_y: f64,
    /// Control point 2 (for cubic bezier)
    pub cp2_x: f64,
    pub cp2_y: f64,
}

impl PathSegment {
    /// Create move to segment
    pub fn move_to(x: f64, y: f64) -> Self {
        Self {
            segment_type: PathSegmentType::MoveTo,
            x,
            y,
            cp1_x: 0.0,
            cp1_y: 0.0,
            cp2_x: 0.0,
            cp2_y: 0.0,
        }
    }
    
    /// Create line to segment
    pub fn line_to(x: f64, y: f64) -> Self {
        Self {
            segment_type: PathSegmentType::LineTo,
            x,
            y,
            cp1_x: 0.0,
            cp1_y: 0.0,
            cp2_x: 0.0,
            cp2_y: 0.0,
        }
    }
    
    /// Create quadratic bezier segment
    pub fn quadratic_to(x: f64, y: f64, cp_x: f64, cp_y: f64) -> Self {
        Self {
            segment_type: PathSegmentType::QuadraticTo,
            x,
            y,
            cp1_x: cp_x,
            cp1_y: cp_y,
            cp2_x: 0.0,
            cp2_y: 0.0,
        }
    }
    
    /// Create cubic bezier segment
    pub fn cubic_to(x: f64, y: f64, cp1_x: f64, cp1_y: f64, cp2_x: f64, cp2_y: f64) -> Self {
        Self {
            segment_type: PathSegmentType::CubicTo,
            x,
            y,
            cp1_x,
            cp1_y,
            cp2_x,
            cp2_y,
        }
    }
    
    /// Create close segment
    pub fn close() -> Self {
        Self {
            segment_type: PathSegmentType::Close,
            x: 0.0,
            y: 0.0,
            cp1_x: 0.0,
            cp1_y: 0.0,
            cp2_x: 0.0,
            cp2_y: 0.0,
        }
    }
    
    /// Get length of segment (approximate)
    pub fn length(&self, start_x: f64, start_y: f64) -> f64 {
        match self.segment_type {
            PathSegmentType::MoveTo => 0.0,
            PathSegmentType::LineTo => {
                let dx = self.x - start_x;
                let dy = self.y - start_y;
                (dx * dx + dy * dy).sqrt()
            }
            PathSegmentType::QuadraticTo => {
                // Approximate quadratic bezier length
                self.approximate_bezier_length(start_x, start_y, self.cp1_x, self.cp1_y, self.x, self.y, 10)
            }
            PathSegmentType::CubicTo => {
                // Approximate cubic bezier length
                self.approximate_bezier_length(start_x, start_y, self.cp1_x, self.cp1_y, self.cp2_x, self.cp2_y, self.x, self.y, 20)
            }
            PathSegmentType::Close => 0.0,
        }
    }
    
    /// Approximate bezier curve length using subdivision
    fn approximate_bezier_length(&self, x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, subdivisions: usize) -> f64 {
        let mut length = 0.0;
        let mut prev_x = x0;
        let mut prev_y = y0;
        
        for i in 1..=subdivisions {
            let t = i as f64 / subdivisions as f64;
            let point = self.evaluate_bezier_point(t, x0, y0, x1, y1, x2, y2, x3, y3);
            let dx = point.0 - prev_x;
            let dy = point.1 - prev_y;
            length += (dx * dx + dy * dy).sqrt();
            prev_x = point.0;
            prev_y = point.1;
        }
        
        length
    }
    
    /// Evaluate point on bezier curve at parameter t
    fn evaluate_bezier_point(&self, t: f64, x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> (f64, f64) {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        
        let x = mt3 * x0 + 3.0 * mt2 * t * x1 + 3.0 * mt * t2 * x2 + t3 * x3;
        let y = mt3 * y0 + 3.0 * mt2 * t * y1 + 3.0 * mt * t2 * y2 + t3 * y3;
        
        (x, y)
    }
    
    /// Sample points along the segment
    pub fn sample_points(&self, start_x: f64, start_y: f64, num_samples: usize) -> Vec<(f64, f64)> {
        let mut points = Vec::with_capacity(num_samples);
        
        match self.segment_type {
            PathSegmentType::MoveTo => {
                points.push((self.x, self.y));
            }
            PathSegmentType::LineTo => {
                for i in 0..num_samples {
                    let t = i as f64 / (num_samples - 1) as f64;
                    let x = start_x + t * (self.x - start_x);
                    let y = start_y + t * (self.y - start_y);
                    points.push((x, y));
                }
            }
            PathSegmentType::QuadraticTo => {
                for i in 0..num_samples {
                    let t = i as f64 / (num_samples - 1) as f64;
                    let point = self.evaluate_quadratic_bezier(t, start_x, start_y, self.cp1_x, self.cp1_y, self.x, self.y);
                    points.push(point);
                }
            }
            PathSegmentType::CubicTo => {
                for i in 0..num_samples {
                    let t = i as f64 / (num_samples - 1) as f64;
                    let point = self.evaluate_bezier_point(t, start_x, start_y, self.cp1_x, self.cp1_y, self.cp2_x, self.cp2_y, self.x, self.y);
                    points.push(point);
                }
            }
            PathSegmentType::Close => {
                // Close segment doesn't generate points
            }
        }
        
        points
    }
    
    /// Evaluate quadratic bezier point
    fn evaluate_quadratic_bezier(&self, t: f64, x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> (f64, f64) {
        let mt = 1.0 - t;
        let x = mt * mt * x0 + 2.0 * mt * t * x1 + t * t * x2;
        let y = mt * mt * y0 + 2.0 * mt * t * y1 + t * t * y2;
        (x, y)
    }
}

/// Path structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Path {
    /// Path segments
    pub segments: Vec<PathSegment>,
    /// Whether path is closed
    pub closed: bool,
    /// Current position for building
    current_x: f64,
    current_y: f64,
    /// Starting position
    start_x: f64,
    start_y: f64,
}

impl Path {
    /// Create empty path
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            closed: false,
            current_x: 0.0,
            current_y: 0.0,
            start_x: 0.0,
            start_y: 0.0,
        }
    }
    
    /// Get total length of path
    pub fn length(&self) -> f64 {
        let mut length = 0.0;
        let mut current_x = self.start_x;
        let mut current_y = self.start_y;
        
        for segment in &self.segments {
            length += segment.length(current_x, current_y);
            
            match segment.segment_type {
                PathSegmentType::MoveTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::LineTo | PathSegmentType::QuadraticTo | PathSegmentType::CubicTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::Close => {
                    current_x = self.start_x;
                    current_y = self.start_y;
                }
            }
        }
        
        length
    }
    
    /// Get bounding box of path
    pub fn bounds(&self) -> crate::shapes::primitives::BoundingBox {
        if self.segments.is_empty() {
            return crate::shapes::primitives::BoundingBox::default();
        }
        
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        
        let mut current_x = self.start_x;
        let mut current_y = self.start_y;
        
        // Include start position
        min_x = min_x.min(current_x);
        min_y = min_y.min(current_y);
        max_x = max_x.max(current_x);
        max_y = max_y.max(current_y);
        
        for segment in &self.segments {
            // Sample points along segment to find bounds
            let samples = match segment.segment_type {
                PathSegmentType::MoveTo => 1,
                PathSegmentType::LineTo => 2,
                PathSegmentType::QuadraticTo => 10,
                PathSegmentType::CubicTo => 20,
                PathSegmentType::Close => 1,
            };
            
            let points = segment.sample_points(current_x, current_y, samples);
            
            for (x, y) in points {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
            
            match segment.segment_type {
                PathSegmentType::MoveTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::LineTo | PathSegmentType::QuadraticTo | PathSegmentType::CubicTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::Close => {
                    current_x = self.start_x;
                    current_y = self.start_y;
                }
            }
        }
        
        crate::shapes::primitives::BoundingBox::new(min_x, min_y, max_x, max_y)
    }
    
    /// Check if point is inside path (using ray casting)
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        if !self.closed || self.segments.is_empty() {
            return false;
        }
        
        let mut inside = false;
        let mut current_x = self.start_x;
        let mut current_y = self.start_y;
        
        // Collect all line segments from path
        let mut edges = Vec::new();
        
        for segment in &self.segments {
            match segment.segment_type {
                PathSegmentType::MoveTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::LineTo => {
                    edges.push((current_x, current_y, segment.x, segment.y));
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::QuadraticTo => {
                    // Approximate quadratic with line segments
                    let points = segment.sample_points(current_x, current_y, 10);
                    for i in 1..points.len() {
                        edges.push((points[i-1].0, points[i-1].1, points[i].0, points[i].1));
                    }
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::CubicTo => {
                    // Approximate cubic with line segments
                    let points = segment.sample_points(current_x, current_y, 20);
                    for i in 1..points.len() {
                        edges.push((points[i-1].0, points[i-1].1, points[i].0, points[i].1));
                    }
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::Close => {
                    edges.push((current_x, current_y, self.start_x, self.start_y));
                    current_x = self.start_x;
                    current_y = self.start_y;
                }
            }
        }
        
        // Ray casting algorithm
        for (x1, y1, x2, y2) in edges {
            if ((y1 > y) != (y2 > y)) && 
               (x < (x2 - x1) * (y - y1) / (y2 - y1) + x1) {
                inside = !inside;
            }
        }
        
        inside
    }
    
    /// Get area of path (using shoelace formula)
    pub fn area(&self) -> f64 {
        if !self.closed || self.segments.is_empty() {
            return 0.0;
        }
        
        let mut area = 0.0;
        let mut current_x = self.start_x;
        let mut current_y = self.start_y;
        
        // Collect vertices from path
        let mut vertices = vec![(current_x, current_y)];
        
        for segment in &self.segments {
            match segment.segment_type {
                PathSegmentType::MoveTo => {
                    vertices.push((segment.x, segment.y));
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::LineTo => {
                    vertices.push((segment.x, segment.y));
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::QuadraticTo => {
                    // Approximate quadratic with vertices
                    let points = segment.sample_points(current_x, current_y, 10);
                    for point in points.iter().skip(1) {
                        vertices.push(*point);
                    }
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::CubicTo => {
                    // Approximate cubic with vertices
                    let points = segment.sample_points(current_x, current_y, 20);
                    for point in points.iter().skip(1) {
                        vertices.push(*point);
                    }
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::Close => {
                    // Don't add new vertex, path closes to start
                    current_x = self.start_x;
                    current_y = self.start_y;
                }
            }
        }
        
        // Apply shoelace formula
        for i in 0..vertices.len() {
            let p1 = vertices[i];
            let p2 = vertices[(i + 1) % vertices.len()];
            area += p1.0 * p2.1 - p2.0 * p1.1;
        }
        
        area.abs() / 2.0
    }
    
    /// Sample points along entire path
    pub fn sample_points(&self, num_points: usize) -> Vec<(f64, f64)> {
        let mut points = Vec::new();
        let total_length = self.length();
        
        if total_length == 0.0 {
            return points;
        }
        
        let target_segment_length = total_length / (num_points - 1) as f64;
        let mut accumulated_length = 0.0;
        let mut current_x = self.start_x;
        let mut current_y = self.start_y;
        
        points.push((current_x, current_y));
        
        for segment in &self.segments {
            let segment_length = segment.length(current_x, current_y);
            
            if accumulated_length + segment_length >= target_segment_length {
                let samples = ((segment_length / target_segment_length) as usize).max(1);
                let segment_points = segment.sample_points(current_x, current_y, samples + 1);
                
                for point in segment_points.iter().skip(1) {
                    if points.len() >= num_points {
                        break;
                    }
                    points.push(*point);
                }
            }
            
            match segment.segment_type {
                PathSegmentType::MoveTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::LineTo | PathSegmentType::QuadraticTo | PathSegmentType::CubicTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                PathSegmentType::Close => {
                    current_x = self.start_x;
                    current_y = self.start_y;
                }
            }
            
            accumulated_length += segment_length;
        }
        
        points
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

/// Path builder for fluent path construction
pub struct PathBuilder {
    path: Path,
}

impl PathBuilder {
    /// Create new path builder
    pub fn new() -> Self {
        Self {
            path: Path::new(),
        }
    }
    
    /// Move to point
    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        let segment = PathSegment::move_to(x, y);
        
        if self.path.segments.is_empty() {
            self.path.start_x = x;
            self.path.start_y = y;
        }
        
        self.path.segments.push(segment);
        self.path.current_x = x;
        self.path.current_y = y;
        self
    }
    
    /// Line to point
    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        let segment = PathSegment::line_to(x, y);
        self.path.segments.push(segment);
        self.path.current_x = x;
        self.path.current_y = y;
        self
    }
    
    /// Quadratic bezier to point
    pub fn quadratic_to(&mut self, x: f64, y: f64, cp_x: f64, cp_y: f64) -> &mut Self {
        let segment = PathSegment::quadratic_to(x, y, cp_x, cp_y);
        self.path.segments.push(segment);
        self.path.current_x = x;
        self.path.current_y = y;
        self
    }
    
    /// Cubic bezier to point
    pub fn bezier_to(&mut self, x: f64, y: f64, cp1_x: f64, cp1_y: f64, cp2_x: f64, cp2_y: f64) -> &mut Self {
        let segment = PathSegment::cubic_to(x, y, cp1_x, cp1_y, cp2_x, cp2_y);
        self.path.segments.push(segment);
        self.path.current_x = x;
        self.path.current_y = y;
        self
    }
    
    /// Close path
    pub fn close(&mut self) -> &mut Self {
        let segment = PathSegment::close();
        self.path.segments.push(segment);
        self.path.closed = true;
        self.path.current_x = self.path.start_x;
        self.path.current_y = self.path.start_y;
        self
    }
    
    /// Add rectangle
    pub fn rectangle(&mut self, x: f64, y: f64, width: f64, height: f64) -> &mut Self {
        self.move_to(x, y);
        self.line_to(x + width, y);
        self.line_to(x + width, y + height);
        self.line_to(x, y + height);
        self.close();
        self
    }
    
    /// Add circle (approximated with bezier curves)
    pub fn circle(&mut self, cx: f64, cy: f64, radius: f64) -> &mut Self {
        let k = 0.552284749831; // Magic number for circle approximation
        
        self.move_to(cx + radius, cy);
        self.bezier_to(
            cx + radius, cy - k * radius,
            cx + k * radius, cy - radius,
            cx, cy - radius,
        );
        self.bezier_to(
            cx - k * radius, cy - radius,
            cx - radius, cy - k * radius,
            cx - radius, cy,
        );
        self.bezier_to(
            cx - radius, cy + k * radius,
            cx - k * radius, cy + radius,
            cx, cy + radius,
        );
        self.bezier_to(
            cx + k * radius, cy + radius,
            cx + radius, cy + k * radius,
            cx + radius, cy,
        );
        self.close();
        self
    }
    
    /// Add ellipse (approximated with bezier curves)
    pub fn ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64) -> &mut Self {
        let k = 0.552284749831; // Magic number for ellipse approximation
        
        self.move_to(cx + rx, cy);
        self.bezier_to(
            cx + rx, cy - k * ry,
            cx + k * rx, cy - ry,
            cx, cy - ry,
        );
        self.bezier_to(
            cx - k * rx, cy - ry,
            cx - rx, cy - k * ry,
            cx - rx, cy,
        );
        self.bezier_to(
            cx - rx, cy + k * ry,
            cx - k * rx, cy + ry,
            cx, cy + ry,
        );
        self.bezier_to(
            cx + k * rx, cy + ry,
            cx + rx, cy + k * ry,
            cx + rx, cy,
        );
        self.close();
        self
    }
    
    /// Add regular polygon
    pub fn regular_polygon(&mut self, cx: f64, cy: f64, radius: f64, sides: usize) -> &mut Self {
        let angle_step = 2.0 * std::f64::consts::PI / sides as f64;
        
        for i in 0..sides {
            let angle = i as f64 * angle_step - std::f64::consts::PI / 2.0;
            let x = cx + radius * angle.cos();
            let y = cy + radius * angle.sin();
            
            if i == 0 {
                self.move_to(x, y);
            } else {
                self.line_to(x, y);
            }
        }
        
        self.close();
        self
    }
    
    /// Build the path
    pub fn build(self) -> Path {
        self.path
    }
}

impl Default for PathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Bezier curve utilities
pub struct BezierCurve;

impl BezierCurve {
    /// Evaluate cubic bezier at parameter t
    pub fn cubic_bezier_point(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
    ) -> (f64, f64) {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        
        let x = mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0;
        let y = mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1;
        
        (x, y)
    }
    
    /// Evaluate quadratic bezier at parameter t
    pub fn quadratic_bezier_point(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
    ) -> (f64, f64) {
        let mt = 1.0 - t;
        let x = mt * mt * p0.0 + 2.0 * mt * t * p1.0 + t * t * p2.0;
        let y = mt * mt * p0.1 + 2.0 * mt * t * p1.1 + t * t * p2.1;
        (x, y)
    }
    
    /// Get tangent vector of cubic bezier at parameter t
    pub fn cubic_bezier_tangent(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
    ) -> (f64, f64) {
        let mt = 1.0 - t;
        let t2 = t * t;
        let mt2 = mt * mt;
        
        let x = 3.0 * mt2 * (p1.0 - p0.0) + 6.0 * mt * t * (p2.0 - p1.0) + 3.0 * t2 * (p3.0 - p2.0);
        let y = 3.0 * mt2 * (p1.1 - p0.1) + 6.0 * mt * t * (p2.1 - p1.1) + 3.0 * t2 * (p3.1 - p2.1);
        
        (x, y)
    }
    
    /// Get tangent vector of quadratic bezier at parameter t
    pub fn quadratic_bezier_tangent(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
    ) -> (f64, f64) {
        let mt = 1.0 - t;
        let x = 2.0 * mt * (p1.0 - p0.0) + 2.0 * t * (p2.0 - p1.0);
        let y = 2.0 * mt * (p1.1 - p0.1) + 2.0 * t * (p2.1 - p1.1);
        (x, y)
    }
    
    /// Subdivide cubic bezier at parameter t
    pub fn subdivide_cubic_bezier(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
    ) -> ((f64, f64, f64, f64, f64, f64, f64, f64), (f64, f64, f64, f64, f64, f64, f64, f64)) {
        let p01 = ((p0.0 + p1.0) / 2.0, (p0.1 + p1.1) / 2.0);
        let p12 = ((p1.0 + p2.0) / 2.0, (p1.1 + p2.1) / 2.0);
        let p23 = ((p2.0 + p3.0) / 2.0, (p2.1 + p3.1) / 2.0);
        
        let p012 = ((p01.0 + p12.0) / 2.0, (p01.1 + p12.1) / 2.0);
        let p123 = ((p12.0 + p23.0) / 2.0, (p12.1 + p23.1) / 2.0);
        
        let p0123 = ((p012.0 + p123.0) / 2.0, (p012.1 + p123.1) / 2.0);
        
        let left = (p0.0, p0.1, p01.0, p01.1, p012.0, p012.1, p0123.0, p0123.1);
        let right = (p0123.0, p0123.1, p123.0, p123.1, p23.0, p23.1, p3.0, p3.1);
        
        (left, right)
    }
    
    /// Approximate bezier curve with line segments
    pub fn approximate_with_lines(
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
        tolerance: f64,
    ) -> Vec<(f64, f64)> {
        let mut points = Vec::new();
        Self::approximate_recursive(p0, p1, p2, p3, tolerance, &mut points);
        points
    }
    
    /// Recursive approximation helper
    fn approximate_recursive(
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
        tolerance: f64,
        points: &mut Vec<(f64, f64)>,
    ) {
        let mid = Self::cubic_bezier_point(0.5, p0, p1, p2, p3);
        
        // Check if curve is close enough to a straight line
        let d1 = Self::point_to_line_distance(mid, p0, p3);
        let d2 = Self::point_to_line_distance(p1, p0, p3);
        let d3 = Self::point_to_line_distance(p2, p0, p3);
        
        if d1.max(d2).max(d3) <= tolerance {
            points.push(p3);
        } else {
            let (left, right) = Self::subdivide_cubic_bezier(0.5, p0, p1, p2, p3);
            
            Self::approximate_recursive(
                (left.0, left.1),
                (left.2, left.3),
                (left.4, left.5),
                (left.6, left.7),
                tolerance,
                points,
            );
            
            Self::approximate_recursive(
                (right.0, right.1),
                (right.2, right.3),
                (right.4, right.5),
                (right.6, right.7),
                tolerance,
                points,
            );
        }
    }
    
    /// Calculate distance from point to line
    fn point_to_line_distance(point: (f64, f64), line_start: (f64, f64), line_end: (f64, f64)) -> f64 {
        let dx = line_end.0 - line_start.0;
        let dy = line_end.1 - line_start.1;
        
        if dx == 0.0 && dy == 0.0 {
            // Line is a point
            let dx = point.0 - line_start.0;
            let dy = point.1 - line_start.1;
            return (dx * dx + dy * dy).sqrt();
        }
        
        let t = ((point.0 - line_start.0) * dx + (point.1 - line_start.1) * dy) / (dx * dx + dy * dy);
        let t_clamped = t.clamp(0.0, 1.0);
        
        let closest_x = line_start.0 + t_clamped * dx;
        let closest_y = line_start.1 + t_clamped * dy;
        
        let dx = point.0 - closest_x;
        let dy = point.1 - closest_y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_path_builder() {
        let mut builder = PathBuilder::new();
        builder.move_to(0.0, 0.0);
        builder.line_to(100.0, 0.0);
        builder.line_to(100.0, 100.0);
        builder.line_to(0.0, 100.0);
        builder.close();
        
        let path = builder.build();
        assert_eq!(path.segments.len(), 5);
        assert!(path.closed);
        assert_eq!(path.area(), 10000.0);
    }
    
    #[test]
    fn test_bezier_curve() {
        let p0 = (0.0, 0.0);
        let p1 = (50.0, 0.0);
        let p2 = (100.0, 50.0);
        let p3 = (100.0, 100.0);
        
        let point = BezierCurve::cubic_bezier_point(0.5, p0, p1, p2, p3);
        assert!(point.0 > 40.0 && point.0 < 60.0);
        assert!(point.1 > 20.0 && point.1 < 30.0);
        
        let tangent = BezierCurve::cubic_bezier_tangent(0.5, p0, p1, p2, p3);
        assert!(tangent.0 > 0.0); // Should be pointing right
        assert!(tangent.1 > 0.0); // Should be pointing up
    }
    
    #[test]
    fn test_circle_path() {
        let mut builder = PathBuilder::new();
        builder.circle(50.0, 50.0, 25.0);
        
        let path = builder.build();
        assert!(path.closed);
        assert!(path.contains_point(50.0, 50.0));
        assert!(path.contains_point(70.0, 50.0));
        assert!(!path.contains_point(80.0, 50.0));
        
        let area = path.area();
        let expected_area = std::f64::consts::PI * 25.0 * 25.0;
        assert!((area - expected_area).abs() < 100.0); // Allow some approximation error
    }
    
    #[test]
    fn test_regular_polygon() {
        let mut builder = PathBuilder::new();
        builder.regular_polygon(50.0, 50.0, 30.0, 6);
        
        let path = builder.build();
        assert_eq!(path.segments.len(), 7); // 6 sides + close
        assert!(path.closed);
        assert!(path.contains_point(50.0, 50.0));
    }
    
    #[test]
    fn test_path_sampling() {
        let mut builder = PathBuilder::new();
        builder.move_to(0.0, 0.0);
        builder.line_to(100.0, 0.0);
        builder.line_to(100.0, 100.0);
        builder.line_to(0.0, 100.0);
        builder.close();
        
        let path = builder.build();
        let points = path.sample_points(10);
        
        assert_eq!(points.len(), 10);
        assert_eq!(points[0], (0.0, 0.0));
        assert_eq!(points[9], (0.0, 100.0)); // Should end at start
    }
}
