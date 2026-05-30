

#[cfg(test)]
mod tests {

    #[derive(Debug, Clone)]
    pub struct Histogram {
        pub bins: usize,
        pub r: Vec<u32>,
        pub g: Vec<u32>,
        pub b: Vec<u32>,
        pub luma: Vec<u32>,
    }

    impl Histogram {
        pub fn new(bins: usize) -> Self {
            Self {
                bins,
                r: vec![0; bins],
                g: vec![0; bins],
                b: vec![0; bins],
                luma: vec![0; bins],
            }
        }

        pub fn add_pixel(&mut self, r: f32, g: f32, b: f32) {
            let r_bin = ((r.clamp(0.0, 1.0) * (self.bins - 1) as f32) as usize).min(self.bins - 1);
            let g_bin = ((g.clamp(0.0, 1.0) * (self.bins - 1) as f32) as usize).min(self.bins - 1);
            let b_bin = ((b.clamp(0.0, 1.0) * (self.bins - 1) as f32) as usize).min(self.bins - 1);

            let luma = 0.299 * r + 0.587 * g + 0.114 * b;
            let luma_bin = ((luma.clamp(0.0, 1.0) * (self.bins - 1) as f32) as usize).min(self.bins - 1);

            self.r[r_bin] += 1;
            self.g[g_bin] += 1;
            self.b[b_bin] += 1;
            self.luma[luma_bin] += 1;
        }

        pub fn total_pixels(&self) -> u32 {
            self.luma.iter().sum()
        }

        pub fn max_value(&self) -> u32 {
            *self.r.iter().chain(self.g.iter()).chain(self.b.iter()).max().unwrap_or(&0)
        }

