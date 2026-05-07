//! Line shape primitive
//! 
//! This module provides the Line shape implementation with support
//! for variable thickness and full transformation capabilities.

use serde::{Deserialize, Serialize};

/// Line shape primitive
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub thickness: f64,
}

impl Line {
    /// Create new line
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            thickness: 1.0,
        }
    }
    
    /// Create line with thickness
    pub fn with_thickness(x1: f64, y1: f64, x2: f64, y2: f64, thickness: f64) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            thickness: thickness.max(0.0),
        }
    }
    
    /// Get length of line
    pub fn length(&self) -> f64 {
        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// Get angle of line in radians
    pub fn angle(&self) -> f64 {
        (self.y2 - self.y1).atan2(self.x2 - self.x1)
    }
    
    /// Check if line is a point (start equals end)
    pub fn is_point(&self) -> bool {
        (self.x1 - self.x2).abs() < f64::EPSILON && (self.y1 - self.y2).abs() < f64::EPSILON
    }
    
    /// Get midpoint of line
    pub fn midpoint(&self) -> (f64, f64) {
        ((self.x1 + self.x2) / 2.0, (self.y1 + self.y2) / 2.0)
    }
}

impl super::types::ShapePrimitive for Line {
    fn shape_type(&self) -> super::types::ShapeType {
        super::types::ShapeType::Line
    }
    
    fn bounds(&self) -> super::transform::BoundingBox {
        let half_thickness = self.thickness / 2.0;
        super::transform::BoundingBox::new(
            self.x1.min(self.x2) - half_thickness,
            self.y1.min(self.y2) - half_thickness,
            self.x1.max(self.x2) + half_thickness,
            self.y1.max(self.y2) + half_thickness,
        )
    }
    
