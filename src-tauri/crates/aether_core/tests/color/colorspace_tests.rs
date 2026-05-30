#[cfg(test)]
mod tests {

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct RGB {
        pub r: f32,
        pub g: f32,
        pub b: f32,
    }

    impl RGB {
        pub fn new(r: f32, g: f32, b: f32) -> Self {
            Self { r, g, b }
        }

        pub fn black() -> Self {
            Self::new(0.0, 0.0, 0.0)
        }

        pub fn white() -> Self {
            Self::new(1.0, 1.0, 1.0)
        }

        pub fn approx_eq(&self, other: &RGB, epsilon: f32) -> bool {
            (self.r - other.r).abs() < epsilon &&
            (self.g - other.g).abs() < epsilon &&
            (self.b - other.b).abs() < epsilon
        }
    }


    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct HSL {
        pub h: f32,
        pub s: f32,
        pub l: f32,
    }

    impl HSL {
        pub fn new(h: f32, s: f32, l: f32) -> Self {
            Self { h, s, l }
        }

        pub fn approx_eq(&self, other: &HSL, epsilon: f32) -> bool {
            (self.h - other.h).abs() < epsilon &&
            (self.s - other.s).abs() < epsilon &&
            (self.l - other.l).abs() < epsilon
        }
    }


    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct HSV {
        pub h: f32,
        pub s: f32,
        pub v: f32,
    }

    impl HSV {
        pub fn new(h: f32, s: f32, v: f32) -> Self {
            Self { h, s, v }
        }

        pub fn approx_eq(&self, other: &HSV, epsilon: f32) -> bool {
            (self.h - other.h).abs() < epsilon &&
            (self.s - other.s).abs() < epsilon &&
            (self.v - other.v).abs() < epsilon
        }
    }


    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct YUV {
        pub y: f32,
        pub u: f32,
        pub v: f32,
    }

    impl YUV {
        pub fn new(y: f32, u: f32, v: f32) -> Self {
            Self { y, u, v }
        }

        pub fn approx_eq(&self, other: &YUV, epsilon: f32) -> bool {
            (self.y - other.y).abs() < epsilon &&
            (self.u - other.u).abs() < epsilon &&
            (self.v - other.v).abs() < epsilon
        }
    }


    pub fn rgb_to_hsl(rgb: &RGB) -> HSL {
        let max = rgb.r.max(rgb.g).max(rgb.b);
        let min = rgb.r.min(rgb.g).min(rgb.b);
        let l = (max + min) / 2.0;

        if (max - min).abs() < f32::EPSILON {
            return HSL::new(0.0, 0.0, l);
        }

        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };

        let h = if (max - rgb.r).abs() < f32::EPSILON {
            let mut h = (rgb.g - rgb.b) / d;
            if rgb.g < rgb.b {
                h += 6.0;
            }
            h
        } else if (max - rgb.g).abs() < f32::EPSILON {
            (rgb.b - rgb.r) / d + 2.0
        } else {
            (rgb.r - rgb.g) / d + 4.0
        };

