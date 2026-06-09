#[cfg(test)]
mod tests {

    #[derive(Debug, Clone)]
    pub struct LUT1D {
        pub size: usize,
        pub data: Vec<f32>,
    }

    impl LUT1D {
        pub fn new(size: usize) -> Self {
            Self {
                size,
                data: vec![0.0; size],
            }
        }

        pub fn identity(size: usize) -> Self {
            let mut lut = Self::new(size);
            for i in 0..size {
                lut.data[i] = i as f32 / (size - 1) as f32;
            }
            lut
        }

        pub fn invert(size: usize) -> Self {
            let mut lut = Self::new(size);
            for i in 0..size {
                lut.data[i] = 1.0 - (i as f32 / (size - 1) as f32);
            }
            lut
        }

        pub fn gamma(size: usize, gamma: f32) -> Self {
            let mut lut = Self::new(size);
            for i in 0..size {
                let x = i as f32 / (size - 1) as f32;
                lut.data[i] = x.powf(gamma);
            }
            lut
        }

        pub fn apply(&self, value: f32) -> f32 {
            let clamped = value.clamp(0.0, 1.0);
            let index = clamped * (self.size - 1) as f32;
            let lower = index.floor() as usize;
            let upper = (lower + 1).min(self.size - 1);
            let t = index - lower as f32;

            self.data[lower] * (1.0 - t) + self.data[upper] * t
        }
    }


    #[derive(Debug, Clone)]
    pub struct LUT3D {
        pub size: usize,
        pub data: Vec<[f32; 3]>,
    }

    impl LUT3D {
        pub fn new(size: usize) -> Self {
            Self {
                size,
                data: vec![[0.0; 3]; size * size * size],
            }
        }

        pub fn identity(size: usize) -> Self {
            let mut lut = Self::new(size);
            for r in 0..size {
                for g in 0..size {
                    for b in 0..size {
                        let index = r * size * size + g * size + b;
                        lut.data[index] = [
                            r as f32 / (size - 1) as f32,
                            g as f32 / (size - 1) as f32,
                            b as f32 / (size - 1) as f32,
                        ];
                    }
                }
            }
            lut
        }

        pub fn desaturate(size: usize, amount: f32) -> Self {
            let mut lut = Self::new(size);
            for r in 0..size {
                for g in 0..size {
                    for b in 0..size {
                        let index = r * size * size + g * size + b;
                        let rf = r as f32 / (size - 1) as f32;
                        let gf = g as f32 / (size - 1) as f32;
                        let bf = b as f32 / (size - 1) as f32;

                        let luma = 0.299 * rf + 0.587 * gf + 0.114 * bf;
                        lut.data[index] = [
                            rf * (1.0 - amount) + luma * amount,
                            gf * (1.0 - amount) + luma * amount,
                            bf * (1.0 - amount) + luma * amount,
                        ];
                    }
                }
            }
            lut
        }

        fn get_index(&self, r: usize, g: usize, b: usize) -> usize {
            r * self.size * self.size + g * self.size + b
        }

        pub fn apply(&self, rgb: [f32; 3]) -> [f32; 3] {
            let r = (rgb[0].clamp(0.0, 1.0) * (self.size - 1) as f32);
            let g = (rgb[1].clamp(0.0, 1.0) * (self.size - 1) as f32);
            let b = (rgb[2].clamp(0.0, 1.0) * (self.size - 1) as f32);

            let r0 = r.floor() as usize;
            let g0 = g.floor() as usize;
            let b0 = b.floor() as usize;
            let r1 = (r0 + 1).min(self.size - 1);
            let g1 = (g0 + 1).min(self.size - 1);
            let b1 = (b0 + 1).min(self.size - 1);

            let dr = r - r0 as f32;
            let dg = g - g0 as f32;
            let db = b - b0 as f32;


            let c000 = self.data[self.get_index(r0, g0, b0)];
            let c001 = self.data[self.get_index(r0, g0, b1)];
            let c010 = self.data[self.get_index(r0, g1, b0)];
            let c011 = self.data[self.get_index(r0, g1, b1)];
            let c100 = self.data[self.get_index(r1, g0, b0)];
            let c101 = self.data[self.get_index(r1, g0, b1)];
            let c110 = self.data[self.get_index(r1, g1, b0)];
            let c111 = self.data[self.get_index(r1, g1, b1)];

            let mut result = [0.0f32; 3];
            for i in 0..3 {
                let c00 = c000[i] * (1.0 - dr) + c100[i] * dr;
                let c01 = c001[i] * (1.0 - dr) + c101[i] * dr;
                let c10 = c010[i] * (1.0 - dr) + c110[i] * dr;
                let c11 = c011[i] * (1.0 - dr) + c111[i] * dr;

                let c0 = c00 * (1.0 - dg) + c10 * dg;
                let c1 = c01 * (1.0 - dg) + c11 * dg;

                result[i] = c0 * (1.0 - db) + c1 * db;
            }

            result
        }
    }

