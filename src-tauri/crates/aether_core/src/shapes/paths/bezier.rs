pub struct BezierCurve;

impl BezierCurve {
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


    pub fn cubic_bezier_normal(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
    ) -> (f64, f64) {
        let (tx, ty) = Self::cubic_bezier_tangent(t, p0, p1, p2, p3);
        let length = (tx * tx + ty * ty).sqrt();

        if length > 0.0 {
            (-ty / length, tx / length)
        } else {
            (0.0, 1.0)
        }
    }

    pub fn quadratic_bezier_normal(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
    ) -> (f64, f64) {
        let (tx, ty) = Self::quadratic_bezier_tangent(t, p0, p1, p2);
        let length = (tx * tx + ty * ty).sqrt();

        if length > 0.0 {
            (-ty / length, tx / length)
        } else {
            (0.0, 1.0)
        }
    }

    pub fn cubic_bezier_curvature(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
    ) -> f64 {
        let (tx, ty) = Self::cubic_bezier_tangent(t, p0, p1, p2, p3);
        let (dtx, dty) = Self::cubic_bezier_derivative2(t, p0, p1, p2, p3);

        let numerator = (tx * dty - ty * dtx).abs();
        let denominator = (tx * tx + ty * ty).powf(1.5);

        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }


    pub fn quadratic_bezier_curvature(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
    ) -> f64 {
        let (tx, ty) = Self::quadratic_bezier_tangent(t, p0, p1, p2);
        let (dtx, dty) = Self::quadratic_bezier_derivative2(t, p0, p1, p2);

        let numerator = (tx * dty - ty * dtx).abs();
        let denominator = (tx * tx + ty * ty).powf(1.5);

        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }


    fn cubic_bezier_derivative2(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
    ) -> (f64, f64) {
        let mt = 1.0 - t;

        let x = 6.0 * mt * (p2.0 - 2.0 * p1.0 + p0.0) + 6.0 * t * (p3.0 - 2.0 * p2.0 + p1.0);
        let y = 6.0 * mt * (p2.1 - 2.0 * p1.1 + p0.1) + 6.0 * t * (p3.1 - 2.0 * p2.1 + p1.1);

        (x, y)
    }


    fn quadratic_bezier_derivative2(
        _t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
    ) -> (f64, f64) {
        let x = 2.0 * (p2.0 - 2.0 * p1.0 + p0.0);
        let y = 2.0 * (p2.1 - 2.0 * p1.1 + p0.1);

        (x, y)
    }


    pub fn subdivide_cubic_bezier(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
    ) -> (((f64, f64), (f64, f64), (f64, f64), (f64, f64)), ((f64, f64), (f64, f64), (f64, f64), (f64, f64))) {
        let p01 = ((p0.0 + p1.0) / 2.0, (p0.1 + p1.1) / 2.0);
        let p12 = ((p1.0 + p2.0) / 2.0, (p1.1 + p2.1) / 2.0);
        let p23 = ((p2.0 + p3.0) / 2.0, (p2.1 + p3.1) / 2.0);

        let p012 = ((p01.0 + p12.0) / 2.0, (p01.1 + p12.1) / 2.0);
        let p123 = ((p12.0 + p23.0) / 2.0, (p12.1 + p23.1) / 2.0);

        let p0123 = ((p012.0 + p123.0) / 2.0, (p012.1 + p123.1) / 2.0);

        let left = (p0, p01, p012, p0123);
        let right = (p0123, p123, p23, p3);

        (left, right)
    }

    pub fn subdivide_quadratic_bezier(
        t: f64,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
    ) -> (((f64, f64), (f64, f64), (f64, f64)), ((f64, f64), (f64, f64), (f64, f64))) {
        let p01 = ((p0.0 + p1.0) / 2.0, (p0.1 + p1.1) / 2.0);
        let p12 = ((p1.0 + p2.0) / 2.0, (p1.1 + p2.1) / 2.0);
        let p012 = ((p01.0 + p12.0) / 2.0, (p01.1 + p12.1) / 2.0);

        let left = (p0, p01, p012);
        let right = (p012, p12, p2);

        (left, right)
    }