        HSL::new(h * 60.0, s, l)
    }


    pub fn hsl_to_rgb(hsl: &HSL) -> RGB {
        if hsl.s.abs() < f32::EPSILON {
            return RGB::new(hsl.l, hsl.l, hsl.l);
        }

        let q = if hsl.l < 0.5 {
            hsl.l * (1.0 + hsl.s)
        } else {
            hsl.l + hsl.s - hsl.l * hsl.s
        };
        let p = 2.0 * hsl.l - q;
        let h = hsl.h / 360.0;

        fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
            if t < 0.0 { t += 1.0; }
            if t > 1.0 { t -= 1.0; }
            if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
            if t < 1.0 / 2.0 { return q; }
            if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
            p
        }

        RGB::new(
            hue_to_rgb(p, q, h + 1.0 / 3.0),
            hue_to_rgb(p, q, h),
            hue_to_rgb(p, q, h - 1.0 / 3.0),
        )
    }


    pub fn rgb_to_hsv(rgb: &RGB) -> HSV {
        let max = rgb.r.max(rgb.g).max(rgb.b);
        let min = rgb.r.min(rgb.g).min(rgb.b);
        let v = max;

        if (max - min).abs() < f32::EPSILON {
            return HSV::new(0.0, 0.0, v);
        }

        let d = max - min;
        let s = d / max;

        let h = if (max - rgb.r).abs() < f32::EPSILON {
            let mut h = (rgb.g - rgb.b) / d;
            if rgb.g < rgb.b {
                h += 6.0;
            }
            h
        } else if (max - rgb.g).abs() < f32::EPSILON {
            (rgb.b - rgb.r) / d + 2.0
        } else {
            (rgb.r - rgb.g) / d + 4.0
        };

        HSV::new(h * 60.0, s, v)
    }


    pub fn hsv_to_rgb(hsv: &HSV) -> RGB {
        if hsv.s.abs() < f32::EPSILON {
            return RGB::new(hsv.v, hsv.v, hsv.v);
        }

        let h = hsv.h / 60.0;
        let i = h.floor() as i32;
        let f = h - i as f32;
        let p = hsv.v * (1.0 - hsv.s);
        let q = hsv.v * (1.0 - hsv.s * f);
        let t = hsv.v * (1.0 - hsv.s * (1.0 - f));

        match i % 6 {
            0 => RGB::new(hsv.v, t, p),
            1 => RGB::new(q, hsv.v, p),
            2 => RGB::new(p, hsv.v, t),
            3 => RGB::new(p, q, hsv.v),
            4 => RGB::new(t, p, hsv.v),
            _ => RGB::new(hsv.v, p, q),
        }
    }


    pub fn rgb_to_yuv(rgb: &RGB) -> YUV {
        let y = 0.299 * rgb.r + 0.587 * rgb.g + 0.114 * rgb.b;
        let u = -0.14713 * rgb.r - 0.28886 * rgb.g + 0.436 * rgb.b;
        let v = 0.615 * rgb.r - 0.51499 * rgb.g - 0.10001 * rgb.b;
        YUV::new(y, u, v)
    }


    pub fn yuv_to_rgb(yuv: &YUV) -> RGB {
        let r = yuv.y + 1.13983 * yuv.v;
        let g = yuv.y - 0.39465 * yuv.u - 0.58060 * yuv.v;
        let b = yuv.y + 2.03211 * yuv.u;
        RGB::new(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    }


    pub fn apply_gamma(rgb: &RGB, gamma: f32) -> RGB {
        RGB::new(
            rgb.r.powf(gamma),
            rgb.g.powf(gamma),
            rgb.b.powf(gamma),
        )
    }


    pub fn linear_to_srgb(rgb: &RGB) -> RGB {
        fn convert(c: f32) -> f32 {
            if c <= 0.0031308 {
                12.92 * c
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        }
        RGB::new(convert(rgb.r), convert(rgb.g), convert(rgb.b))
    }


    pub fn srgb_to_linear(rgb: &RGB) -> RGB {
        fn convert(c: f32) -> f32 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        RGB::new(convert(rgb.r), convert(rgb.g), convert(rgb.b))
    }

    const EPSILON: f32 = 0.01;

    #[test]
    fn test_rgb_to_hsl_black() {
        let rgb = RGB::black();
        let hsl = rgb_to_hsl(&rgb);
        assert!(hsl.approx_eq(&HSL::new(0.0, 0.0, 0.0), EPSILON));
    }

    #[test]
    fn test_rgb_to_hsl_white() {
        let rgb = RGB::white();
        let hsl = rgb_to_hsl(&rgb);
        assert!(hsl.approx_eq(&HSL::new(0.0, 0.0, 1.0), EPSILON));
    }

    #[test]
    fn test_rgb_to_hsl_red() {
        let rgb = RGB::new(1.0, 0.0, 0.0);
        let hsl = rgb_to_hsl(&rgb);
        assert!(hsl.approx_eq(&HSL::new(0.0, 1.0, 0.5), EPSILON));
    }

    #[test]
    fn test_rgb_to_hsl_green() {
        let rgb = RGB::new(0.0, 1.0, 0.0);
        let hsl = rgb_to_hsl(&rgb);
        assert!(hsl.approx_eq(&HSL::new(120.0, 1.0, 0.5), EPSILON));
    }

    #[test]
    fn test_rgb_to_hsl_blue() {
        let rgb = RGB::new(0.0, 0.0, 1.0);
        let hsl = rgb_to_hsl(&rgb);
        assert!(hsl.approx_eq(&HSL::new(240.0, 1.0, 0.5), EPSILON));
    }

    #[test]
    fn test_hsl_to_rgb_roundtrip() {
        let colors = vec![
            RGB::new(1.0, 0.0, 0.0),
            RGB::new(0.0, 1.0, 0.0),
            RGB::new(0.0, 0.0, 1.0),
            RGB::new(0.5, 0.5, 0.5),
            RGB::new(0.25, 0.75, 0.5),
        ];

        for rgb in colors {
            let hsl = rgb_to_hsl(&rgb);
            let back = hsl_to_rgb(&hsl);
            assert!(rgb.approx_eq(&back, EPSILON), "Failed for {:?}", rgb);
        }
    }

    #[test]
    fn test_rgb_to_hsv_roundtrip() {
        let colors = vec![
            RGB::new(1.0, 0.0, 0.0),
            RGB::new(0.0, 1.0, 0.0),
            RGB::new(0.0, 0.0, 1.0),
            RGB::new(0.5, 0.5, 0.5),
            RGB::new(0.25, 0.75, 0.5),
        ];

        for rgb in colors {
            let hsv = rgb_to_hsv(&rgb);
            let back = hsv_to_rgb(&hsv);
            assert!(rgb.approx_eq(&back, EPSILON), "Failed for {:?}", rgb);
        }
    }

    #[test]
    fn test_rgb_to_yuv_roundtrip() {
        let colors = vec![
            RGB::new(1.0, 0.0, 0.0),
            RGB::new(0.0, 1.0, 0.0),
            RGB::new(0.0, 0.0, 1.0),
            RGB::new(0.5, 0.5, 0.5),
        ];

        for rgb in colors {
            let yuv = rgb_to_yuv(&rgb);
            let back = yuv_to_rgb(&yuv);
            assert!(rgb.approx_eq(&back, EPSILON), "Failed for {:?}", rgb);
        }
    }

    #[test]
    fn test_gamma_correction() {
        let rgb = RGB::new(0.5, 0.5, 0.5);
        let gamma_22 = apply_gamma(&rgb, 2.2);
        let gamma_inv = apply_gamma(&gamma_22, 1.0 / 2.2);
        assert!(rgb.approx_eq(&gamma_inv, EPSILON));
    }

    #[test]
    fn test_srgb_linear_roundtrip() {
        let colors = vec![
            RGB::new(0.0, 0.0, 0.0),
            RGB::new(1.0, 1.0, 1.0),
            RGB::new(0.5, 0.5, 0.5),
            RGB::new(0.1, 0.5, 0.9),
        ];

        for rgb in colors {
            let linear = srgb_to_linear(&rgb);
            let back = linear_to_srgb(&linear);
            assert!(rgb.approx_eq(&back, EPSILON), "Failed for {:?}", rgb);
        }
    }

    #[test]
    fn test_srgb_linear_known_values() {

        let srgb = RGB::new(0.5, 0.5, 0.5);
        let linear = srgb_to_linear(&srgb);
        assert!((linear.r - 0.214).abs() < 0.01);
    }
}
