use serde::{Deserialize, Serialize};
use crate::shapes::primitives::BoundingBox;
use crate::shapes::paths::{Path, PathSegment};
use super::types::*;


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShapeMaskType {
    Rectangle,
    Ellipse,
    Circle,
    Polygon,
    Star,
    Heart,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeMask {
    pub properties: MaskProperties,
    pub shape_type: ShapeMaskType,
    pub size: (f64, f64),
    pub corner_radius: f64,
    pub points: Vec<Point>,
    pub closed: bool,
    pub path: Option<Path>,
    pub feather_edges: bool,
    pub anti_aliasing: bool,
}

impl ShapeMask {
    pub fn new(id: String, name: String, shape_type: ShapeMaskType) -> Self {
        let properties = MaskProperties::new(id.clone(), name, MaskType::Shape);
        Self {
            properties,
            shape_type,
            size: (100.0, 100.0),
            corner_radius: 0.0,
            points: Vec::new(),
            closed: true,
            path: None,
            feather_edges: true,
            anti_aliasing: true,
        }
    }

    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.size = (width, height);
        self
    }

    pub fn with_corner_radius(mut self, radius: f64) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn with_points(mut self, points: Vec<Point>) -> Self {
        self.points = points;
        self
    }

    pub fn with_path(mut self, path: Path) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_feather_edges(mut self, feather: bool) -> Self {
        self.feather_edges = feather;
        self
    }

    pub fn with_anti_aliasing(mut self, aa: bool) -> Self {
        self.anti_aliasing = aa;
        self
    }

    pub fn set_size(&mut self, width: f64, height: f64) {
        self.size = (width, height);
        self.properties.update_modified_time();
    }

    pub fn set_corner_radius(&mut self, radius: f64) {
        self.corner_radius = radius;
        self.properties.update_modified_time();
    }

    pub fn add_point(&mut self, point: Point) {
        self.points.push(point);
        self.properties.update_modified_time();
    }

    pub fn clear_points(&mut self) {
        self.points.clear();
        self.properties.update_modified_time();
    }

    pub fn get_bounds(&self) -> Option<BoundingBox> {
        let (x, y) = self.properties.position;
        let (width, height) = self.size;

        Some(BoundingBox::new(
            x - width / 2.0,
            y - height / 2.0,
            x + width / 2.0,
            y + height / 2.0,
        ))
    }

    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        if !self.properties.enabled {
            return false;
        }

        let (x, y) = self.properties.position;
        let (width, height) = self.size;


        let cos_r = self.properties.rotation.to_radians().cos();
        let sin_r = self.properties.rotation.to_radians().sin();
        let (sx, sy) = self.properties.scale;


        let local_x = (px - x) / sx;
        let local_y = (py - y) / sy;


        let rotated_x = local_x * cos_r + local_y * sin_r;
        let rotated_y = -local_x * sin_r + local_y * cos_r;

        match self.shape_type {
            ShapeMaskType::Rectangle => {
                let half_width = width / 2.0;
                let half_height = height / 2.0;

                if self.corner_radius > 0.0 {

                    let r = self.corner_radius.min(half_width).min(half_height);
                    let center_x = rotated_x.abs();
                    let center_y = rotated_y.abs();

                    if center_x >= half_width - r && center_y >= half_height - r {

                        let corner_center_x = half_width - r;
                        let corner_center_y = half_height - r;
                        let dx = center_x - corner_center_x;
                        let dy = center_y - corner_center_y;
                        dx * dx + dy * dy <= r * r
                    } else {
                        center_x <= half_width && center_y <= half_height
                    }
                } else {

                    rotated_x.abs() <= half_width && rotated_y.abs() <= half_height
                }
            }
            ShapeMaskType::Ellipse => {
                let rx = width / 2.0;
                let ry = height / 2.0;
                (rotated_x * rotated_x) / (rx * rx) + (rotated_y * rotated_y) / (ry * ry) <= 1.0
            }
            ShapeMaskType::Circle => {
                let radius = width.min(height) / 2.0;
                rotated_x * rotated_x + rotated_y * rotated_y <= radius * radius
            }
            ShapeMaskType::Polygon => {
                if self.points.len() < 3 {
                    return false;
                }
                point_in_polygon(rotated_x, rotated_y, &self.points)
            }
            ShapeMaskType::Star => {
                contains_star(rotated_x, rotated_y, width, height, 5)
            }
            ShapeMaskType::Heart => {
                contains_heart(rotated_x, rotated_y, width, height)
            }
            ShapeMaskType::Custom => {
                if let Some(ref path) = self.path {
                    path.contains_point(rotated_x, rotated_y)
                } else if !self.points.is_empty() {
                    point_in_polygon(rotated_x, rotated_y, &self.points)
                } else {
                    false
                }
            }
        }
    }

    pub fn evaluate_at(&self, x: f64, y: f64) -> MaskEvaluation {
        let mut value = 0.0;
        let mut edge_distance = None;

        if self.properties.enabled {
            if self.contains_point(x, y) {
                value = 1.0;


                if self.properties.feather > 0.0 && self.feather_edges {
                    edge_distance = Some(self.calculate_edge_distance(x, y));
                }
            }
        }

        let mut eval = MaskEvaluation::new(self.properties.id.clone(), value, (x, y));

        if let Some(dist) = edge_distance {
            eval = eval.with_edge_distance(dist);
        }

        eval.apply_opacity(self.properties.opacity)
            .apply_invert(self.properties.invert_mode)
    }

    fn calculate_edge_distance(&self, x: f64, y: f64) -> f64 {
        let (px, py) = self.properties.position;
        let (width, height) = self.size;
        let feather = self.properties.feather;

        match self.shape_type {
            ShapeMaskType::Rectangle => {
                let half_width = width / 2.0;
                let half_height = height / 2.0;

                let dx = (x - px).abs() - half_width;
                let dy = (y - py).abs() - half_height;

                if dx <= 0.0 && dy <= 0.0 {

                    dx.min(dy).max(0.0)
                } else if dx > 0.0 && dy > 0.0 {

                    (dx * dx + dy * dy).sqrt()
                } else {

                    dx.max(dy)
                }
            }
            ShapeMaskType::Ellipse => {
                let rx = width / 2.0;
                let ry = height / 2.0;
                let dx = (x - px) / rx;
                let dy = (y - py) / ry;
                let dist = (dx * dx + dy * dy).sqrt();
                (dist - 1.0) * rx.min(ry)
            }
            ShapeMaskType::Circle => {
                let radius = width.min(height) / 2.0;
                let dist = ((x - px) * (x - px) + (y - py) * (y - py)).sqrt();
                dist - radius
            }
            _ => {

                let bounds = self.get_bounds().unwrap();
                let center_x = (bounds.min_x + bounds.max_x) / 2.0;
                let center_y = (bounds.min_y + bounds.max_y) / 2.0;
                let dx = (x - center_x).abs() - (bounds.max_x - bounds.min_x) / 2.0;
                let dy = (y - center_y).abs() - (bounds.max_y - bounds.min_y) / 2.0;
                dx.max(dy)
            }
        }
    }

    pub fn create_rectangle_mask(id: String, name: String, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::new(id, name, ShapeMaskType::Rectangle)
            .with_size(width, height)
            .with_position(x, y)
    }

    pub fn create_ellipse_mask(id: String, name: String, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::new(id, name, ShapeMaskType::Ellipse)
            .with_size(width, height)
            .with_position(x, y)
    }

    pub fn create_circle_mask(id: String, name: String, x: f64, y: f64, radius: f64) -> Self {
        Self::new(id, name, ShapeMaskType::Circle)
            .with_size(radius * 2.0, radius * 2.0)
            .with_position(x, y)
    }

    pub fn create_polygon_mask(id: String, name: String, points: Vec<Point>) -> Self {
        let bounds = calculate_polygon_bounds(&points);
        let center_x = (bounds.min_x + bounds.max_x) / 2.0;
        let center_y = (bounds.min_y + bounds.max_y) / 2.0;
        let width = bounds.max_x - bounds.min_x;
        let height = bounds.max_y - bounds.min_y;

        Self::new(id, name, ShapeMaskType::Polygon)
            .with_points(points)
            .with_size(width, height)
            .with_position(center_x, center_y)
    }

    pub fn create_star_mask(id: String, name: String, x: f64, y: f64, outer_radius: f64, inner_radius: f64, points: usize) -> Self {
        let star_points = generate_star_points(outer_radius, inner_radius, points);
        Self::new(id, name, ShapeMaskType::Star)
            .with_points(star_points)
            .with_size(outer_radius * 2.0, outer_radius * 2.0)
            .with_position(x, y)
    }

    fn with_position(self, x: f64, y: f64) -> Self {
        let mut mask = self;
        mask.properties.set_position(x, y);
        mask
    }

    pub fn validate(&self) -> Result<(), String> {
        self.properties.validate()?;

        if self.size.0 <= 0.0 || self.size.1 <= 0.0 {
            return Err("Mask dimensions must be positive".to_string());
        }

        if self.shape_type == ShapeMaskType::Polygon && self.points.len() < 3 {
            return Err("Polygon mask must have at least 3 points".to_string());
        }

        if self.shape_type == ShapeMaskType::Custom && self.points.is_empty() && self.path.is_none() {
            return Err("Custom mask must have points or a path".to_string());
        }

        Ok(())
    }
}