    pub fn cubic_bezier_closest_point(
        point: (f64, f64),
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
        tolerance: f64,
    ) -> f64 {
        let mut t_min = 0.0;
        let mut min_dist = f64::INFINITY;


        for i in 0..=20 {
            let t = i as f64 / 20.0;
            let bezier_point = Self::cubic_bezier_point(t, p0, p1, p2, p3);
            let dist = (point.0 - bezier_point.0).powi(2) + (point.1 - bezier_point.1).powi(2);

            if dist < min_dist {
                min_dist = dist;
                t_min = t;
            }
        }


        for _ in 0..10 {
            let bezier_point = Self::quadratic_bezier_point(t_min, p0, p1, p2);
            let tangent = Self::quadratic_bezier_tangent(t_min, p0, p1, p2);

            let dx = bezier_point.0 - point.0;
            let dy = bezier_point.1 - point.1;
            let denominator = tangent.0 * dx + tangent.1 * dy;

            if denominator.abs() < tolerance {
                break;
            }

            let numerator = tangent.0 * tangent.0 + tangent.1 * tangent.1;
            let delta = denominator / numerator;

            let new_t = (t_min - delta).clamp(0.0, 1.0);
            if (new_t - t_min).abs() < tolerance {
                break;
            }
            t_min = new_t;
        }

        t_min
    }


    pub fn cubic_bezier_length(
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
        tolerance: f64,
    ) -> f64 {
        let mut length = 0.0;
        let mut stack = vec![(p0, p1, p2, p3, 0.0, 1.0)];

        while let Some((cp0, cp1, cp2, cp3, t0, t1)) = stack.pop() {
            let mid_t = (t0 + t1) / 2.0;
            let mid_point = Self::cubic_bezier_point(mid_t, cp0, cp1, cp2, cp3);

            let d1 = Self::point_to_line_distance(mid_point, cp0, cp3);
            let d2 = Self::point_to_line_distance(cp1, cp0, cp3);
            let d3 = Self::point_to_line_distance(cp2, cp0, cp3);

            if d1.max(d2).max(d3) <= tolerance {

                let dx = cp3.0 - cp0.0;
                let dy = cp3.1 - cp0.1;
                length += (dx * dx + dy * dy).sqrt();
            } else {

                let (left, right) = Self::subdivide_cubic_bezier(0.5, cp0, cp1, cp2, cp3);

                stack.push((right.0, right.1, right.2, right.3, mid_t, t1));
                stack.push((left.0, left.1, left.2, left.3, t0, mid_t));
            }
        }

        length
    }

    pub fn quadratic_bezier_length(
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        tolerance: f64,
    ) -> f64 {
        let mut length = 0.0;
        let mut stack = vec![(p0, p1, p2, 0.0, 1.0)];

        while let Some((cp0, cp1, cp2, t0, t1)) = stack.pop() {
            let mid_t = (t0 + t1) / 2.0;
            let mid_point = Self::quadratic_bezier_point(mid_t, cp0, cp1, cp2);

            let d1 = Self::point_to_line_distance(mid_point, cp0, cp2);
            let d2 = Self::point_to_line_distance(cp1, cp0, cp2);

            if d1.max(d2) <= tolerance {
                let dx = cp2.0 - cp0.0;
                let dy = cp2.1 - cp0.1;
                length += (dx * dx + dy * dy).sqrt();
            } else {

                let (left, right) = Self::subdivide_quadratic_bezier(0.5, cp0, cp1, cp2);

                stack.push((right.0, right.1, right.2, mid_t, t1));
                stack.push((left.0, left.1, left.2, t0, mid_t));
            }
        }

        length
    }

