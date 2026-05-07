//! Path builder for fluent path construction
//! 
//! This module provides a fluent interface for building paths
//! with various shape primitives and bezier curves.

/// Path builder for fluent path construction
pub struct PathBuilder {
    path: super::path::Path,
}

impl PathBuilder {
    /// Create new path builder
    pub fn new() -> Self {
        Self {
            path: super::path::Path::new(),
        }
    }
    
    /// Move to point
    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        let segment = super::segments::PathSegment::move_to(x, y);
        
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
        let segment = super::segments::PathSegment::line_to(x, y);
        self.path.segments.push(segment);
        self.path.current_x = x;
        self.path.current_y = y;
        self
    }
    
    /// Quadratic bezier to point
    pub fn quadratic_to(&mut self, x: f64, y: f64, cp_x: f64, cp_y: f64) -> &mut Self {
        let segment = super::segments::PathSegment::quadratic_to(x, y, cp_x, cp_y);
        self.path.segments.push(segment);
        self.path.current_x = x;
        self.path.current_y = y;
        self
    }
    
    /// Cubic bezier to point
    pub fn bezier_to(&mut self, x: f64, y: f64, cp1_x: f64, cp1_y: f64, cp2_x: f64, cp2_y: f64) -> &mut Self {
        let segment = super::segments::PathSegment::cubic_to(x, y, cp1_x, cp1_y, cp2_x, cp2_y);
        self.path.segments.push(segment);
        self.path.current_x = x;
        self.path.current_y = y;
        self
    }
    
    /// Close path
    pub fn close(&mut self) -> &mut Self {
        let segment = super::segments::PathSegment::close();
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
    
    /// Add rounded rectangle
    pub fn rounded_rectangle(&mut self, x: f64, y: f64, width: f64, height: f64, radius: f64) -> &mut Self {
        let r = radius.min(width / 2.0).min(height / 2.0);
        
        self.move_to(x + r, y);
        self.line_to(x + width - r, y);
        self.quadratic_to(x + width, y, x + width, y + r);
        self.line_to(x + width, y + height - r);
        self.quadratic_to(x + width, y + height, x + width - r, y + height);
        self.line_to(x + r, y + height);
        self.quadratic_to(x, y + height, x, y + height - r);
        self.line_to(x, y + r);
        self.quadratic_to(x, y, x + r, y);
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
    
    /// Add star shape
    pub fn star(&mut self, cx: f64, cy: f64, outer_radius: f64, inner_radius: f64, points: usize) -> &mut Self {
        let angle_step = std::f64::consts::PI / points as f64;
        
        for i in 0..points * 2 {
            let angle = i as f64 * angle_step - std::f64::consts::PI / 2.0;
            let radius = if i % 2 == 0 { outer_radius } else { inner_radius };
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
    
    /// Add arc
    pub fn arc(&mut self, cx: f64, cy: f64, radius: f64, start_angle: f64, end_angle: f64) -> &mut Self {
        let start_x = cx + radius * start_angle.cos();
        let start_y = cy + radius * start_angle.sin();
        
        if self.path.segments.is_empty() {
            self.move_to(start_x, start_y);
        } else {
            self.line_to(start_x, start_y);
        }
        
        // Approximate arc with bezier curves
        let angle_diff = end_angle - start_angle;
        let steps = ((angle_diff.abs() / (std::f64::consts::PI / 4.0)).ceil() as usize).max(1);
        let angle_step = angle_diff / steps as f64;
        
        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            let angle = start_angle + t * angle_diff;
            let x = cx + radius * angle.cos();
            let y = cy + radius * angle.sin();
            
            if i == 1 {
                self.line_to(x, y);
            } else {
                // Use quadratic bezier for smoother arc
                let prev_angle = start_angle + (i - 1) as f64 * angle_step;
                let mid_angle = prev_angle + angle_step / 2.0;
                let mid_x = cx + radius * mid_angle.cos();
                let mid_y = cy + radius * mid_angle.sin();
                
                self.quadratic_to(x, y, mid_x, mid_y);
            }
        }
        
        self
    }
    
    /// Add polyline (open path)
    pub fn polyline(&mut self, points: &[(f64, f64)]) -> &mut Self {
        if let Some((x, y)) = points.first() {
            self.move_to(*x, *y);
            
            for (x, y) in points.iter().skip(1) {
                self.line_to(*x, *y);
            }
        }
        
        self
    }
    
    /// Add smooth curve through points
    pub fn smooth_curve(&mut self, points: &[(f64, f64)], tension: f64) -> &mut Self {
        if points.len() < 2 {
            return self;
        }
        
        if let Some((x, y)) = points.first() {
            self.move_to(*x, *y);
        }
        
        for i in 1..points.len() {
            if i == 1 {
                // First segment - line to second point
                if let Some((x, y)) = points.get(i) {
                    self.line_to(*x, *y);
                }
            } else if i == points.len() - 1 {
                // Last segment - line from previous point
                if let Some((x, y)) = points.get(i) {
                    self.line_to(*x, *y);
                }
            } else {
                // Middle segments - use bezier curves
                let (x0, y0) = points[i - 1];
                let (x1, y1) = points[i];
                let (x2, y2) = points[i + 1];
                
                let cp1_x = x0 + (x1 - x0) * (1.0 - tension);
                let cp1_y = y0 + (y1 - y0) * (1.0 - tension);
                let cp2_x = x1 - (x2 - x0) * (1.0 - tension) / 3.0;
                let cp2_y = y1 - (y2 - y0) * (1.0 - tension) / 3.0;
                
                self.bezier_to(x1, y1, cp1_x, cp1_y, cp2_x, cp2_y);
            }
        }
        
        self
    }
    
    /// Build the path
    pub fn build(self) -> super::path::Path {
        self.path
    }
    
    /// Get current position
    pub fn current_position(&self) -> (f64, f64) {
        (self.path.current_x, self.path.current_y)
    }
    
    /// Get starting position
    pub fn start_position(&self) -> (f64, f64) {
        (self.path.start_x, self.path.start_y)
    }
    
    /// Check if path is closed
    pub fn is_closed(&self) -> bool {
        self.path.closed
    }
    
    /// Get segment count
    pub fn segment_count(&self) -> usize {
        self.path.segments.len()
    }
}

impl Default for PathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_builder() {
        let mut builder = PathBuilder::new();
        builder.move_to(0.0, 0.0);
        builder.line_to(10.0, 0.0);
        builder.line_to(10.0, 10.0);
        builder.line_to(0.0, 10.0);
        builder.close();
        
        let path = builder.build();
        assert_eq!(path.segments.len(), 5);
        assert!(path.closed);
    }
    
    #[test]
    fn test_rectangle_builder() {
        let mut builder = PathBuilder::new();
        builder.rectangle(0.0, 0.0, 10.0, 5.0);
        
        let path = builder.build();
        assert!(path.closed);
        assert_eq!(path.area(), 50.0);
    }
    
    #[test]
    fn test_rounded_rectangle_builder() {
        let mut builder = PathBuilder::new();
        builder.rounded_rectangle(0.0, 0.0, 10.0, 10.0, 2.0);
        
        let path = builder.build();
        assert!(path.closed);
        assert!(path.area() < 100.0); // Rounded corners reduce area
    }
    
    #[test]
    fn test_circle_builder() {
        let mut builder = PathBuilder::new();
        builder.circle(50.0, 50.0, 25.0);
        
        let path = builder.build();
        assert!(path.closed);
        let expected_area = std::f64::consts::PI * 25.0 * 25.0;
        assert!((path.area() - expected_area).abs() < 100.0); // Allow approximation error
    }
    
    #[test]
    fn test_ellipse_builder() {
        let mut builder = PathBuilder::new();
        builder.ellipse(50.0, 50.0, 40.0, 20.0);
        
        let path = builder.build();
        assert!(path.closed);
        let expected_area = std::f64::consts::PI * 40.0 * 20.0;
        assert!((path.area() - expected_area).abs() < 100.0); // Allow approximation error
    }
    
    #[test]
    fn test_regular_polygon_builder() {
        let mut builder = PathBuilder::new();
        builder.regular_polygon(50.0, 50.0, 30.0, 6);
        
        let path = builder.build();
        assert!(path.closed);
        assert!(path.contains_point(50.0, 50.0));
    }
    
    #[test]
    fn test_star_builder() {
        let mut builder = PathBuilder::new();
        builder.star(50.0, 50.0, 30.0, 15.0, 5);
        
        let path = builder.build();
        assert!(path.closed);
        assert!(path.contains_point(50.0, 50.0));
    }
    
    #[test]
    fn test_polyline_builder() {
        let points = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (20.0, 10.0)];
        let mut builder = PathBuilder::new();
        builder.polyline(&points);
        
        let path = builder.build();
        assert!(!path.closed);
        assert_eq!(path.segments.len(), 4);
    }
    
    #[test]
    fn test_smooth_curve_builder() {
        let points = vec![(0.0, 0.0), (10.0, 5.0), (20.0, 0.0), (30.0, 5.0)];
        let mut builder = PathBuilder::new();
        builder.smooth_curve(&points, 0.5);
        
        let path = builder.build();
        assert!(!path.closed);
        assert_eq!(path.segments.len(), 4);
    }
    
    #[test]
    fn test_arc_builder() {
        let mut builder = PathBuilder::new();
        builder.arc(50.0, 50.0, 25.0, 0.0, std::f64::consts::PI);
        
        let path = builder.build();
        assert!(!path.closed);
        assert!(path.segments.len() > 1);
    }
    
    #[test]
    fn test_builder_position_tracking() {
        let mut builder = PathBuilder::new();
        assert_eq!(builder.current_position(), (0.0, 0.0));
        
        builder.move_to(10.0, 20.0);
        assert_eq!(builder.current_position(), (10.0, 20.0));
        assert_eq!(builder.start_position(), (10.0, 20.0));
        
        builder.line_to(30.0, 40.0);
        assert_eq!(builder.current_position(), (30.0, 40.0));
        assert_eq!(builder.start_position(), (10.0, 20.0));
    }
    
    #[test]
    fn test_builder_state() {
        let mut builder = PathBuilder::new();
        assert!(!builder.is_closed());
        assert_eq!(builder.segment_count(), 0);
        
        builder.move_to(0.0, 0.0);
        assert_eq!(builder.segment_count(), 1);
        assert!(!builder.is_closed());
        
        builder.close();
        assert!(builder.is_closed());
        assert_eq!(builder.segment_count(), 2);
    }
}
