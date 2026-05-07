//! Path structure and operations
//! 
//! This module provides the main Path structure with comprehensive
//! geometric operations including area calculation, point testing,
//! and path analysis.

use std::fmt;

/// Path structure
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<super::segments::PathSegment>,
    pub closed: bool,
    pub current_x: f64,
    pub current_y: f64,
    pub start_x: f64,
    pub start_y: f64,
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
    
    /// Create path from segments
    pub fn from_segments(segments: Vec<super::segments::PathSegment>) -> Self {
        let mut path = Self::new();
        path.segments = segments;
        
        // Update positions based on segments
        for segment in &path.segments {
            match segment.segment_type {
                super::segments::PathSegmentType::MoveTo => {
                    if path.segments.first() == Some(segment) {
                        path.start_x = segment.x;
                        path.start_y = segment.y;
                    }
                    path.current_x = segment.x;
                    path.current_y = segment.y;
                }
                super::segments::PathSegmentType::LineTo |
                super::segments::PathSegmentType::QuadraticTo |
                super::segments::PathSegmentType::CubicTo => {
                    path.current_x = segment.x;
                    path.current_y = segment.y;
                }
                super::segments::PathSegmentType::Close => {
                    path.closed = true;
                    path.current_x = path.start_x;
                    path.current_y = path.start_y;
                }
            }
        }
        
        path
    }
    
    /// Get total length of path
    pub fn length(&self) -> f64 {
        let mut length = 0.0;
        let mut current_x = self.start_x;
        let mut current_y = self.start_y;
        
        for segment in &self.segments {
            length += segment.length(current_x, current_y);
            
            match segment.segment_type {
                super::segments::PathSegmentType::MoveTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::LineTo | 
                super::segments::PathSegmentType::QuadraticTo | 
                super::segments::PathSegmentType::CubicTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::Close => {
                    current_x = self.start_x;
                    current_y = self.start_y;
                }
            }
        }
        
        length
    }
    
    /// Get bounding box of path
    pub fn bounds(&self) -> crate::shapes::primitives::transform::BoundingBox {
        if self.segments.is_empty() {
            return crate::shapes::primitives::transform::BoundingBox::default();
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
                super::segments::PathSegmentType::MoveTo => 1,
                super::segments::PathSegmentType::LineTo => 2,
                super::segments::PathSegmentType::QuadraticTo => 10,
                super::segments::PathSegmentType::CubicTo => 20,
                super::segments::PathSegmentType::Close => 1,
            };
            
            let points = segment.sample_points(current_x, current_y, samples);
            
            for (x, y) in points {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
            
            match segment.segment_type {
                super::segments::PathSegmentType::MoveTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::LineTo | 
                super::segments::PathSegmentType::QuadraticTo | 
                super::segments::PathSegmentType::CubicTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::Close => {
                    current_x = self.start_x;
                    current_y = self.start_y;
                }
            }
        }
        
        crate::shapes::primitives::transform::BoundingBox::new(min_x, min_y, max_x, max_y)
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
                super::segments::PathSegmentType::MoveTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::LineTo => {
                    edges.push((current_x, current_y, segment.x, segment.y));
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::QuadraticTo => {
                    // Approximate quadratic with line segments
                    let points = segment.sample_points(current_x, current_y, 10);
                    for i in 1..points.len() {
                        edges.push((points[i-1].0, points[i-1].1, points[i].0, points[i].1));
                    }
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::CubicTo => {
                    // Approximate cubic with line segments
                    let points = segment.sample_points(current_x, current_y, 20);
                    for i in 1..points.len() {
                        edges.push((points[i-1].0, points[i-1].1, points[i].0, points[i].1));
                    }
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::Close => {
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
                super::segments::PathSegmentType::MoveTo => {
                    vertices.push((segment.x, segment.y));
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::LineTo => {
                    vertices.push((segment.x, segment.y));
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::QuadraticTo => {
                    // Approximate quadratic with vertices
                    let points = segment.sample_points(current_x, current_y, 10);
                    for point in points.iter().skip(1) {
                        vertices.push(*point);
                    }
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::CubicTo => {
                    // Approximate cubic with vertices
                    let points = segment.sample_points(current_x, current_y, 20);
                    for point in points.iter().skip(1) {
                        vertices.push(*point);
                    }
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::Close => {
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
                super::segments::PathSegmentType::MoveTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::LineTo | 
                super::segments::PathSegmentType::QuadraticTo | 
                super::segments::PathSegmentType::CubicTo => {
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::Close => {
                    current_x = self.start_x;
                    current_y = self.start_y;
                }
            }
            
            accumulated_length += segment_length;
        }
        
        points
    }
    
    /// Get vertices of path (approximate bezier curves)
    pub fn vertices(&self, tolerance: f64) -> Vec<(f64, f64)> {
        let mut vertices = Vec::new();
        let mut current_x = self.start_x;
        let mut current_y = self.start_y;
        
        vertices.push((current_x, current_y));
        
        for segment in &self.segments {
            match segment.segment_type {
                super::segments::PathSegmentType::MoveTo => {
                    vertices.push((segment.x, segment.y));
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::LineTo => {
                    vertices.push((segment.x, segment.y));
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::QuadraticTo => {
                    let points = segment.sample_points(current_x, current_y, 10);
                    for point in points.iter().skip(1) {
                        vertices.push(*point);
                    }
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::CubicTo => {
                    let points = segment.sample_points(current_x, current_y, 20);
                    for point in points.iter().skip(1) {
                        vertices.push(*point);
                    }
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::Close => {
                    // Don't add vertex, path closes to start
                    current_x = self.start_x;
                    current_y = self.start_y;
                }
            }
        }
        
        vertices
    }
    
    /// Simplify path by removing redundant vertices
    pub fn simplify(&self, tolerance: f64) -> Path {
        let vertices = self.vertices(tolerance);
        
        if vertices.len() < 3 {
            return self.clone();
        }
        
        let mut simplified_segments = Vec::new();
        let mut current_x = self.start_x;
        let mut current_y = self.start_y;
        
        for segment in &self.segments {
            match segment.segment_type {
                super::segments::PathSegmentType::MoveTo => {
                    simplified_segments.push(segment.clone());
                    current_x = segment.x;
                    current_y = segment.y;
                }
                super::segments::PathSegmentType::LineTo => {
                    // Check if this segment is necessary
                    if let Some(last_segment) = simplified_segments.last() {
                        let distance = ((segment.x - current_x).powi(2) + (segment.y - current_y).powi(2)).sqrt();
                        
                        if distance > tolerance {
                            simplified_segments.push(segment.clone());
                            current_x = segment.x;
                            current_y = segment.y;
                        }
                    } else {
                        simplified_segments.push(segment.clone());
                        current_x = segment.x;
                        current_y = segment.y;
                    }
                }
                super::segments::PathSegmentType::Close => {
                    simplified_segments.push(segment.clone());
                }
                // Keep bezier segments as-is for now
                super::segments::PathSegmentType::QuadraticTo | 
                super::segments::PathSegmentType::CubicTo => {
                    simplified_segments.push(segment.clone());
                    current_x = segment.x;
                    current_y = segment.y;
                }
            }
        }
        
        let mut simplified_path = Path::new();
        simplified_path.segments = simplified_segments;
        simplified_path.closed = self.closed;
        simplified_path.start_x = self.start_x;
        simplified_path.start_y = self.start_y;
        
        simplified_path
    }
    
    /// Reverse path direction
    pub fn reverse(&self) -> Path {
        let mut reversed = Path::new();
        reversed.closed = self.closed;
        reversed.start_x = self.current_x;
        reversed.start_y = self.current_y;
        
        // Reverse segments and adjust their types
        for segment in self.segments.iter().rev() {
            match segment.segment_type {
                super::segments::PathSegmentType::MoveTo => {
                    reversed.segments.push(super::segments::PathSegment::move_to(segment.x, segment.y));
                }
                super::segments::PathSegmentType::LineTo => {
                    reversed.segments.push(super::segments::PathSegment::line_to(segment.x, segment.y));
                }
                super::segments::PathSegmentType::QuadraticTo => {
                    // Swap control point and end point for quadratic
                    reversed.segments.push(super::segments::PathSegment::quadratic_to(
                        segment.cp1_x, segment.cp1_y, segment.x, segment.y
                    ));
                }
                super::segments::PathSegmentType::CubicTo => {
                    // Swap control points and end point for cubic
                    reversed.segments.push(super::segments::PathSegment::cubic_to(
                        segment.cp2_x, segment.cp2_y, segment.cp1_x, segment.cp1_y, segment.x, segment.y
                    ));
                }
                super::segments::PathSegmentType::Close => {
                    reversed.segments.push(super::segments::PathSegment::close());
                }
            }
        }
        
        reversed
    }
    
    /// Get number of segments
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
    
    /// Check if path is empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
    
    /// Get segment at index
    pub fn segment(&self, index: usize) -> Option<&super::segments::PathSegment> {
        self.segments.get(index)
    }
    
    /// Add segment to path
    pub fn add_segment(&mut self, segment: super::segments::PathSegment) {
        if self.segments.is_empty() && matches!(segment.segment_type, super::segments::PathSegmentType::MoveTo) {
            self.start_x = segment.x;
            self.start_y = segment.y;
        }
        
        self.segments.push(segment);
        
        // Update current position
        if let Some(last_segment) = self.segments.last() {
            match last_segment.segment_type {
                super::segments::PathSegmentType::MoveTo => {
                    self.current_x = last_segment.x;
                    self.current_y = last_segment.y;
                }
                super::segments::PathSegmentType::LineTo |
                super::segments::PathSegmentType::QuadraticTo |
                super::segments::PathSegmentType::CubicTo => {
                    self.current_x = last_segment.x;
                    self.current_y = last_segment.y;
                }
                super::segments::PathSegmentType::Close => {
                    self.closed = true;
                    self.current_x = self.start_x;
                    self.current_y = self.start_y;
                }
            }
        }
    }
    
    /// Clear all segments
    pub fn clear(&mut self) {
        self.segments.clear();
        self.closed = false;
        self.current_x = 0.0;
        self.current_y = 0.0;
        self.start_x = 0.0;
        self.start_y = 0.0;
    }
    
    /// Transform path
    pub fn transform(&mut self, transform: &crate::shapes::primitives::transform::Transform) {
        let mut current_x = self.start_x;
        let mut current_y = self.start_y;
        
        for segment in &mut self.segments {
            match segment.segment_type {
                super::segments::PathSegmentType::MoveTo => {
                    let (tx, ty) = transform.transform_point(segment.x, segment.y);
                    segment.x = tx;
                    segment.y = ty;
                    
                    if self.segments.first() == Some(segment) {
                        self.start_x = tx;
                        self.start_y = ty;
                    }
                    current_x = tx;
                    current_y = ty;
                }
                super::segments::PathSegmentType::LineTo => {
                    let (tx, ty) = transform.transform_point(segment.x, segment.y);
                    segment.x = tx;
                    segment.y = ty;
                    current_x = tx;
                    current_y = ty;
                }
                super::segments::PathSegmentType::QuadraticTo => {
                    let (tx, ty) = transform.transform_point(segment.x, segment.y);
                    let (tcp_x, tcp_y) = transform.transform_point(segment.cp1_x, segment.cp1_y);
                    segment.x = tx;
                    segment.y = ty;
                    segment.cp1_x = tcp_x;
                    segment.cp1_y = tcp_y;
                    current_x = tx;
                    current_y = ty;
                }
                super::segments::PathSegmentType::CubicTo => {
                    let (tx, ty) = transform.transform_point(segment.x, segment.y);
                    let (tcp1_x, tcp1_y) = transform.transform_point(segment.cp1_x, segment.cp1_y);
                    let (tcp2_x, tcp2_y) = transform.transform_point(segment.cp2_x, segment.cp2_y);
                    segment.x = tx;
                    segment.y = ty;
                    segment.cp1_x = tcp1_x;
                    segment.cp1_y = tcp1_y;
                    segment.cp2_x = tcp2_x;
                    segment.cp2_y = tcp2_y;
                    current_x = tx;
                    current_y = ty;
                }
                super::segments::PathSegmentType::Close => {
                    current_x = self.start_x;
                    current_y = self.start_y;
                }
            }
        }
        
        self.current_x = current_x;
        self.current_y = current_y;
    }
    
    /// Get transformed copy of path
    pub fn transformed(&self, transform: &crate::shapes::primitives::transform::Transform) -> Path {
        let mut copy = self.clone();
        copy.transform(transform);
        copy
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Path({} segments, closed: {})", self.segments.len(), self.closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::segments::PathSegment;
    
    #[test]
    fn test_path_creation() {
        let path = Path::new();
        assert!(path.is_empty());
        assert_eq!(path.segment_count(), 0);
        assert!(!path.closed);
    }
    
    #[test]
    fn test_path_from_segments() {
        let segments = vec![
            PathSegment::move_to(0.0, 0.0),
            PathSegment::line_to(10.0, 0.0),
            PathSegment::line_to(10.0, 10.0),
            PathSegment::line_to(0.0, 10.0),
            PathSegment::close(),
        ];
        
        let path = Path::from_segments(segments);
        assert_eq!(path.segment_count(), 5);
        assert!(path.closed);
        assert_eq!(path.area(), 100.0);
    }
    
    #[test]
    fn test_path_area() {
        let mut path = Path::new();
        path.add_segment(PathSegment::move_to(0.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 10.0));
        path.add_segment(PathSegment::line_to(0.0, 10.0));
        path.add_segment(PathSegment::close());
        
        assert_eq!(path.area(), 100.0);
    }
    
    #[test]
    fn test_path_contains_point() {
        let mut path = Path::new();
        path.add_segment(PathSegment::move_to(0.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 10.0));
        path.add_segment(PathSegment::line_to(0.0, 10.0));
        path.add_segment(PathSegment::close());
        
        assert!(path.contains_point(5.0, 5.0));
        assert!(path.contains_point(1.0, 1.0));
        assert!(!path.contains_point(11.0, 5.0));
        assert!(!path.contains_point(5.0, 11.0));
    }
    
    #[test]
    fn test_path_length() {
        let mut path = Path::new();
        path.add_segment(PathSegment::move_to(0.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 10.0));
        path.add_segment(PathSegment::line_to(0.0, 10.0));
        path.add_segment(PathSegment::close());
        
        assert_eq!(path.length(), 40.0); // 10 + 10 + 10 + 10
    }
    
    #[test]
    fn test_path_bounds() {
        let mut path = Path::new();
        path.add_segment(PathSegment::move_to(0.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 10.0));
        path.add_segment(PathSegment::line_to(0.0, 10.0));
        path.add_segment(PathSegment::close());
        
        let bounds = path.bounds();
        assert_eq!(bounds.min_x, 0.0);
        assert_eq!(bounds.min_y, 0.0);
        assert_eq!(bounds.max_x, 10.0);
        assert_eq!(bounds.max_y, 10.0);
        assert_eq!(bounds.center(), (5.0, 5.0));
    }
    
    #[test]
    fn test_path_sampling() {
        let mut path = Path::new();
        path.add_segment(PathSegment::move_to(0.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 10.0));
        path.add_segment(PathSegment::line_to(0.0, 10.0));
        path.add_segment(PathSegment::close());
        
        let points = path.sample_points(5);
        assert_eq!(points.len(), 5);
        assert_eq!(points[0], (0.0, 0.0));
        assert_eq!(points[2], (10.0, 10.0)); // Middle point
    }
    
    #[test]
    fn test_path_simplify() {
        let mut path = Path::new();
        path.add_segment(PathSegment::move_to(0.0, 0.0));
        path.add_segment(PathSegment::line_to(5.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 10.0));
        path.add_segment(PathSegment::line_to(0.0, 10.0));
        path.add_segment(PathSegment::close());
        
        let simplified = path.simplify(1.0);
        assert!(simplified.segment_count() <= path.segment_count());
    }
    
    #[test]
    fn test_path_reverse() {
        let mut path = Path::new();
        path.add_segment(PathSegment::move_to(0.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 10.0));
        path.add_segment(PathSegment::line_to(0.0, 10.0));
        path.add_segment(PathSegment::close());
        
        let reversed = path.reverse();
        assert_eq!(reversed.segment_count(), path.segment_count());
        assert!(reversed.closed);
        
        // Check that start and end are swapped
        assert_eq!(reversed.start_x, path.current_x);
        assert_eq!(reversed.start_y, path.current_y);
    }
    
    #[test]
    fn test_path_transform() {
        let mut path = Path::new();
        path.add_segment(PathSegment::move_to(0.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 10.0));
        path.add_segment(PathSegment::line_to(0.0, 10.0));
        path.add_segment(PathSegment::close());
        
        let transform = crate::shapes::primitives::transform::Transform::translation(5.0, 10.0);
        path.transform(&transform);
        
        let bounds = path.bounds();
        assert_eq!(bounds.min_x, 5.0);
        assert_eq!(bounds.min_y, 10.0);
        assert_eq!(bounds.max_x, 15.0);
        assert_eq!(bounds.max_y, 20.0);
    }
    
    #[test]
    fn test_path_display() {
        let path = Path::new();
        assert_eq!(format!("{}", path), "Path(0 segments, closed: false)");
        
        let mut path = Path::new();
        path.add_segment(PathSegment::move_to(0.0, 0.0));
        path.add_segment(PathSegment::line_to(10.0, 0.0));
        assert_eq!(format!("{}", path), "Path(2 segments, closed: false)");
    }
}