        pub fn normalize(&self) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
            let max = self.max_value() as f32;
            if max == 0.0 {
                return (vec![0.0; self.bins], vec![0.0; self.bins], vec![0.0; self.bins], vec![0.0; self.bins]);
            }
            (
                self.r.iter().map(|&v| v as f32 / max).collect(),
                self.g.iter().map(|&v| v as f32 / max).collect(),
                self.b.iter().map(|&v| v as f32 / max).collect(),
                self.luma.iter().map(|&v| v as f32 / max).collect(),
            )
        }
    }


    #[derive(Debug, Clone)]
    pub struct Waveform {
        pub width: usize,
        pub height: usize,
        pub data: Vec<Vec<u32>>,
    }

    impl Waveform {
        pub fn new(width: usize, height: usize) -> Self {
            Self {
                width,
                height,
                data: vec![vec![0; height]; width],
            }
        }

        pub fn add_sample(&mut self, x: usize, luma: f32) {
            if x >= self.width {
                return;
            }
            let y = ((1.0 - luma.clamp(0.0, 1.0)) * (self.height - 1) as f32) as usize;
            let y = y.min(self.height - 1);
            self.data[x][y] += 1;
        }

        pub fn max_intensity(&self) -> u32 {
            self.data.iter()
                .flat_map(|col| col.iter())
                .copied()
                .max()
                .unwrap_or(0)
        }

        pub fn normalize(&self) -> Vec<Vec<f32>> {
            let max = self.max_intensity() as f32;
            if max == 0.0 {
                return vec![vec![0.0; self.height]; self.width];
            }
            self.data.iter()
                .map(|col| col.iter().map(|&v| v as f32 / max).collect())
                .collect()
        }
    }


    #[derive(Debug, Clone)]
    pub struct Vectorscope {
        pub size: usize,
        pub data: Vec<Vec<u32>>,
    }

    impl Vectorscope {
        pub fn new(size: usize) -> Self {
            Self {
                size,
                data: vec![vec![0; size]; size],
            }
        }

        pub fn add_pixel(&mut self, r: f32, g: f32, b: f32) {

            let u = -0.14713 * r - 0.28886 * g + 0.436 * b;
            let v = 0.615 * r - 0.51499 * g - 0.10001 * b;


            let x = ((u + 0.5) * (self.size - 1) as f32).clamp(0.0, (self.size - 1) as f32) as usize;
            let y = ((0.5 - v) * (self.size - 1) as f32).clamp(0.0, (self.size - 1) as f32) as usize;

            self.data[x][y] += 1;
        }

        pub fn max_intensity(&self) -> u32 {
            self.data.iter()
                .flat_map(|col| col.iter())
                .copied()
                .max()
                .unwrap_or(0)
        }

        pub fn normalize(&self) -> Vec<Vec<f32>> {
            let max = self.max_intensity() as f32;
            if max == 0.0 {
                return vec![vec![0.0; self.size]; self.size];
            }
            self.data.iter()
                .map(|col| col.iter().map(|&v| v as f32 / max).collect())
                .collect()
        }

        pub fn get_skin_tone_line(&self) -> Vec<(usize, usize)> {

            let center = self.size / 2;
            let mut points = Vec::new();

            for i in 0..self.size {
                let angle = 123.0_f32.to_radians();
                let radius = i as f32 - center as f32;
                let x = center as f32 + radius * angle.cos();
                let y = center as f32 - radius * angle.sin();

                if x >= 0.0 && x < self.size as f32 && y >= 0.0 && y < self.size as f32 {
                    points.push((x as usize, y as usize));
                }
            }

            points
        }
    }


    #[derive(Debug, Clone)]
    pub struct RGBParade {
        pub width: usize,
        pub height: usize,
        pub r_data: Vec<Vec<u32>>,
        pub g_data: Vec<Vec<u32>>,
        pub b_data: Vec<Vec<u32>>,
    }

    impl RGBParade {
        pub fn new(width: usize, height: usize) -> Self {
            Self {
                width,
                height,
                r_data: vec![vec![0; height]; width],
                g_data: vec![vec![0; height]; width],
                b_data: vec![vec![0; height]; width],
            }
        }

        pub fn add_sample(&mut self, x: usize, r: f32, g: f32, b: f32) {
            if x >= self.width {
                return;
            }

            let r_y = ((1.0 - r.clamp(0.0, 1.0)) * (self.height - 1) as f32) as usize;
            let g_y = ((1.0 - g.clamp(0.0, 1.0)) * (self.height - 1) as f32) as usize;
            let b_y = ((1.0 - b.clamp(0.0, 1.0)) * (self.height - 1) as f32) as usize;

            self.r_data[x][r_y.min(self.height - 1)] += 1;
            self.g_data[x][g_y.min(self.height - 1)] += 1;
            self.b_data[x][b_y.min(self.height - 1)] += 1;
        }
    }

    #[test]
    fn test_histogram_empty() {
        let hist = Histogram::new(256);
        assert_eq!(hist.total_pixels(), 0);
        assert_eq!(hist.max_value(), 0);
    }

    #[test]
    fn test_histogram_single_pixel() {
        let mut hist = Histogram::new(256);
        hist.add_pixel(0.5, 0.5, 0.5);

        assert_eq!(hist.total_pixels(), 1);
        assert_eq!(hist.r[127], 1);
        assert_eq!(hist.g[127], 1);
        assert_eq!(hist.b[127], 1);
    }

    #[test]
    fn test_histogram_black_white() {
        let mut hist = Histogram::new(256);
        hist.add_pixel(0.0, 0.0, 0.0);
        hist.add_pixel(1.0, 1.0, 1.0);

        assert_eq!(hist.total_pixels(), 2);
        assert_eq!(hist.r[0], 1);
        assert_eq!(hist.r[255], 1);
        assert_eq!(hist.luma[0], 1);
        assert_eq!(hist.luma[255], 1);
    }

    #[test]
    fn test_histogram_primary_colors() {
        let mut hist = Histogram::new(256);
        hist.add_pixel(1.0, 0.0, 0.0);
        hist.add_pixel(0.0, 1.0, 0.0);
        hist.add_pixel(0.0, 0.0, 1.0);

        assert_eq!(hist.r[255], 1);
        assert_eq!(hist.r[0], 2);
        assert_eq!(hist.g[255], 1);
        assert_eq!(hist.g[0], 2);
        assert_eq!(hist.b[255], 1);
        assert_eq!(hist.b[0], 2);
    }

    #[test]
    fn test_histogram_normalization() {
        let mut hist = Histogram::new(256);
        for _ in 0..100 {
            hist.add_pixel(0.5, 0.5, 0.5);
        }

        let (r, g, b, luma) = hist.normalize();
        assert!((r[127] - 1.0).abs() < 0.01);
        assert!((g[127] - 1.0).abs() < 0.01);
        assert!((b[127] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_waveform_empty() {
        let wf = Waveform::new(100, 256);
        assert_eq!(wf.max_intensity(), 0);
    }

    #[test]
    fn test_waveform_single_sample() {
        let mut wf = Waveform::new(100, 256);
        wf.add_sample(50, 0.5);

        assert_eq!(wf.max_intensity(), 1);

        assert_eq!(wf.data[50][127], 1);
    }

    #[test]
    fn test_waveform_full_range() {
        let mut wf = Waveform::new(100, 256);
        wf.add_sample(0, 0.0);
        wf.add_sample(50, 0.5);
        wf.add_sample(99, 1.0);

        assert_eq!(wf.data[0][255], 1);
        assert_eq!(wf.data[50][127], 1);
        assert_eq!(wf.data[99][0], 1);
    }

    #[test]
    fn test_waveform_normalization() {
        let mut wf = Waveform::new(100, 256);
        for _ in 0..10 {
            wf.add_sample(50, 0.5);
        }

        let normalized = wf.normalize();
        assert!((normalized[50][127] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_vectorscope_empty() {
        let vs = Vectorscope::new(256);
        assert_eq!(vs.max_intensity(), 0);
    }

    #[test]
    fn test_vectorscope_neutral() {
        let mut vs = Vectorscope::new(256);
        vs.add_pixel(0.5, 0.5, 0.5);


        let center = 128;

        let found = vs.data[center - 1..=center + 1].iter()
            .flat_map(|col| col[center - 1..=center + 1].iter())
            .any(|&v| v > 0);
        assert!(found, "Neutral gray should be near center");
    }

    #[test]
    fn test_vectorscope_primary_colors() {
        let mut vs = Vectorscope::new(256);
        vs.add_pixel(1.0, 0.0, 0.0);
        vs.add_pixel(0.0, 1.0, 0.0);
        vs.add_pixel(0.0, 0.0, 1.0);


        assert!(vs.max_intensity() > 0);
    }

    #[test]
    fn test_vectorscope_skin_tone_line() {
        let vs = Vectorscope::new(256);
        let skin_line = vs.get_skin_tone_line();


        assert!(!skin_line.is_empty());


        for (x, y) in skin_line {
            assert!(x < vs.size);
            assert!(y < vs.size);
        }
    }

    #[test]
    fn test_rgb_parade_empty() {
        let parade = RGBParade::new(100, 256);
        assert_eq!(parade.r_data[0][0], 0);
    }

    #[test]
    fn test_rgb_parade_primary_colors() {
        let mut parade = RGBParade::new(100, 256);
        parade.add_sample(50, 1.0, 0.0, 0.0);


        assert_eq!(parade.r_data[50][0], 1);
        assert_eq!(parade.g_data[50][255], 1);
        assert_eq!(parade.b_data[50][255], 1);
    }

    #[test]
    fn test_histogram_clamping() {
        let mut hist = Histogram::new(256);
        hist.add_pixel(-0.5, 1.5, 0.5);


        assert_eq!(hist.r[0], 1);
        assert_eq!(hist.g[255], 1);
    }

    #[test]
    fn test_scope_large_dataset() {
        let mut hist = Histogram::new(256);
        let mut wf = Waveform::new(1920, 256);
        let mut vs = Vectorscope::new(256);


        for x in 0..1920 {
            for _ in 0..1080 {
                let r = (x as f32 / 1920.0);
                let g = 0.5;
                let b = 1.0 - r;

                hist.add_pixel(r, g, b);
                wf.add_sample(x, 0.299 * r + 0.587 * g + 0.114 * b);
                vs.add_pixel(r, g, b);
            }
        }

        assert_eq!(hist.total_pixels(), 1920 * 1080);
        assert!(wf.max_intensity() > 0);
        assert!(vs.max_intensity() > 0);
    }
}
