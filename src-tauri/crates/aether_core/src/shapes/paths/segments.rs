use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathSegmentType {
    MoveTo,
    LineTo,
    QuadraticTo,
    CubicTo,
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
    pub fn point_at(&self, t: f64, start_x: f64, start_y: f64) -> (f64, f64) {
        match self.segment_type {
            PathSegmentType::MoveTo => (self.x, self.y),
            PathSegmentType::LineTo => {
                (start_x + t * (self.x - start_x), start_y + t * (self.y - start_y))
            }
            PathSegmentType::QuadraticTo => {
                let one_t = 1.0 - t;
                let x = one_t * one_t * start_x + 2.0 * one_t * t * self.cp1_x + t * t * self.x;
                let y = one_t * one_t * start_y + 2.0 * one_t * t * self.cp1_y + t * t * self.y;
                (x, y)
            }
            PathSegmentType::CubicTo => {
                let one_t = 1.0 - t;
                let one_t2 = one_t * one_t;
                let one_t3 = one_t2 * one_t;
                let t2 = t * t;
                let t3 = t2 * t;
                let x = one_t3 * start_x + 3.0 * one_t2 * t * self.cp1_x + 3.0 * one_t * t2 * self.cp2_x + t3 * self.x;
                let y = one_t3 * start_y + 3.0 * one_t2 * t * self.cp1_y + 3.0 * one_t * t2 * self.cp2_y + t3 * self.y;
                (x, y)
            }
            PathSegmentType::Close => (self.x, self.y),
        }
    }
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


    fn evaluate_quadratic_bezier(&self, t: f64, x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> (f64, f64) {
        let mt = 1.0 - t;
        let x = mt * mt * x0 + 2.0 * mt * t * x1 + t * t * x2;
        let y = mt * mt * y0 + 2.0 * mt * t * y1 + t * t * y2;
        (x, y)
    }


    pub fn is_move(&self) -> bool {
        matches!(self.segment_type, PathSegmentType::MoveTo)
    }


    pub fn is_drawing(&self) -> bool {
        matches!(self.segment_type, PathSegmentType::LineTo | PathSegmentType::QuadraticTo | PathSegmentType::CubicTo)
    }


    pub fn is_close(&self) -> bool {
        matches!(self.segment_type, PathSegmentType::Close)
    }


    pub fn end_point(&self) -> (f64, f64) {
        (self.x, self.y)
    }


    pub fn control_points(&self) -> Option<[(f64, f64); 2]> {
        match self.segment_type {
            PathSegmentType::QuadraticTo => Some([(self.cp1_x, self.cp1_y), (self.x, self.y)]),
            PathSegmentType::CubicTo => Some([(self.cp1_x, self.cp1_y), (self.cp2_x, self.cp2_y)]),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_segment_creation() {
        let move_seg = PathSegment::move_to(10.0, 20.0);
        assert_eq!(move_seg.segment_type, PathSegmentType::MoveTo);
        assert_eq!(move_seg.x, 10.0);
        assert_eq!(move_seg.y, 20.0);
        assert!(move_seg.is_move());
        assert!(!move_seg.is_drawing());
        assert!(!move_seg.is_close());

        let line_seg = PathSegment::line_to(30.0, 40.0);
        assert_eq!(line_seg.segment_type, PathSegmentType::LineTo);
        assert!(line_seg.is_drawing());
        assert!(!line_seg.is_move());
        assert!(!line_seg.is_close());

        let close_seg = PathSegment::close();
        assert_eq!(close_seg.segment_type, PathSegmentType::Close);
        assert!(close_seg.is_close());
        assert!(!close_seg.is_move());
        assert!(!close_seg.is_drawing());
    }

    #[test]
    fn test_bezier_segments() {
        let quad_seg = PathSegment::quadratic_to(50.0, 60.0, 30.0, 40.0);
        assert_eq!(quad_seg.segment_type, PathSegmentType::QuadraticTo);
        assert_eq!(quad_seg.cp1_x, 30.0);
        assert_eq!(quad_seg.cp1_y, 40.0);

        let control_points = quad_seg.control_points();
        assert!(control_points.is_some());
        assert_eq!(control_points.unwrap()[0], (30.0, 40.0));
        assert_eq!(control_points.unwrap()[1], (50.0, 60.0));

        let cubic_seg = PathSegment::cubic_to(70.0, 80.0, 30.0, 40.0, 50.0, 60.0);
        assert_eq!(cubic_seg.segment_type, PathSegmentType::CubicTo);
        assert_eq!(cubic_seg.cp1_x, 30.0);
        assert_eq!(cubic_seg.cp1_y, 40.0);
        assert_eq!(cubic_seg.cp2_x, 50.0);
        assert_eq!(cubic_seg.cp2_y, 60.0);
    }

    #[test]
    fn test_line_segment_length() {
        let line_seg = PathSegment::line_to(10.0, 0.0);
        let length = line_seg.length(0.0, 0.0);
        assert_eq!(length, 10.0);

        let diagonal_seg = PathSegment::line_to(10.0, 10.0);
        let diagonal_length = diagonal_seg.length(0.0, 0.0);
        assert!((diagonal_length - 14.142).abs() < 0.001);
    }

    #[test]
    fn test_segment_sampling() {
        let line_seg = PathSegment::line_to(10.0, 0.0);
        let points = line_seg.sample_points(0.0, 0.0, 5);

        assert_eq!(points.len(), 5);
        assert_eq!(points[0], (0.0, 0.0));
        assert_eq!(points[4], (10.0, 0.0));
        assert_eq!(points[2], (5.0, 0.0));
    }

    #[test]
    fn test_bezier_segment_sampling() {
        let quad_seg = PathSegment::quadratic_to(10.0, 0.0, 5.0, -5.0);
        let points = quad_seg.sample_points(0.0, 0.0, 5);

        assert_eq!(points.len(), 5);
        assert_eq!(points[0], (0.0, 0.0));
        assert_eq!(points[4], (10.0, 0.0));
    }

    #[test]
    fn test_segment_end_point() {
        let seg = PathSegment::line_to(30.0, 40.0);
        let end_point = seg.end_point();
        assert_eq!(end_point, (30.0, 40.0));
    }

    #[test]
    fn test_segment_type_display() {
        assert_eq!(format!("{}", PathSegmentType::MoveTo), "MoveTo");
        assert_eq!(format!("{}", PathSegmentType::LineTo), "LineTo");
        assert_eq!(format!("{}", PathSegmentType::QuadraticTo), "QuadraticTo");
        assert_eq!(format!("{}", PathSegmentType::CubicTo), "CubicTo");
        assert_eq!(format!("{}", PathSegmentType::Close), "Close");
    }
}
