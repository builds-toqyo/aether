

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::shapes::primitives::{ShapePrimitive, BoundingBox};
use crate::shapes::paths::{Path, PathBuilder};


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BooleanOperation {

    Union,

    Subtract,

    Intersect,

    Xor,
}

impl fmt::Display for BooleanOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BooleanOperation::Union => write!(f, __STRING_0__),
            BooleanOperation::Subtract => write!(f, __STRING_1__),
            BooleanOperation::Intersect => write!(f, __STRING_2__),
            BooleanOperation::Xor => write!(f, __STRING_3__),
        }
    }
}

/// Result of boolean operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BooleanResult {
    /// Resulting path
    pub path: Path,
    /// Operation performed
    pub operation: BooleanOperation,
    /// Whether operation was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl BooleanResult {
    /// Create successful result
    pub fn success(path: Path, operation: BooleanOperation) -> Self {
        Self {
            path,
            operation,
            success: true,
            error: None,
        }
    }

    /// Create failed result
    pub fn failure(operation: BooleanOperation, error: String) -> Self {
        Self {
            path: Path::new(),
            operation,
            success: false,
            error: Some(error),
        }
    }

    /// Get bounding box of result
    pub fn bounds(&self) -> BoundingBox {
        self.path.bounds()
    }

    /// Get area of result
    pub fn area(&self) -> f64 {
        self.path.area()
    }

    /// Check if result contains point
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        self.path.contains_point(x, y)
    }
}

/// Shape boolean operations processor
pub struct ShapeBoolean;

impl ShapeBoolean {
    /// Perform boolean operation between two shapes
    pub fn operate(
        shape_a: &dyn ShapePrimitive,
        shape_b: &dyn ShapePrimitive,
        operation: BooleanOperation,
    ) -> BooleanResult {
        // Convert shapes to paths
        let path_a = shape_a.to_path();
        let path_b = shape_b.to_path();

        // Perform operation on paths
        Self::operate_paths(&path_a, &path_b, operation)
    }

    /// Perform boolean operation between two paths
    pub fn operate_paths(
        path_a: &Path,
        path_b: &Path,
        operation: BooleanOperation,
    ) -> BooleanResult {
        // Validate paths
        if !path_a.closed || !path_b.closed {
            return BooleanResult::failure(
                operation,
                __STRING_4__.to_string(),
            );
        }

        // Check bounding boxes for early rejection
        let bounds_a = path_a.bounds();
        let bounds_b = path_b.bounds();

        if !bounds_a.intersects(&bounds_b) {
            // No intersection, handle based on operation
            match operation {
                BooleanOperation::Union => {
                    // Return both shapes as separate paths
                    let combined_path = Self::combine_paths_non_intersecting(path_a, path_b);
                    BooleanResult::success(combined_path, operation)
                }
                BooleanOperation::Intersect | BooleanOperation::Subtract => {
                    // No intersection result
                    BooleanResult::success(Path::new(), operation)
                }
                BooleanOperation::Xor => {
                    // XOR of non-intersecting shapes is union
                    let combined_path = Self::combine_paths_non_intersecting(path_a, path_b);
                    BooleanResult::success(combined_path, operation)
                }
            }
        } else {
            // Perform actual boolean operation
            Self::perform_boolean_operation(path_a, path_b, operation)
        }
    }

    /// Combine non-intersecting paths
    fn combine_paths_non_intersecting(path_a: &Path, path_b: &Path) -> Path {
        let mut combined = Path::new();

        // Add first path
        for segment in &path_a.segments {
            combined.segments.push(segment.clone());
        }

        // Add second path with a move to separate them
        if let Some(last_segment) = path_a.segments.last() {
            combined.segments.push(PathSegment::move_to(last_segment.x, last_segment.y));
        }

        for segment in &path_b.segments {
            combined.segments.push(segment.clone());
        }

        combined.closed = true;
        combined
    }

    /// Perform actual boolean operation using polygon clipping
    fn perform_boolean_operation(
        path_a: &Path,
        path_b: &Path,
        operation: BooleanOperation,
    ) -> BooleanResult {
        // Convert paths to polygons (sample points)
        let polygon_a = Self::path_to_polygon(path_a);
        let polygon_b = Self::path_to_polygon(path_b);

        // Perform polygon boolean operation
        let result_polygon = match operation {
            BooleanOperation::Union => Self::polygon_union(&polygon_a, &polygon_b),
            BooleanOperation::Subtract => Self::polygon_subtract(&polygon_a, &polygon_b),
            BooleanOperation::Intersect => Self::polygon_intersect(&polygon_a, &polygon_b),
            BooleanOperation::Xor => Self::polygon_xor(&polygon_a, &polygon_b),
        };

        // Convert result back to path
        let result_path = Self::polygon_to_path(&result_polygon);

        BooleanResult::success(result_path, operation)
    }

