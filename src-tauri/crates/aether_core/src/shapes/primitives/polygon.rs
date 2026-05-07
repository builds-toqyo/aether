

use serde::{Deserialize, Serialize};


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


    pub fn regular_rotated(cx: f64, cy: f64, radius: f64, sides: usize, rotation: f64) -> Self {
        let mut vertices = Vec::with_capacity(sides);
        let angle_step = 2.0 * std::f64::consts::PI / sides as f64;

        for i in 0..sides {
            let angle = i as f64 * angle_step + rotation;
            let x = cx + radius * angle.cos();
            let y = cy + radius * angle.sin();
            vertices.push((x, y));
        }

        Self::new(vertices)
    }


    pub fn from_rectangle(x: f64, y: f64, width: f64, height: f64) -> Self {
        let vertices = vec![
            (x, y),
            (x + width, y),
            (x + width, y + height),
            (x, y + height),
        ];
        Self::new(vertices)
    }


    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }


    pub fn vertex(&self, index: usize) -> Option<(f64, f64)> {
        self.vertices.get(index).copied()
    }


    pub fn add_vertex(&mut self, x: f64, y: f64) {
        self.vertices.push((x, y));
    }


    pub fn insert_vertex(&mut self, index: usize, x: f64, y: f64) {
        self.vertices.insert(index, (x, y));
    }


    pub fn remove_vertex(&mut self, index: usize) -> Option<(f64, f64)> {
        self.vertices.remove(index)
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


    pub fn centroid(&self) -> Option<(f64, f64)> {
        if self.vertices.is_empty() {
            return None;
        }

        let mut cx = 0.0;
        let mut cy = 0.0;

        for (x, y) in &self.vertices {
            cx += x;
            cy += y;
        }

        Some((cx / self.vertices.len() as f64, cy / self.vertices.len() as f64))
    }


    pub fn perimeter(&self) -> f64 {
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


    pub fn simplify(&self, tolerance: f64) -> Self {
        if self.vertices.len() < 3 {
            return self.clone();
        }

        let mut simplified = Vec::new();

        for (i, &vertex) in self.vertices.iter().enumerate() {
            if i == 0 || i == self.vertices.len() - 1 {
                simplified.push(vertex);
                continue;
            }

            let prev = self.vertices[i - 1];
            let next = self.vertices[(i + 1) % self.vertices.len()];


            let area = (prev.0 - vertex.0) * (next.1 - vertex.1) - (prev.1 - vertex.1) * (next.0 - vertex.0);

            if area.abs() > tolerance {
                simplified.push(vertex);
            }
        }

        Self {
            vertices: simplified,
            closed: self.closed,
        }
    }
}

impl super::types::ShapePrimitive for Polygon {
    fn shape_type(&self) -> super::types::ShapeType {
        super::types::ShapeType::Polygon
    }

    fn bounds(&self) -> super::transform::BoundingBox {
        if self.vertices.is_empty() {
            return super::transform::BoundingBox::default();
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

        super::transform::BoundingBox::new(min_x, min_y, max_x, max_y)
    }

    fn to_path(&self) -> super::super::paths::Path {
        let mut builder = super::super::paths::PathBuilder::new();

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
        self.perimeter()
    }

    fn transform(&mut self, transform: &super::transform::Transform) {
        for (x, y) in &mut self.vertices {
            let (tx, ty) = transform.transform_point(*x, *y);
            *x = tx;
            *y = ty;
        }
    }

    fn transformed(&self, transform: &super::transform::Transform) -> Self {
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
    use super::super::types::ShapePrimitive;

    #[test]
    fn test_polygon_creation() {
        let vertices = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
        let polygon = Polygon::new(vertices);

        assert_eq!(polygon.vertex_count(), 4);
        assert!(polygon.closed);
        assert!(polygon.is_valid());
        assert!(polygon.is_convex());
    }

    #[test]
    fn test_regular_polygon() {
        let triangle = Polygon::regular(50.0, 50.0, 30.0, 3);

        assert_eq!(triangle.vertex_count(), 3);
        assert!(triangle.is_valid());
        assert!(triangle.is_convex());

        let centroid = triangle.centroid();
        assert!(centroid.is_some());
        assert!((centroid.unwrap().0 - 50.0).abs() < 0.001);
        assert!((centroid.unwrap().1 - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_polygon_from_rectangle() {
        let polygon = Polygon::from_rectangle(10.0, 20.0, 100.0, 50.0);

        assert_eq!(polygon.vertex_count(), 4);
        assert_eq!(polygon.area(), 5000.0);
        assert_eq!(polygon.perimeter(), 300.0);
    }

    #[test]
    fn test_polygon_area_perimeter() {
        let square = Polygon::new(vec![
            (0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)
        ]);

        assert_eq!(square.area(), 100.0);
        assert_eq!(square.perimeter(), 40.0);
    }

    #[test]
    fn test_polygon_contains_point() {
        let square = Polygon::new(vec![
            (0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)
        ]);

        assert!(square.contains_point(5.0, 5.0));
        assert!(square.contains_point(1.0, 1.0));
        assert!(!square.contains_point(11.0, 5.0));
        assert!(!square.contains_point(5.0, 11.0));
    }

    #[test]
    fn test_polygon_transform() {
        let mut polygon = Polygon::new(vec![
            (0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)
        ]);
        let transform = super::transform::Transform::translation(5.0, 10.0);

        polygon.transform(&transform);

        assert_eq!(polygon.vertices[0], (5.0, 10.0));
        assert_eq!(polygon.vertices[1], (15.0, 10.0));
        assert_eq!(polygon.vertices[2], (15.0, 20.0));
        assert_eq!(polygon.vertices[3], (5.0, 20.0));
    }

    #[test]
    fn test_polygon_validation() {
        let valid_polygon = Polygon::new(vec![
            (0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)
        ]);
        assert!(valid_polygon.validate().is_ok());

        let empty_polygon = Polygon::new(vec![]);
        assert!(empty_polygon.validate().is_err());

        let invalid_closed = Polygon::new(vec![(0.0, 0.0), (10.0, 0.0)]);
        assert!(invalid_closed.validate().is_err());

        let valid_open = Polygon::open(vec![(0.0, 0.0), (10.0, 0.0)]);
        assert!(valid_open.validate().is_ok());
    }

    #[test]
    fn test_polygon_bounds() {
        let polygon = Polygon::new(vec![
            (10.0, 20.0), (100.0, 30.0), (80.0, 90.0), (20.0, 80.0)
        ]);
        let bounds = polygon.bounds();

        assert_eq!(bounds.min_x, 10.0);
        assert_eq!(bounds.min_y, 20.0);
        assert_eq!(bounds.max_x, 100.0);
        assert_eq!(bounds.max_y, 90.0);
        assert_eq!(bounds.center(), (55.0, 55.0));
    }

    #[test]
    fn test_polygon_to_path() {
        let polygon = Polygon::new(vec![
            (0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)
        ]);
        let path = polygon.to_path();

        assert!(path.closed);
        assert_eq!(path.area(), 100.0);
    }

    #[test]
    fn test_polygon_simplify() {
        let polygon = Polygon::new(vec![
            (0.0, 0.0), (5.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)
        ]);

        let simplified = polygon.simplify(0.1);
        assert!(simplified.vertex_count() < polygon.vertex_count());
    }

    #[test]
    fn test_vertex_operations() {
        let mut polygon = Polygon::new(vec![(0.0, 0.0), (10.0, 0.0)]);

        polygon.add_vertex(10.0, 10.0);
        assert_eq!(polygon.vertex_count(), 3);

        polygon.insert_vertex(1, 5.0, 0.0);
        assert_eq!(polygon.vertex_count(), 4);
        assert_eq!(polygon.vertex(1), Some((5.0, 0.0)));

        let removed = polygon.remove_vertex(1);
        assert_eq!(removed, Some((5.0, 0.0)));
        assert_eq!(polygon.vertex_count(), 3);
    }

    #[test]
    fn test_rotated_regular_polygon() {
        let rotated = Polygon::regular_rotated(50.0, 50.0, 30.0, 4, std::f64::consts::PI / 4.0);

        assert_eq!(rotated.vertex_count(), 4);


        let first_vertex = rotated.vertex(0).unwrap();
        let expected_x = 50.0 + 30.0 * (std::f64::consts::PI / 4.0).cos();
        let expected_y = 50.0 + 30.0 * (std::f64::consts::PI / 4.0).sin();

        assert!((first_vertex.0 - expected_x).abs() < 0.001);
        assert!((first_vertex.1 - expected_y).abs() < 0.001);
    }
}