    fn point_to_line_distance(point: (f64, f64), line_start: (f64, f64), line_end: (f64, f64)) -> f64 {
        let dx = line_end.0 - line_start.0;
        let dy = line_end.1 - line_start.1;

        if dx == 0.0 && dy == 0.0 {

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


pub struct BezierUtils;

impl BezierUtils {

    pub fn cubic_to_quadratic(
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
    ) -> Vec<((f64, f64), (f64, f64), (f64, f64))> {
        let mut result = Vec::new();
        let mut stack = vec![(p0, p1, p2, p3)];

        while let Some((cp0, cp1, cp2, cp3)) = stack.pop() {

            let mid_t = 0.5;
            let cubic_mid = BezierCurve::cubic_bezier_point(mid_t, cp0, cp1, cp2, cp3);


            let qp1 = (
                2.0 * cp1.0 - (cp0.0 + cp3.0) / 2.0,
                2.0 * cp1.1 - (cp0.1 + cp3.1) / 2.0,
            );
            let quad_mid = BezierCurve::quadratic_bezier_point(mid_t, cp0, qp1, cp3);

            let error = (cubic_mid.0 - quad_mid.0).abs() + (cubic_mid.1 - quad_mid.1).abs();

            if error < 0.1 {
                result.push((cp0, qp1, cp3));
            } else {

                let (left, right) = BezierCurve::subdivide_cubic_bezier(0.5, cp0, cp1, cp2, cp3);
                stack.push((right.0, right.1, right.2, right.3));
                stack.push((left.0, left.1, left.2, left.3));
            }
        }

        result
    }


    pub fn cubic_bezier_bounds(
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
    ) -> ((f64, f64), (f64, f64)) {
        let mut min_x = p0.0.min(p3.0);
        let mut max_x = p0.0.max(p3.0);
        let mut min_y = p0.1.min(p3.1);
        let mut max_y = p0.1.max(p3.1);


        let x_extrema = Self::cubic_bezier_extrema(p0.0, p1.0, p2.0, p3.0);
        for t in x_extrema {
            if t >= 0.0 && t <= 1.0 {
                let point = BezierCurve::cubic_bezier_point(t, p0, p1, p2, p3);
                min_x = min_x.min(point.0);
                max_x = max_x.max(point.0);
            }
        }


        let y_extrema = Self::cubic_bezier_extrema(p0.1, p1.1, p2.1, p3.1);
        for t in y_extrema {
            if t >= 0.0 && t <= 1.0 {
                let point = BezierCurve::cubic_bezier_point(t, p0, p1, p2, p3);
                min_y = min_y.min(point.1);
                max_y = max_y.max(point.1);
            }
        }

        ((min_x, min_y), (max_x, max_y))
    }


    fn cubic_bezier_extrema(p0: f64, p1: f64, p2: f64, p3: f64) -> Vec<f64> {
        let mut extrema = Vec::new();

        let a = 3.0 * (p3 - 3.0 * p2 + 3.0 * p1 - p0);
        let b = 6.0 * (p2 - 2.0 * p1 + p0);
        let c = 3.0 * (p1 - p0);

        if a.abs() > f64::EPSILON {
            let discriminant = b * b - 4.0 * a * c;
            if discriminant >= 0.0 {
                let sqrt_d = discriminant.sqrt();
                let t1 = (-b + sqrt_d) / (2.0 * a);
                let t2 = (-b - sqrt_d) / (2.0 * a);

                if t1 >= 0.0 && t1 <= 1.0 {
                    extrema.push(t1);
                }
                if t2 >= 0.0 && t2 <= 1.0 && (t2 - t1).abs() > f64::EPSILON {
                    extrema.push(t2);
                }
            }
        } else if b.abs() > f64::EPSILON {
            let t = -c / b;
            if t >= 0.0 && t <= 1.0 {
                extrema.push(t);
            }
        }

        extrema
    }


    pub fn quadratic_bezier_bounds(
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
    ) -> ((f64, f64), (f64, f64)) {
        let mut min_x = p0.0.min(p2.0);
        let mut max_x = p0.0.max(p2.0);
        let mut min_y = p0.1.min(p2.1);
        let mut max_y = p0.1.max(p2.1);


        let x_extrema = Self::quadratic_bezier_extrema(p0.0, p1.0, p2.0);
        for t in x_extrema {
            if t >= 0.0 && t <= 1.0 {
                let point = BezierCurve::quadratic_bezier_point(t, p0, p1, p2);
                min_x = min_x.min(point.0);
                max_x = max_x.max(point.0);
            }
        }


        let y_extrema = Self::quadratic_bezier_extrema(p0.1, p1.1, p2.1);
        for t in y_extrema {
            if t >= 0.0 && t <= 1.0 {
                let point = BezierCurve::quadratic_bezier_point(t, p0, p1, p2);
                min_y = min_y.min(point.1);
                max_y = max_y.max(point.1);
            }
        }

        ((min_x, min_y), (max_x, max_y))
    }


    fn quadratic_bezier_extrema(p0: f64, p1: f64, p2: f64) -> Vec<f64> {
        let mut extrema = Vec::new();


        let a = 2.0 * (p2 - 2.0 * p1 + p0);
        let b = 2.0 * (p1 - p0);

        if a.abs() > f64::EPSILON {
            let t = -b / a;
            if t >= 0.0 && t <= 1.0 {
                extrema.push(t);
            }
        }

        extrema
    }


    pub fn approximate_with_lines(
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
        tolerance: f64,
        max_segments: usize,
    ) -> Vec<(f64, f64)> {
        let mut points = Vec::new();
        let mut stack = vec![(p0, p1, p2, p3, 0.0, 1.0)];

        while let Some((cp0, cp1, cp2, cp3, t0, t1)) = stack.pop() {
            if points.len() >= max_segments {
                break;
            }

            let mid_t = (t0 + t1) / 2.0;
            let mid_point = BezierCurve::cubic_bezier_point(mid_t, cp0, cp1, cp2, cp3);


            let d1 = BezierCurve::point_to_line_distance(mid_point, cp0, cp3);
            let d2 = BezierCurve::point_to_line_distance(cp1, cp0, cp3);
            let d3 = BezierCurve::point_to_line_distance(cp2, cp0, cp3);

            if d1.max(d2).max(d3) <= tolerance || (t1 - t0) < 0.01 {

                points.push(cp3);
            } else {

                let (left, right) = BezierCurve::subdivide_cubic_bezier(0.5, cp0, cp1, cp2, cp3);

                stack.push((right.0, right.1, right.2, right.3, mid_t, t1));
                stack.push((left.0, left.1, left.2, left.3, t0, mid_t));
            }
        }

        points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubic_bezier_point() {
        let p0 = (0.0, 0.0);
        let p1 = (50.0, 0.0);
        let p2 = (100.0, 50.0);
        let p3 = (100.0, 100.0);

        let point = BezierCurve::cubic_bezier_point(0.5, p0, p1, p2, p3);
        assert!(point.0 > 40.0 && point.0 < 60.0);
        assert!(point.1 > 20.0 && point.1 < 30.0);


        assert_eq!(BezierCurve::cubic_bezier_point(0.0, p0, p1, p2, p3), p0);
        assert_eq!(BezierCurve::cubic_bezier_point(1.0, p0, p1, p2, p3), p3);
    }

    #[test]
    fn test_quadratic_bezier_point() {
        let p0 = (0.0, 0.0);
        let p1 = (50.0, 0.0);
        let p2 = (100.0, 0.0);

        let point = BezierCurve::quadratic_bezier_point(0.5, p0, p1, p2);
        assert_eq!(point, (50.0, 0.0));


        assert_eq!(BezierCurve::quadratic_bezier_point(0.0, p0, p1, p2), p0);
        assert_eq!(BezierCurve::quadratic_bezier_point(1.0, p0, p1, p2), p2);
    }

    #[test]
    fn test_bezier_tangent() {
        let p0 = (0.0, 0.0);
        let p1 = (50.0, 0.0);
        let p2 = (100.0, 50.0);
        let p3 = (100.0, 100.0);

        let tangent = BezierCurve::cubic_bezier_tangent(0.5, p0, p1, p2, p3);
        assert!(tangent.0 > 0.0);
        assert!(tangent.1 > 0.0);
    }

    #[test]
    fn test_bezier_normal() {
        let p0 = (0.0, 0.0);
        let p1 = (50.0, 0.0);
        let p2 = (100.0, 50.0);
        let p3 = (100.0, 100.0);

        let normal = BezierCurve::cubic_bezier_normal(0.5, p0, p1, p2, p3);
        let tangent = BezierCurve::cubic_bezier_tangent(0.5, p0, p1, p2, p3);


        let dot_product = normal.0 * tangent.0 + normal.1 * tangent.1;
        assert!(dot_product.abs() < 0.001);
    }

    #[test]
    fn test_bezier_subdivision() {
        let p0 = (0.0, 0.0);
        let p1 = (50.0, 0.0);
        let p2 = (100.0, 50.0);
        let p3 = (100.0, 100.0);

        let (left, right) = BezierCurve::subdivide_cubic_bezier(0.5, p0, p1, p2, p3);


        assert_eq!((left.0, left.1), p0);
        assert_eq!((right.6, right.7), p3);


        let left_mid = BezierCurve::cubic_bezier_point(1.0, (left.0, left.1), (left.2, left.3), (left.4, left.5), (left.6, left.7));
        let right_mid = BezierCurve::cubic_bezier_point(0.0, (right.0, right.1), (right.2, right.3), (right.4, right.5), (right.6, right.7));
        assert!((left_mid.0 - right_mid.0).abs() < 0.001);
        assert!((left_mid.1 - right_mid.1).abs() < 0.001);
    }

    #[test]
    fn test_bezier_length() {
        let p0 = (0.0, 0.0);
        let p1 = (50.0, 0.0);
        let p2 = (100.0, 0.0);
        let p3 = (100.0, 0.0);

        let length = BezierCurve::cubic_bezier_length(p0, p1, p2, p3, 0.1);
        assert!((length - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_bezier_closest_point() {
        let p0 = (0.0, 0.0);
        let p1 = (50.0, 0.0);
        let p2 = (100.0, 50.0);
        let p3 = (100.0, 100.0);

        let point = (50.0, 25.0);
        let t = BezierCurve::cubic_bezier_closest_point(point, p0, p1, p2, p3, 0.01);

        let closest = BezierCurve::cubic_bezier_point(t, p0, p1, p2, p3);
        let dist = ((point.0 - closest.0).powi(2) + (point.1 - closest.1).powi(2)).sqrt();

        assert!(dist < 5.0);
    }

    #[test]
    fn test_bezier_bounds() {
        let p0 = (0.0, 0.0);
        let p1 = (50.0, -10.0);
        let p2 = (100.0, 110.0);
        let p3 = (100.0, 100.0);

        let (min, max) = BezierUtils::cubic_bezier_bounds(p0, p1, p2, p3);

        assert!(min.0 <= 0.0);
        assert!(max.0 >= 100.0);
        assert!(min.1 <= -10.0);
        assert!(max.1 >= 110.0);
    }

    #[test]
    fn test_cubic_to_quadratic() {
        let p0 = (0.0, 0.0);
        let p1 = (50.0, 0.0);
        let p2 = (100.0, 0.0);
        let p3 = (100.0, 100.0);

        let quads = BezierUtils::cubic_to_quadratic(p0, p1, p2, p3);
        assert!(!quads.is_empty());


        for (qp0, qp1, qp2) in &quads {
            assert!(qp0.0 <= qp2.0);
        }
    }

    #[test]
    fn test_approximate_with_lines() {
        let p0 = (0.0, 0.0);
        let p1 = (50.0, 0.0);
        let p2 = (100.0, 50.0);
        let p3 = (100.0, 100.0);

        let points = BezierUtils::approximate_with_lines(p0, p1, p2, p3, 1.0, 10);
        assert!(!points.is_empty());
        assert!(points.len() <= 10);


        assert_eq!(points[0], p3);
    }
}