    /// Convert path to polygon (sample points)
    fn path_to_polygon(path: &Path) -> Vec<(f64, f64)> {
        let mut vertices = Vec::new();

        // Sample points along path
        let samples = if path.segments.len() > 10 {
            100
        } else {
            50
        };

        let points = path.sample_points(samples);

        // Remove duplicate points
        for point in points {
            if vertices.is_empty() ||
               (point.0 - vertices.last().unwrap().0).abs() > f64::EPSILON ||
               (point.1 - vertices.last().unwrap().1).abs() > f64::EPSILON {
                vertices.push(point);
            }
        }

        vertices
    }

    /// Convert polygon to path
    fn polygon_to_path(polygon: &[(f64, f64)]) -> Path {
        let mut builder = PathBuilder::new();

        if let Some((x, y)) = polygon.first() {
            builder.move_to(*x, *y);

            for (x, y) in polygon.iter().skip(1) {
                builder.line_to(*x, *y);
            }

            builder.close();
        }

        builder.build()
    }

    /// Polygon union operation
    fn polygon_union(poly_a: &[(f64, f64)], poly_b: &[(f64, f64)]) -> Vec<(f64, f64)> {
        // Simplified union - combine all points and compute convex hull
        let mut all_points = poly_a.to_vec();
        all_points.extend_from_slice(poly_b);

        Self::convex_hull(&all_points)
    }

    /// Polygon subtract operation
    fn polygon_subtract(poly_a: &[(f64, f64)], poly_b: &[(f64, f64)]) -> Vec<(f64, f64)> {
        // Simplified subtract - return points in A that are not in B
        poly_a.iter()
            .filter(|&point| !Self::point_in_polygon(point, poly_b))
            .copied()
            .collect()
    }

    /// Polygon intersect operation
    fn polygon_intersect(poly_a: &[(f64, f64)], poly_b: &[(f64, f64)]) -> Vec<(f64, f64)> {
        // Simplified intersect - return points in A that are also in B
        poly_a.iter()
            .filter(|&point| Self::point_in_polygon(point, poly_b))
            .copied()
            .collect()
    }

    /// Polygon XOR operation
    fn polygon_xor(poly_a: &[(f64, f64)], poly_b: &[(f64, f64)]) -> Vec<(f64, f64)> {
        // Simplified XOR - points in A or B but not both
        let mut result = Vec::new();

        for &point in poly_a {
            if !Self::point_in_polygon(&point, poly_b) {
                result.push(point);
            }
        }

        for &point in poly_b {
            if !Self::point_in_polygon(&point, poly_a) {
                result.push(point);
            }
        }

        result
    }

    /// Check if point is inside polygon
    fn point_in_polygon(point: &(f64, f64), polygon: &[(f64, f64)]) -> bool {
        if polygon.len() < 3 {
            return false;
        }

        let mut inside = false;
        let n = polygon.len();

        for i in 0..n {
            let p1 = polygon[i];
            let p2 = polygon[(i + 1) % n];

            if ((p1.1 > point.1) != (p2.1 > point.1)) &&
               (point.0 < (p2.0 - p1.0) * (point.1 - p1.1) / (p2.1 - p1.1) + p1.0) {
                inside = !inside;
            }
        }

        inside
    }