fn point_in_polygon(x: f64, y: f64, points: &[Point]) -> bool {
    if points.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = points.len() - 1;

    for i in 0..points.len() {
        let xi = points[i].x;
        let yi = points[i].y;
        let xj = points[j].x;
        let yj = points[j].y;

        if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }

    inside
}


fn contains_star(x: f64, y: f64, width: f64, height: f64, points: usize) -> bool {
    let outer_radius = width.min(height) / 2.0;
    let inner_radius = outer_radius * 0.4;
    let star_points = generate_star_points(outer_radius, inner_radius, points);
    point_in_polygon(x, y, &star_points)
}


fn contains_heart(x: f64, y: f64, width: f64, height: f64) -> bool {

    let scale = width.min(height) / 2.0;
    let nx = x / scale;
    let ny = y / scale;


    let equation = (nx * nx + ny * ny - 1.0).powi(3) - nx * nx * ny.powi(3);
    equation <= 0.0 && ny > -0.5
}


fn generate_star_points(outer_radius: f64, inner_radius: f64, points: usize) -> Vec<Point> {
    let mut star_points = Vec::new();
    let angle_step = std::f64::consts::PI / points as f64;

    for i in 0..points * 2 {
        let angle = i as f64 * angle_step - std::f64::consts::PI / 2.0;
        let radius = if i % 2 == 0 { outer_radius } else { inner_radius };

        let x = radius * angle.cos();
        let y = radius * angle.sin();
        star_points.push(Point::new(x, y));
    }

    star_points
}


