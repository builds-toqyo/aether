

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub tx: f64,
    pub ty: f64,
    pub sx: f64,
    pub sy: f64,
    pub rotation: f64,
    pub kx: f64,
    pub ky: f64,
}

impl Transform {

    pub fn identity() -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            sx: 1.0,
            sy: 1.0,
            rotation: 0.0,
            kx: 0.0,
            ky: 0.0,
        }
    }


    pub fn translation(tx: f64, ty: f64) -> Self {
        Self {
            tx,
            ty,
            sx: 1.0,
            sy: 1.0,
            rotation: 0.0,
            kx: 0.0,
            ky: 0.0,
        }
    }


    pub fn scale(sx: f64, sy: f64) -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            sx,
            sy,
            rotation: 0.0,
            kx: 0.0,
            ky: 0.0,
        }
    }


    pub fn rotation(angle: f64) -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            sx: 1.0,
            sy: 1.0,
            rotation: angle,
            kx: 0.0,
            ky: 0.0,
        }
    }


    pub fn transform_point(&self, x: f64, y: f64) -> (f64, f64) {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();


        let x_rot = x * self.sx * cos_r - y * self.sy * sin_r;
        let y_rot = x * self.sx * sin_r + y * self.sy * cos_r;


        let x_skew = x_rot + y_rot * self.kx;
        let y_skew = y_rot + x_rot * self.ky;


        (x_skew + self.tx, y_skew + self.ty)
    }


    pub fn inverse(&self) -> Option<Self> {
        if self.sx == 0.0 || self.sy == 0.0 {
            return None;
        }

        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();


        let inv_sx = 1.0 / self.sx;
        let inv_sy = 1.0 / self.sy;


        let inv_cos = cos_r;
        let inv_sin = -sin_r;


        let inv_tx = -(self.tx * inv_cos - self.ty * inv_sin) * inv_sx;
        let inv_ty = -(self.tx * inv_sin + self.ty * inv_cos) * inv_sy;

        Some(Self {
            tx: inv_tx,
            ty: inv_ty,
            sx: inv_sx,
            sy: inv_sy,
            rotation: -self.rotation,
            kx: -self.kx,
            ky: -self.ky,
        })
    }


    pub fn combine(&self, other: &Transform) -> Self {
        let cos_r1 = self.rotation.cos();
        let sin_r1 = self.rotation.sin();
        let cos_r2 = other.rotation.cos();
        let sin_r2 = other.rotation.sin();


        let sx = self.sx * other.sx;
        let sy = self.sy * other.sy;
        let rotation = self.rotation + other.rotation;


        let tx = self.tx + other.tx * self.sx * cos_r1 - other.ty * self.sy * sin_r1;
        let ty = self.ty + other.tx * self.sx * sin_r1 + other.ty * self.sy * cos_r1;


        let kx = self.kx + other.kx;
        let ky = self.ky + other.ky;

        Self {
            tx,
            ty,
            sx,
            sy,
            rotation,
            kx,
            ky,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {

    pub min_x: f64,

    pub min_y: f64,

    pub max_x: f64,

    pub max_y: f64,
}

impl BoundingBox {

    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            max_x: max_x.max(min_x),
            max_y: max_y.max(min_y),
        }
    }


    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }


    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }


    pub fn center(&self) -> (f64, f64) {
        ((self.min_x + self.max_x) / 2.0, (self.min_y + self.max_y) / 2.0)
    }


    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }


    pub fn intersects(&self, other: &BoundingBox) -> bool {
        !(self.max_x < other.min_x || self.min_x > other.max_x ||
          self.max_y < other.min_y || self.min_y > other.max_y)
    }


    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
        )
    }


    pub fn transform(&self, transform: &Transform) -> BoundingBox {
        let corners = [
            transform.transform_point(self.min_x, self.min_y),
            transform.transform_point(self.max_x, self.min_y),
            transform.transform_point(self.max_x, self.max_y),
            transform.transform_point(self.min_x, self.max_y),
        ];

        let min_x = corners.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
        let max_x = corners.iter().map(|(x, _)| *x).fold(f64::NEG_INFINITY, f64::max);
        let min_y = corners.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
        let max_y = corners.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, f64::max);

        BoundingBox::new(min_x, min_y, max_x, max_y)
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_identity() {
        let transform = Transform::identity();
        let point = transform.transform_point(10.0, 20.0);
        assert_eq!(point, (10.0, 20.0));
    }

    #[test]
    fn test_transform_translation() {
        let transform = Transform::translation(5.0, 10.0);
        let point = transform.transform_point(10.0, 20.0);
        assert_eq!(point, (15.0, 30.0));
    }

    #[test]
    fn test_transform_scale() {
        let transform = Transform::scale(2.0, 3.0);
        let point = transform.transform_point(10.0, 20.0);
        assert_eq!(point, (20.0, 60.0));
    }

    #[test]
    fn test_transform_rotation() {
        let transform = Transform::rotation(std::f64::consts::PI / 2.0);
        let point = transform.transform_point(10.0, 0.0);
        assert!((point.0 - 0.0).abs() < f64::EPSILON);
        assert!((point.1 - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transform_combine() {
        let t1 = Transform::translation(5.0, 10.0);
        let t2 = Transform::scale(2.0, 3.0);
        let combined = t1.combine(&t2);

        let point = combined.transform_point(10.0, 20.0);
        assert_eq!(point, (25.0, 70.0));
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(10.0, 20.0, 100.0, 80.0);

        assert_eq!(bbox.width(), 90.0);
        assert_eq!(bbox.height(), 60.0);
        assert_eq!(bbox.center(), (55.0, 50.0));
        assert!(bbox.contains_point(50.0, 50.0));
        assert!(!bbox.contains_point(5.0, 50.0));

        let other = BoundingBox::new(50.0, 40.0, 150.0, 90.0);
        assert!(bbox.intersects(&other));

        let union = bbox.union(&other);
        assert_eq!(union.min_x, 10.0);
        assert_eq!(union.max_x, 150.0);
    }

    #[test]
    fn test_bounding_box_transform() {
        let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        let transform = Transform::translation(5.0, 10.0);

        let transformed = bbox.transform(&transform);
        assert_eq!(transformed.min_x, 5.0);
        assert_eq!(transformed.min_y, 10.0);
        assert_eq!(transformed.max_x, 15.0);
        assert_eq!(transformed.max_y, 20.0);
    }
}