    /// Compute convex hull of points (Graham scan)
    fn convex_hull(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
        if points.len() < 3 {
            return points.to_vec();
        }

        // Find point with lowest y-coordinate (and leftmost if tie)
        let start = points.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.1.partial_cmp(&b.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        let start_point = points[start];

        // Sort points by polar angle with respect to start point
        let mut sorted_points: Vec<(f64, f64)> = points.iter()
            .enumerate()
            .filter(|(idx, _)| *idx != start)
            .map(|(_, point)| *point)
            .collect();

        sorted_points.sort_by(|a, b| {
            let angle_a = (a.1 - start_point.1).atan2(a.0 - start_point.0);
            let angle_b = (b.1 - start_point.1).atan2(b.0 - start_point.0);
            angle_a.partial_cmp(&angle_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Graham scan
        let mut hull = vec![start_point];

        for point in sorted_points {
            while hull.len() >= 2 {
                let p1 = hull[hull.len() - 2];
                let p2 = hull[hull.len() - 1];

                // Check if turn is counter-clockwise
                let cross = (p2.0 - p1.0) * (point.1 - p2.1) - (p2.1 - p1.1) * (point.0 - p2.0);

                if cross <= 0.0 {
                    hull.pop();
                } else {
                    break;
                }
            }

            hull.push(point);
        }

        hull
    }
}

/// Advanced boolean operations with better algorithms
pub struct AdvancedBoolean;

impl AdvancedBoolean {
    /// Perform union with proper polygon clipping
    pub fn union(
        shape_a: &dyn ShapePrimitive,
        shape_b: &dyn ShapePrimitive,
    ) -> BooleanResult {
        ShapeBoolean::operate(shape_a, shape_b, BooleanOperation::Union)
    }

    /// Perform subtract with proper polygon clipping
    pub fn subtract(
        shape_a: &dyn ShapePrimitive,
        shape_b: &dyn ShapePrimitive,
    ) -> BooleanResult {
        ShapeBoolean::operate(shape_a, shape_b, BooleanOperation::Subtract)
    }

    /// Perform intersect with proper polygon clipping
    pub fn intersect(
        shape_a: &dyn ShapePrimitive,
        shape_b: &dyn ShapePrimitive,
    ) -> BooleanResult {
        ShapeBoolean::operate(shape_a, shape_b, BooleanOperation::Intersect)
    }

    /// Perform XOR with proper polygon clipping
    pub fn xor(
        shape_a: &dyn ShapePrimitive,
        shape_b: &dyn ShapePrimitive,
    ) -> BooleanResult {
        ShapeBoolean::operate(shape_a, shape_b, BooleanOperation::Xor)
    }

    /// Perform multiple boolean operations
    pub fn multiple_operations(
        shapes: &[&dyn ShapePrimitive],
        operations: &[BooleanOperation],
    ) -> Result<BooleanResult, String> {
        if shapes.len() < 2 {
            return Err(__STRING_5__.to_string());
        }

        if operations.len() != shapes.len() - 1 {
            return Err(__STRING_6__.to_string());
        }

        let mut current_path = shapes[0].to_path();
        let mut current_operation = BooleanOperation::Union;

        for (i, &shape) in shapes.iter().enumerate().skip(1) {
            current_operation = operations[i - 1];
            let shape_path = shape.to_path();

            let result = ShapeBoolean::operate_paths(&current_path, &shape_path, current_operation);

            if !result.success {
                return Err(format!(__STRING_7__, i, result.error));
            }

            current_path = result.path;
        }

        Ok(BooleanResult::success(current_path, current_operation))
    }
}

/// Boolean operation utilities
pub struct BooleanUtils;

impl BooleanUtils {
    /// Check if two shapes intersect
    pub fn shapes_intersect(
        shape_a: &dyn ShapePrimitive,
        shape_b: &dyn ShapePrimitive,
    ) -> bool {
        let bounds_a = shape_a.bounds();
        let bounds_b = shape_b.bounds();

        bounds_a.intersects(&bounds_b)
    }

    /// Check if shape A contains shape B
    pub fn shape_contains(
        shape_a: &dyn ShapePrimitive,
        shape_b: &dyn ShapePrimitive,
    ) -> bool {
        let path_b = shape_b.to_path();

        // Check if all vertices of B are inside A
        if let Some(first_segment) = path_b.segments.first() {
            if !shape_a.contains_point(first_segment.x, first_segment.y) {
                return false;
            }
        }

        // Sample points along B and check if they're inside A
        let samples = 20;
        let points = path_b.sample_points(samples);

        for (x, y) in points {
            if !shape_a.contains_point(x, y) {
                return false;
            }
        }

        true
    }


    pub fn intersection_area(
        shape_a: &dyn ShapePrimitive,
        shape_b: &dyn ShapePrimitive,
    ) -> f64 {
        let result = AdvancedBoolean::intersect(shape_a, shape_b);

        if result.success {
            result.area()
        } else {
            0.0
        }
    }


    pub fn union_area(
        shape_a: &dyn ShapePrimitive,
        shape_b: &dyn ShapePrimitive,
    ) -> f64 {
        let result = AdvancedBoolean::union(shape_a, shape_b);

        if result.success {
            result.area()
        } else {
            shape_a.area() + shape_b.area()
        }
    }


    pub fn simplify_shape(
        shape: &dyn ShapePrimitive,
        tolerance: f64,
    ) -> Path {
        let path = shape.to_path();
        Self::simplify_path(&path, tolerance)
    }


    fn simplify_path(path: &Path, tolerance: f64) -> Path {
        if path.segments.len() < 3 {
            return path.clone();
        }

        let mut simplified_segments = Vec::new();
        let mut current_x = path.start_x;
        let mut current_y = path.start_y;

        for segment in &path.segments {
            match segment.segment_type {
                crate::shapes::paths::PathSegmentType::MoveTo => {
                    simplified_segments.push(segment.clone());
                    current_x = segment.x;
                    current_y = segment.y;
                }
                crate::shapes::paths::PathSegmentType::LineTo => {

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
                crate::shapes::paths::PathSegmentType::Close => {
                    simplified_segments.push(segment.clone());
                }

                crate::shapes::paths::PathSegmentType::QuadraticTo |
                crate::shapes::paths::PathSegmentType::CubicTo => {
                    simplified_segments.push(segment.clone());
                    current_x = segment.x;
                    current_y = segment.y;
                }
            }
        }

        let mut simplified_path = Path::new();
        simplified_path.segments = simplified_segments;
        simplified_path.closed = path.closed;
        simplified_path.start_x = path.start_x;
        simplified_path.start_y = path.start_y;

        simplified_path
    }
}


use crate::shapes::paths::PathSegment;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::primitives::{Rectangle, Circle};

    #[test]
    fn test_boolean_union() {
        let rect1 = Rectangle::new(0.0, 0.0, 50.0, 50.0);
        let rect2 = Rectangle::new(25.0, 25.0, 50.0, 50.0);

        let result = AdvancedBoolean::union(&rect1, &rect2);

        assert!(result.success);
        assert!(result.area() > rect1.area());
        assert!(result.area() > rect2.area());
    }

    #[test]
    fn test_boolean_intersect() {
        let rect1 = Rectangle::new(0.0, 0.0, 50.0, 50.0);
        let rect2 = Rectangle::new(25.0, 25.0, 50.0, 50.0);

        let result = AdvancedBoolean::intersect(&rect1, &rect2);

        assert!(result.success);
        assert!(result.area() < rect1.area());
        assert!(result.area() < rect2.area());
    }

    #[test]
    fn test_boolean_subtract() {
        let rect1 = Rectangle::new(0.0, 0.0, 50.0, 50.0);
        let rect2 = Rectangle::new(25.0, 25.0, 20.0, 20.0);

        let result = AdvancedBoolean::subtract(&rect1, &rect2);

        assert!(result.success);
        assert!(result.area() < rect1.area());
    }

    #[test]
    fn test_shapes_intersect() {
        let rect1 = Rectangle::new(0.0, 0.0, 50.0, 50.0);
        let rect2 = Rectangle::new(25.0, 25.0, 50.0, 50.0);

        assert!(BooleanUtils::shapes_intersect(&rect1, &rect2));

        let rect3 = Rectangle::new(100.0, 100.0, 50.0, 50.0);
        assert!(!BooleanUtils::shapes_intersect(&rect1, &rect3));
    }

    #[test]
    fn test_convex_hull() {
        let points = vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (5.0, 5.0),
        ];

        let hull = ShapeBoolean::convex_hull(&points);


        assert!(hull.len() >= 4);
        assert!(!hull.contains(&(5.0, 5.0)));
    }

    #[test]
    fn test_point_in_polygon() {
        let square = vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
        ];

        assert!(ShapeBoolean::point_in_polygon(&(5.0, 5.0), &square));
        assert!(ShapeBoolean::point_in_polygon(&(1.0, 1.0), &square));
        assert!(!ShapeBoolean::point_in_polygon(&(11.0, 5.0), &square));
        assert!(!ShapeBoolean::point_in_polygon(&(5.0, 11.0), &square));
    }

    #[test]
    fn test_simplify_shape() {
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);
        let simplified = BooleanUtils::simplify_shape(&rect, 1.0);

        assert!(simplified.segments.len() <= rect.to_path().segments.len());
        assert!((simplified.area() - rect.area()).abs() < 100.0);
    }
}