fn calculate_polygon_bounds(points: &[Point]) -> BoundingBox {
    if points.is_empty() {
        return BoundingBox::new(0.0, 0.0, 0.0, 0.0);
    }

    let mut min_x = points[0].x;
    let mut min_y = points[0].y;
    let mut max_x = points[0].x;
    let mut max_y = points[0].y;

    for point in points.iter().skip(1) {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    BoundingBox::new(min_x, min_y, max_x, max_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_mask_creation() {
        let mask = ShapeMask::new("mask1".to_string(), "Test Mask".to_string(), ShapeMaskType::Rectangle);

        assert_eq!(mask.properties.id, "mask1");
        assert_eq!(mask.properties.name, "Test Mask");
        assert_eq!(mask.shape_type, ShapeMaskType::Rectangle);
        assert_eq!(mask.size, (100.0, 100.0));
        assert!(mask.feather_edges);
        assert!(mask.anti_aliasing);
    }

    #[test]
    fn test_shape_mask_builder() {
        let mask = ShapeMask::new("mask1".to_string(), "Test".to_string(), ShapeMaskType::Ellipse)
            .with_size(200.0, 150.0)
            .with_corner_radius(10.0)
            .with_feather_edges(false)
            .with_anti_aliasing(false);

        assert_eq!(mask.size, (200.0, 150.0));
        assert_eq!(mask.corner_radius, 10.0);
        assert!(!mask.feather_edges);
        assert!(!mask.anti_aliasing);
    }

    #[test]
    fn test_rectangle_mask_contains_point() {
        let mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);

        assert!(mask.contains_point(0.0, 0.0));
        assert!(mask.contains_point(25.0, 25.0));
        assert!(mask.contains_point(-49.0, 0.0));
        assert!(mask.contains_point(49.0, 24.0));

        assert!(!mask.contains_point(51.0, 0.0));
        assert!(!mask.contains_point(0.0, 26.0));
        assert!(!mask.contains_point(100.0, 100.0));
    }

    #[test]
    fn test_circle_mask_contains_point() {
        let mask = ShapeMask::create_circle_mask("circle".to_string(), "Circle".to_string(), 0.0, 0.0, 50.0);

        assert!(mask.contains_point(0.0, 0.0));
        assert!(mask.contains_point(25.0, 25.0));
        assert!(mask.contains_point(-49.0, 0.0));

        assert!(!mask.contains_point(51.0, 0.0));
        assert!(!mask.contains_point(0.0, 51.0));
        assert!(!mask.contains_point(35.0, 35.0));
    }

    #[test]
    fn test_ellipse_mask_contains_point() {
        let mask = ShapeMask::create_ellipse_mask("ellipse".to_string(), "Ellipse".to_string(), 0.0, 0.0, 100.0, 50.0);

        assert!(mask.contains_point(0.0, 0.0));
        assert!(mask.contains_point(49.0, 0.0));
        assert!(mask.contains_point(0.0, 24.0));

        assert!(!mask.contains_point(51.0, 0.0));
        assert!(!mask.contains_point(0.0, 26.0));
        assert!(mask.contains_point(30.0, 20.0));
    }

    #[test]
    fn test_polygon_mask_contains_point() {
        let points = vec![
            Point::new(0.0, -50.0),
            Point::new(50.0, 0.0),
            Point::new(0.0, 50.0),
            Point::new(-50.0, 0.0),
        ];

        let mask = ShapeMask::create_polygon_mask("polygon".to_string(), "Polygon".to_string(), points);

        assert!(mask.contains_point(0.0, 0.0));
        assert!(mask.contains_point(25.0, 0.0));
        assert!(mask.contains_point(0.0, 25.0));

        assert!(!mask.contains_point(51.0, 0.0));
        assert!(!mask.contains_point(0.0, 51.0));
    }

    #[test]
    fn test_star_mask_contains_point() {
        let mask = ShapeMask::create_star_mask("star".to_string(), "Star".to_string(), 0.0, 0.0, 50.0, 20.0, 5);

        assert!(mask.contains_point(0.0, 0.0));
        assert!(mask.contains_point(25.0, 0.0));

        assert!(!mask.contains_point(51.0, 0.0));
        assert!(!mask.contains_point(0.0, 51.0));
    }

    #[test]
    fn test_mask_evaluation() {
        let mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);

        let eval_inside = mask.evaluate_at(0.0, 0.0);
        assert_eq!(eval_inside.value, 1.0);
        assert!(eval_inside.is_inside);

        let eval_outside = mask.evaluate_at(100.0, 100.0);
        assert_eq!(eval_outside.value, 0.0);
        assert!(!eval_outside.is_inside);


        mask.properties.set_opacity(0.5);
        let eval_half = mask.evaluate_at(0.0, 0.0);
        assert_eq!(eval_half.value, 0.5);
    }

    #[test]
    fn test_mask_validation() {
        let valid_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);
        assert!(valid_mask.validate().is_ok());

        let mut invalid_mask = valid_mask.clone();
        invalid_mask.properties.id = "".to_string();
        assert!(invalid_mask.validate().is_err());

        invalid_mask.properties.id = "test".to_string();
        invalid_mask.size = (0.0, 50.0);
        assert!(invalid_mask.validate().is_err());

        invalid_mask.size = (100.0, 50.0);
        invalid_mask.shape_type = ShapeMaskType::Polygon;
        invalid_mask.points = vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
        assert!(invalid_mask.validate().is_err());
    }

    #[test]
    fn test_rounded_rectangle() {
        let mask = ShapeMask::create_rectangle_mask("rounded".to_string(), "Rounded".to_string(), 0.0, 0.0, 100.0, 50.0);
        mask.set_corner_radius(10.0);

        assert!(mask.contains_point(40.0, 20.0));
        assert!(mask.contains_point(45.0, 20.0));
        assert!(!mask.contains_point(49.0, 24.0));
    }

    #[test]
    fn test_mask_setters() {
        let mut mask = ShapeMask::default();

        mask.set_size(200.0, 150.0);
        assert_eq!(mask.size, (200.0, 150.0));

        mask.set_corner_radius(15.0);
        assert_eq!(mask.corner_radius, 15.0);

        mask.add_point(Point::new(10.0, 20.0));
        assert_eq!(mask.points.len(), 1);
        assert_eq!(mask.points[0], Point::new(10.0, 20.0));

        mask.clear_points();
        assert!(mask.points.is_empty());
    }

    #[test]
    fn test_edge_distance_calculation() {
        let mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);

        let distance = mask.calculate_edge_distance(60.0, 0.0);
        assert!((distance - 10.0).abs() < 0.001);

        let distance = mask.calculate_edge_distance(0.0, 0.0);
        assert!(distance <= 0.0);
    }

    impl Default for ShapeMask {
        fn default() -> Self {
            Self::new("default".to_string(), "Default Mask".to_string(), ShapeMaskType::Rectangle)
        }
    }
}