    const EPSILON: f32 = 0.001;

    #[test]
    fn test_1d_lut_identity() {
        let lut = LUT1D::identity(256);

        assert!((lut.apply(0.0) - 0.0).abs() < EPSILON);
        assert!((lut.apply(0.5) - 0.5).abs() < EPSILON);
        assert!((lut.apply(1.0) - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_1d_lut_invert() {
        let lut = LUT1D::invert(256);

        assert!((lut.apply(0.0) - 1.0).abs() < EPSILON);
        assert!((lut.apply(0.5) - 0.5).abs() < EPSILON);
        assert!((lut.apply(1.0) - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_1d_lut_gamma() {
        let lut = LUT1D::gamma(256, 2.2);


        assert!((lut.apply(0.5) - 0.217).abs() < 0.01);
        assert!((lut.apply(0.0) - 0.0).abs() < EPSILON);
        assert!((lut.apply(1.0) - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_1d_lut_interpolation() {
        let lut = LUT1D::identity(17);


        assert!((lut.apply(0.25) - 0.25).abs() < EPSILON);
        assert!((lut.apply(0.75) - 0.75).abs() < EPSILON);
    }

    #[test]
    fn test_1d_lut_clamping() {
        let lut = LUT1D::identity(256);


        assert!((lut.apply(-0.5) - 0.0).abs() < EPSILON);
        assert!((lut.apply(1.5) - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_3d_lut_identity() {
        let lut = LUT3D::identity(17);

        let test_colors = vec![
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.5, 0.5, 0.5],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];

        for color in test_colors {
            let result = lut.apply(color);
            assert!((result[0] - color[0]).abs() < EPSILON, "R mismatch for {:?}", color);
            assert!((result[1] - color[1]).abs() < EPSILON, "G mismatch for {:?}", color);
            assert!((result[2] - color[2]).abs() < EPSILON, "B mismatch for {:?}", color);
        }
    }

    #[test]
    fn test_3d_lut_desaturate() {
        let lut = LUT3D::desaturate(17, 1.0);


        let red = lut.apply([1.0, 0.0, 0.0]);
        let luma = 0.299;
        assert!((red[0] - luma).abs() < 0.02);
        assert!((red[1] - luma).abs() < 0.02);
        assert!((red[2] - luma).abs() < 0.02);
    }

    #[test]
    fn test_3d_lut_partial_desaturate() {
        let lut = LUT3D::desaturate(17, 0.5);


        let red = lut.apply([1.0, 0.0, 0.0]);

        assert!(red[0] > 0.5);
        assert!(red[1] > 0.0 && red[1] < 0.5);
        assert!(red[2] > 0.0 && red[2] < 0.5);
    }

    #[test]
    fn test_3d_lut_interpolation() {
        let lut = LUT3D::identity(9);


        let result = lut.apply([0.25, 0.5, 0.75]);
        assert!((result[0] - 0.25).abs() < 0.02);
        assert!((result[1] - 0.5).abs() < 0.02);
        assert!((result[2] - 0.75).abs() < 0.02);
    }

    #[test]
    fn test_3d_lut_clamping() {
        let lut = LUT3D::identity(17);


        let result = lut.apply([-0.5, 1.5, 0.5]);
        assert!((result[0] - 0.0).abs() < EPSILON);
        assert!((result[1] - 1.0).abs() < EPSILON);
        assert!((result[2] - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_lut_chain() {

        let gamma_up = LUT1D::gamma(256, 2.2);
        let gamma_down = LUT1D::gamma(256, 1.0 / 2.2);

        let value = 0.5;
        let result = gamma_down.apply(gamma_up.apply(value));


        assert!((result - value).abs() < 0.02);
    }

    #[test]
    fn test_lut_size_variations() {

        let sizes = vec![9, 17, 33, 65];

        for size in sizes {
            let lut = LUT3D::identity(size);
            let result = lut.apply([0.5, 0.5, 0.5]);
            assert!((result[0] - 0.5).abs() < 0.02, "Failed for size {}", size);
            assert!((result[1] - 0.5).abs() < 0.02, "Failed for size {}", size);
            assert!((result[2] - 0.5).abs() < 0.02, "Failed for size {}", size);
        }
    }
}