    fn to_path(&self) -> super::super::paths::Path {
        let mut builder = super::super::paths::PathBuilder::new();
        
        if self.thickness > 0.0 {
            // Create thick line as rectangle
            let dx = self.x2 - self.x1;
            let dy = self.y2 - self.y1;
            let length = (dx * dx + dy * dy).sqrt();
            
            if length > 0.0 {
                let nx = -dy / length; // Normal X
                let ny = dx / length;  // Normal Y
                
                let half_thickness = self.thickness / 2.0;
                
                // Calculate rectangle corners
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
            // Thin line
            builder.move_to(self.x1, self.y1);
            builder.line_to(self.x2, self.y2);
        }
        
        builder.build()
    }
    
    fn contains_point(&self, x: f64, y: f64) -> bool {
        if self.thickness == 0.0 {
            return false; // Zero thickness line has no area
        }
        
        // Check if point is within distance threshold of line segment
        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        let length_sq = dx * dx + dy * dy;
        
        if length_sq == 0.0 {
            // Point is line, check distance to point
            let dist_sq = (x - self.x1) * (x - self.x1) + (y - self.y1) * (y - self.y1);
            return dist_sq <= (self.thickness / 2.0).powi(2);
        }
        
        // Calculate projection parameter
        let t = ((x - self.x1) * dx + (y - self.y1) * dy) / length_sq;
        let t_clamped = t.clamp(0.0, 1.0);
        
        // Find closest point on line segment
        let closest_x = self.x1 + t_clamped * dx;
        let closest_y = self.y1 + t_clamped * dy;
        
        // Check distance to closest point
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
    
    fn transform(&mut self, transform: &super::transform::Transform) {
        let (x1, y1) = transform.transform_point(self.x1, self.y1);
        let (x2, y2) = transform.transform_point(self.x2, self.y2);
        
        self.x1 = x1;
        self.y1 = y1;
        self.x2 = x2;
        self.y2 = y2;
        
        // Scale thickness by average scale factor
        self.thickness *= (transform.sx * transform.sy).sqrt() / 2.0;
    }
    
    fn transformed(&self, transform: &super::transform::Transform) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ShapePrimitive;
    
    #[test]
    fn test_line_creation() {
        let line = Line::new(0.0, 0.0, 100.0, 100.0);
        
        assert_eq!(line.x1, 0.0);
        assert_eq!(line.y1, 0.0);
        assert_eq!(line.x2, 100.0);
        assert_eq!(line.y2, 100.0);
        assert_eq!(line.thickness, 1.0);
        assert!(!line.is_point());
    }
    
    #[test]
    fn test_line_with_thickness() {
        let line = Line::with_thickness(0.0, 0.0, 100.0, 100.0, 5.0);
        
        assert_eq!(line.thickness, 5.0);
        assert!(line.validate().is_ok());
    }
    
    #[test]
    fn test_line_length_angle() {
        let line = Line::new(0.0, 0.0, 100.0, 0.0);
        
        assert_eq!(line.length(), 100.0);
        assert_eq!(line.angle(), 0.0);
        
        let diagonal_line = Line::new(0.0, 0.0, 100.0, 100.0);
        assert!((diagonal_line.length() - 141.421).abs() < 0.001);
        assert!((diagonal_line.angle() - std::f64::consts::PI / 4.0).abs() < 0.001);
    }
    
    #[test]
    fn test_line_midpoint() {
        let line = Line::new(0.0, 0.0, 100.0, 100.0);
        let midpoint = line.midpoint();
        
        assert_eq!(midpoint, (50.0, 50.0));
    }
    
    #[test]
    fn test_line_contains_point() {
        let line = Line::with_thickness(0.0, 0.0, 100.0, 0.0, 10.0);
        
        assert!(line.contains_point(50.0, 0.0)); // On line
        assert!(line.contains_point(50.0, 4.0)); // Within thickness
        assert!(!line.contains_point(50.0, 6.0)); // Outside thickness
        assert!(!line.contains_point(150.0, 0.0)); // Beyond end
    }
    
    #[test]
    fn test_line_area_perimeter() {
        let line = Line::with_thickness(0.0, 0.0, 100.0, 0.0, 5.0);
        
        assert_eq!(line.area(), 500.0); // 100 * 5
        assert_eq!(line.perimeter(), 210.0); // 2*100 + 2*5
        
        let thin_line = Line::new(0.0, 0.0, 100.0, 0.0);
        assert_eq!(thin_line.area(), 0.0);
        assert_eq!(thin_line.perimeter(), 100.0);
    }
    
    #[test]
    fn test_line_transform() {
        let mut line = Line::new(0.0, 0.0, 100.0, 0.0);
        let transform = super::transform::Transform::translation(5.0, 10.0);
        
        line.transform(&transform);
        
        assert_eq!(line.x1, 5.0);
        assert_eq!(line.y1, 10.0);
        assert_eq!(line.x2, 105.0);
        assert_eq!(line.y2, 10.0);
    }
    
    #[test]
    fn test_line_validation() {
        let valid_line = Line::with_thickness(0.0, 0.0, 100.0, 0.0, 5.0);
        assert!(valid_line.validate().is_ok());
        
        let invalid_line = Line::with_thickness(0.0, 0.0, 100.0, 0.0, -5.0);
        assert!(invalid_line.validate().is_err());
    }
    
    #[test]
    fn test_line_bounds() {
        let line = Line::with_thickness(10.0, 20.0, 100.0, 80.0, 6.0);
        let bounds = line.bounds();
        
        assert_eq!(bounds.min_x, 7.0); // 10 - 3
        assert_eq!(bounds.min_y, 17.0); // 20 - 3
        assert_eq!(bounds.max_x, 103.0); // 100 + 3
        assert_eq!(bounds.max_y, 83.0); // 80 + 3
        assert_eq!(bounds.width(), 96.0);
        assert_eq!(bounds.height(), 66.0);
    }
    
    #[test]
    fn test_line_to_path() {
        let line = Line::with_thickness(0.0, 0.0, 100.0, 0.0, 10.0);
        let path = line.to_path();
        
        assert!(path.closed);
        assert!((path.area() - 1000.0).abs() < 0.001); // 100 * 10
    }
    
    #[test]
    fn test_point_line() {
        let point_line = Line::new(50.0, 50.0, 50.0, 50.0);
        assert!(point_line.is_point());
        
        // Point line should still work with thickness
        let thick_point = Line::with_thickness(50.0, 50.0, 50.0, 50.0, 10.0);
        assert!(thick_point.contains_point(55.0, 50.0));
        assert!(!thick_point.contains_point(60.0, 50.0));
    }
}
