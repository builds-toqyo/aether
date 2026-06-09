use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextPath {
    pub id: String,
    pub path_type: PathType,
    pub points: Vec<(f64, f64)>,
    pub closed: bool,
    pub start_offset: f64,
    pub direction: PathDirection,
    pub spacing: PathSpacing,
    pub alignment: PathAlignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathType {
    Line,
    Bezier,
    Circle,
    Ellipse,
    Rectangle,
    Polygon,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathDirection {
    Forward,
    Reverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathSpacing {
    Uniform,
    Proportional,
    Fixed(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathAlignment {
    Center,
    Left,
    Right,
}

impl TextPath {
    pub fn new(id: String, path_type: PathType) -> Self {
        Self {
            id,
            path_type,
            points: Vec::new(),
            closed: false,
            start_offset: 0.0,
            direction: PathDirection::Forward,
            spacing: PathSpacing::Uniform,
            alignment: PathAlignment::Center,
        }
    }

    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push((x, y));
    }

    pub fn add_points(&mut self, points: &[(f64, f64)]) {
        self.points.extend_from_slice(points);
    }

    pub fn clear_points(&mut self) {
        self.points.clear();
    }

    pub fn get_length(&self) -> f64 {
        if self.points.is_empty() {
            return 0.0;
        }

        let mut total_length = 0.0;
        for i in 0..self.points.len() - 1 {
            let (x1, y1) = self.points[i];
            let (x2, y2) = self.points[i + 1];
            total_length += ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
        }

        if self.closed && !self.points.is_empty() {
            let (x1, y1) = self.points[self.points.len() - 1];
            let (x2, y2) = self.points[0];
            total_length += ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
        }

        total_length
    }

    pub fn get_point_at_distance(&self, distance: f64) -> Option<(f64, f64)> {
        if self.points.is_empty() {
            return None;
        }

        if self.points.len() == 1 {
            return Some(self.points[0]);
        }

        let total_length = self.get_length();
        if total_length == 0.0 {
            return Some(self.points[0]);
        }

        let target_distance = (distance + self.start_offset) % total_length;
        let mut accumulated_distance = 0.0;

        for i in 0..self.points.len() - 1 {
            let (x1, y1) = self.points[i];
            let (x2, y2) = self.points[i + 1];
            let segment_length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();

            if accumulated_distance + segment_length >= target_distance {
                let t = (target_distance - accumulated_distance) / segment_length;
                let x = x1 + (x2 - x1) * t;
                let y = y1 + (y2 - y1) * t;
                return Some((x, y));
            }

            accumulated_distance += segment_length;
        }

        if self.closed && !self.points.is_empty() {
            let (x1, y1) = self.points[self.points.len() - 1];
            let (x2, y2) = self.points[0];
            let segment_length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();

            if accumulated_distance + segment_length >= target_distance {
                let t = (target_distance - accumulated_distance) / segment_length;
                let x = x1 + (x2 - x1) * t;
                let y = y1 + (y2 - y1) * t;
                return Some((x, y));
            }
        }

        Some(self.points[0])
    }

    pub fn get_tangent_at_distance(&self, distance: f64) -> Option<(f64, f64)> {
        if self.points.len() < 2 {
            return None;
        }

        let epsilon = 0.01;
        let p1 = self.get_point_at_distance(distance - epsilon)?;
        let p2 = self.get_point_at_distance(distance + epsilon)?;

        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let length = (dx * dx + dy * dy).sqrt();

        if length > 0.0 {
            Some((dx / length, dy / length))
        } else {
            Some((1.0, 0.0))
        }
    }

    pub fn get_normal_at_distance(&self, distance: f64) -> Option<(f64, f64)> {
        let (tx, ty) = self.get_tangent_at_distance(distance)?;
        Some((-ty, tx))
    }

    pub fn get_angle_at_distance(&self, distance: f64) -> Option<f64> {
        let (tx, ty) = self.get_tangent_at_distance(distance)?;
        Some(ty.atan2(tx))
    }

    pub fn get_bounds(&self) -> Option<(f64, f64, f64, f64)> {
        if self.points.is_empty() {
            return None;
        }

        let (mut min_x, mut min_y) = self.points[0];
        let (mut max_x, mut max_y) = self.points[0];

        for (x, y) in &self.points[1..] {
            min_x = min_x.min(*x);
            min_y = min_y.min(*y);
            max_x = max_x.max(*x);
            max_y = max_y.max(*y);
        }

        Some((min_x, min_y, max_x, max_y))
    }

    pub fn reverse(&mut self) {
        self.points.reverse();
        self.direction = match self.direction {
            PathDirection::Forward => PathDirection::Reverse,
            PathDirection::Reverse => PathDirection::Forward,
        };
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Path ID cannot be empty".to_string());
        }

        if self.points.is_empty() {
            return Err("Path must have at least one point".to_string());
        }

        if self.closed && self.points.len() < 2 {
            return Err("Closed path must have at least 2 points".to_string());
        }

        if let PathSpacing::Fixed(spacing) = self.spacing {
            if spacing <= 0.0 {
                return Err("Fixed spacing must be positive".to_string());
            }
        }

        Ok(())
    }
}

impl Default for TextPath {
    fn default() -> Self {
        Self::new("default_path".to_string(), PathType::Line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_path_creation() {
        let path = TextPath::new("test_path".to_string(), PathType::Line);
        assert_eq!(path.id, "test_path");
        assert_eq!(path.path_type, PathType::Line);
        assert!(path.points.is_empty());
        assert!(!path.closed);
        assert_eq!(path.direction, PathDirection::Forward);
    }

    #[test]
    fn test_text_path_add_points() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);
        path.add_point(100.0, 100.0);

        assert_eq!(path.points.len(), 3);
        assert_eq!(path.points[0], (0.0, 0.0));
        assert_eq!(path.points[1], (100.0, 0.0));
        assert_eq!(path.points[2], (100.0, 100.0));
    }

    #[test]
    fn test_text_path_add_multiple_points() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        let points = vec![(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)];
        path.add_points(&points);

        assert_eq!(path.points.len(), 3);
        assert_eq!(path.points, points);
    }

    #[test]
    fn test_text_path_length() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);
        path.add_point(100.0, 100.0);

        let length = path.get_length();
        assert!((length - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_text_path_point_at_distance() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);

        let point = path.get_point_at_distance(50.0);
        assert!(point.is_some());
        assert_eq!(point.unwrap(), (50.0, 0.0));

        let point = path.get_point_at_distance(25.0);
        assert!(point.is_some());
        assert_eq!(point.unwrap(), (25.0, 0.0));
    }

    #[test]
    fn test_text_path_tangent() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);

        let tangent = path.get_tangent_at_distance(50.0);
        assert!(tangent.is_some());
        let (tx, ty) = tangent.unwrap();
        assert!((tx - 1.0).abs() < 0.001);
        assert!((ty - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_text_path_normal() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);

        let normal = path.get_normal_at_distance(50.0);
        assert!(normal.is_some());
        let (nx, ny) = normal.unwrap();
        assert!((nx - 0.0).abs() < 0.001);
        assert!((ny - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_text_path_angle() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);

        let angle = path.get_angle_at_distance(50.0);
        assert!(angle.is_some());
        assert!((angle.unwrap() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_closed_path() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);
        path.add_point(100.0, 100.0);
        path.add_point(0.0, 100.0);
        path.closed = true;

        let length = path.get_length();
        assert!((length - 400.0).abs() < 0.001);


        let point = path.get_point_at_distance(450.0);
        assert!(point.is_some());
    }

    #[test]
    fn test_path_bounds() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(10.0, 20.0);
        path.add_point(100.0, 150.0);
        path.add_point(50.0, 80.0);

        let bounds = path.get_bounds();
        assert!(bounds.is_some());
        let (min_x, min_y, max_x, max_y) = bounds.unwrap();
        assert_eq!(min_x, 10.0);
        assert_eq!(min_y, 20.0);
        assert_eq!(max_x, 100.0);
        assert_eq!(max_y, 150.0);
    }

    #[test]
    fn test_path_reverse() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);
        path.add_point(100.0, 100.0);

        path.reverse();
        assert_eq!(path.points, vec![(100.0, 100.0), (100.0, 0.0), (0.0, 0.0)]);
        assert_eq!(path.direction, PathDirection::Reverse);

        path.reverse();
        assert_eq!(path.points, vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0)]);
        assert_eq!(path.direction, PathDirection::Forward);
    }

    #[test]
    fn test_path_validation() {
        let valid_path = TextPath::new("test".to_string(), PathType::Line);
        assert!(valid_path.validate().is_err());

        let mut path_with_points = TextPath::new("test".to_string(), PathType::Line);
        path_with_points.add_point(0.0, 0.0);
        assert!(path_with_points.validate().is_ok());

        path_with_points.id = "".to_string();
        assert!(path_with_points.validate().is_err());

        path_with_points.id = "test".to_string();
        path_with_points.closed = true;
        assert!(path_with_points.validate().is_err());

        path_with_points.add_point(100.0, 0.0);
        assert!(path_with_points.validate().is_ok());

        path_with_points.spacing = PathSpacing::Fixed(-1.0);
        assert!(path_with_points.validate().is_err());
    }

    #[test]
    fn test_path_types() {
        let path_types = vec![
            PathType::Line,
            PathType::Bezier,
            PathType::Circle,
            PathType::Ellipse,
            PathType::Rectangle,
            PathType::Polygon,
            PathType::Custom,
        ];

        for path_type in path_types {
            let path = TextPath::new("test".to_string(), path_type);
            assert_eq!(path.path_type, path_type);
        }
    }

    #[test]
    fn test_path_spacing_options() {
        let spacing_options = vec![
            PathSpacing::Uniform,
            PathSpacing::Proportional,
            PathSpacing::Fixed(10.0),
        ];

        for spacing in spacing_options {
            let mut path = TextPath::new("test".to_string(), PathType::Line);
            path.spacing = spacing;
            assert_eq!(path.spacing, spacing);
        }
    }

    #[test]
    fn test_path_alignment_options() {
        let alignments = vec![
            PathAlignment::Center,
            PathAlignment::Left,
            PathAlignment::Right,
        ];

        for alignment in alignments {
            let mut path = TextPath::new("test".to_string(), PathType::Line);
            path.alignment = alignment;
            assert_eq!(path.alignment, alignment);
        }
    }

    #[test]
    fn test_clear_points() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);
        assert_eq!(path.points.len(), 2);

        path.clear_points();
        assert!(path.points.is_empty());
    }

    #[test]
    fn test_default_path() {
        let path = TextPath::default();
        assert_eq!(path.id, "default_path");
        assert_eq!(path.path_type, PathType::Line);
        assert!(path.points.is_empty());
        assert!(!path.closed);
    }
}
