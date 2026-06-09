#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use std::collections::VecDeque;


    #[derive(Debug, Clone, Default)]
    pub struct FrameStats {
        pub frame_times: VecDeque<Duration>,
        pub max_samples: usize,
        pub dropped_frames: u64,
        pub total_frames: u64,
    }

    impl FrameStats {
        pub fn new(max_samples: usize) -> Self {
            Self {
                frame_times: VecDeque::with_capacity(max_samples),
                max_samples,
                dropped_frames: 0,
                total_frames: 0,
            }
        }

        pub fn record_frame(&mut self, time: Duration, target_fps: f64) {
            self.total_frames += 1;

            if self.frame_times.len() >= self.max_samples {
                self.frame_times.pop_front();
            }
            self.frame_times.push_back(time);

            let target_time = Duration::from_secs_f64(1.0 / target_fps);
            if time > target_time {
                self.dropped_frames += 1;
            }
        }

        pub fn average_fps(&self) -> f64 {
            if self.frame_times.is_empty() {
                return 0.0;
            }
            let avg: Duration = self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32;
            1.0 / avg.as_secs_f64()
        }

        pub fn min_fps(&self) -> f64 {
            self.frame_times.iter()
                .max()
                .map(|t| 1.0 / t.as_secs_f64())
                .unwrap_or(0.0)
        }

        pub fn max_fps(&self) -> f64 {
            self.frame_times.iter()
                .min()
                .map(|t| 1.0 / t.as_secs_f64())
                .unwrap_or(0.0)
        }

        pub fn percentile(&self, p: f64) -> Duration {
            if self.frame_times.is_empty() {
                return Duration::ZERO;
            }
            let mut sorted: Vec<_> = self.frame_times.iter().copied().collect();
            sorted.sort();
            let index = ((sorted.len() as f64 * p / 100.0) as usize).min(sorted.len() - 1);
            sorted[index]
        }

        pub fn drop_rate(&self) -> f64 {
            if self.total_frames == 0 {
                return 0.0;
            }
            self.dropped_frames as f64 / self.total_frames as f64 * 100.0
        }
    }


    #[derive(Debug, Clone)]
    pub struct PreviewConfig {
        pub resolution: (u32, u32),
        pub target_fps: f64,
        pub quality: PreviewQuality,
        pub proxy_enabled: bool,
        pub effects_enabled: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum PreviewQuality {
        Draft,
        Quarter,
        Half,
        Full,
    }

    impl PreviewQuality {
        pub fn scale_factor(&self) -> f64 {
            match self {
                PreviewQuality::Draft => 0.125,
                PreviewQuality::Quarter => 0.25,
                PreviewQuality::Half => 0.5,
                PreviewQuality::Full => 1.0,
            }
        }
    }

    impl Default for PreviewConfig {
        fn default() -> Self {
            Self {
                resolution: (1920, 1080),
                target_fps: 30.0,
                quality: PreviewQuality::Full,
                proxy_enabled: false,
                effects_enabled: true,
            }
        }
    }


    pub struct MockPreviewRenderer {
        config: PreviewConfig,
        frame_stats: FrameStats,
        current_frame: u64,
    }

    impl MockPreviewRenderer {
        pub fn new(config: PreviewConfig) -> Self {
            Self {
                config,
                frame_stats: FrameStats::new(300),
                current_frame: 0,
            }
        }

        pub fn render_frame(&mut self) -> Duration {
            let start = Instant::now();


            let base_time_us = self.calculate_base_render_time();


            let variance = (self.current_frame % 10) as u64 * 100;
            let render_time = Duration::from_micros(base_time_us + variance);

            std::thread::sleep(render_time.min(Duration::from_millis(50)));

            let elapsed = start.elapsed();
            self.frame_stats.record_frame(elapsed, self.config.target_fps);
            self.current_frame += 1;

            elapsed
        }

        fn calculate_base_render_time(&self) -> u64 {
            let (width, height) = self.config.resolution;
            let scale = self.config.quality.scale_factor();
            let effective_pixels = (width as f64 * height as f64 * scale * scale) as u64;


            let base_per_mpixel = if self.config.proxy_enabled { 500 } else { 1000 };
            let effect_multiplier = if self.config.effects_enabled { 1.5 } else { 1.0 };

            ((effective_pixels / 1_000_000).max(1) * base_per_mpixel) as f64 * effect_multiplier as f64
        }

        pub fn get_stats(&self) -> &FrameStats {
            &self.frame_stats
        }

        pub fn reset_stats(&mut self) {
            self.frame_stats = FrameStats::new(300);
        }

        pub fn meets_target_fps(&self) -> bool {
            self.frame_stats.average_fps() >= self.config.target_fps * 0.95
        }
    }


    pub struct PreviewBenchmark {
        pub name: String,
        pub config: PreviewConfig,
        pub duration_frames: u64,
    }

    impl PreviewBenchmark {
        pub fn new(name: &str, config: PreviewConfig, duration_frames: u64) -> Self {
            Self {
                name: name.to_string(),
                config,
                duration_frames,
            }
        }

        pub fn run(&self) -> BenchmarkResult {
            let mut renderer = MockPreviewRenderer::new(self.config.clone());

            let start = Instant::now();
            for _ in 0..self.duration_frames {
                renderer.render_frame();
            }
            let total_time = start.elapsed();

            let stats = renderer.get_stats();
            BenchmarkResult {
                name: self.name.clone(),
                total_time,
                frames_rendered: self.duration_frames,
                average_fps: stats.average_fps(),
                min_fps: stats.min_fps(),
                max_fps: stats.max_fps(),
                p99_frame_time: stats.percentile(99.0),
                dropped_frames: stats.dropped_frames,
                drop_rate: stats.drop_rate(),
                meets_target: renderer.meets_target_fps(),
            }
        }
    }

    #[derive(Debug)]
    pub struct BenchmarkResult {
        pub name: String,
        pub total_time: Duration,
        pub frames_rendered: u64,
        pub average_fps: f64,
        pub min_fps: f64,
        pub max_fps: f64,
        pub p99_frame_time: Duration,
        pub dropped_frames: u64,
        pub drop_rate: f64,
        pub meets_target: bool,
    }

    impl BenchmarkResult {
        pub fn print(&self) {
            println!("=== {} ===", self.name);
            println!("  Total time: {:?}", self.total_time);
            println!("  Frames: {}", self.frames_rendered);
            println!("  Average FPS: {:.1}", self.average_fps);
            println!("  FPS range: {:.1} - {:.1}", self.min_fps, self.max_fps);
            println!("  P99 frame time: {:?}", self.p99_frame_time);
            println!("  Dropped frames: {} ({:.1}%)", self.dropped_frames, self.drop_rate);
            println!("  Meets target: {}", self.meets_target);
        }
    }

    #[test]
    fn test_frame_stats_basic() {
        let mut stats = FrameStats::new(100);

        stats.record_frame(Duration::from_millis(16), 60.0);
        stats.record_frame(Duration::from_millis(17), 60.0);
        stats.record_frame(Duration::from_millis(15), 60.0);

        assert_eq!(stats.total_frames, 3);
        assert!(stats.average_fps() > 55.0);
    }

    #[test]
    fn test_frame_stats_dropped_frames() {
        let mut stats = FrameStats::new(100);


        stats.record_frame(Duration::from_millis(10), 60.0);
        stats.record_frame(Duration::from_millis(20), 60.0);
        stats.record_frame(Duration::from_millis(30), 60.0);

        assert_eq!(stats.dropped_frames, 2);
        assert!((stats.drop_rate() - 66.67).abs() < 1.0);
    }

    #[test]
    fn test_frame_stats_percentile() {
        let mut stats = FrameStats::new(100);

        for i in 1..=100 {
            stats.record_frame(Duration::from_millis(i), 60.0);
        }

        let p50 = stats.percentile(50.0);
        let p99 = stats.percentile(99.0);

        assert!(p50 >= Duration::from_millis(50));
        assert!(p99 >= Duration::from_millis(99));
    }

    #[test]
    fn test_preview_quality_scaling() {
        assert_eq!(PreviewQuality::Full.scale_factor(), 1.0);
        assert_eq!(PreviewQuality::Half.scale_factor(), 0.5);
        assert_eq!(PreviewQuality::Quarter.scale_factor(), 0.25);
    }

    #[test]
    fn test_preview_renderer_basic() {
        let config = PreviewConfig {
            resolution: (1920, 1080),
            target_fps: 30.0,
            quality: PreviewQuality::Half,
            proxy_enabled: false,
            effects_enabled: false,
        };

        let mut renderer = MockPreviewRenderer::new(config);

        for _ in 0..10 {
            renderer.render_frame();
        }

        let stats = renderer.get_stats();
        assert_eq!(stats.total_frames, 10);
        assert!(stats.average_fps() > 0.0);
    }

    #[test]
    fn test_benchmark_1080p_30fps() {
        let config = PreviewConfig {
            resolution: (1920, 1080),
            target_fps: 30.0,
            quality: PreviewQuality::Full,
            proxy_enabled: false,
            effects_enabled: true,
        };

        let benchmark = PreviewBenchmark::new("1080p @ 30fps", config, 30);
        let result = benchmark.run();

        assert!(result.frames_rendered == 30);
        assert!(result.average_fps > 0.0);
    }

    #[test]
    fn test_benchmark_4k_30fps() {
        let config = PreviewConfig {
            resolution: (3840, 2160),
            target_fps: 30.0,
            quality: PreviewQuality::Half,
            proxy_enabled: true,
            effects_enabled: true,
        };

        let benchmark = PreviewBenchmark::new("4K @ 30fps (half quality)", config, 30);
        let result = benchmark.run();

        assert!(result.frames_rendered == 30);
    }

    #[test]
    fn test_proxy_performance_improvement() {
        let base_config = PreviewConfig {
            resolution: (3840, 2160),
            target_fps: 30.0,
            quality: PreviewQuality::Full,
            proxy_enabled: false,
            effects_enabled: true,
        };

        let proxy_config = PreviewConfig {
            proxy_enabled: true,
            ..base_config.clone()
        };

        let base_benchmark = PreviewBenchmark::new("4K no proxy", base_config, 10);
        let proxy_benchmark = PreviewBenchmark::new("4K with proxy", proxy_config, 10);

        let base_result = base_benchmark.run();
        let proxy_result = proxy_benchmark.run();


        assert!(proxy_result.average_fps >= base_result.average_fps);
    }

    #[test]
    fn test_quality_scaling_performance() {
        let qualities = vec![
            PreviewQuality::Draft,
            PreviewQuality::Quarter,
            PreviewQuality::Half,
            PreviewQuality::Full,
        ];

        let mut fps_values = Vec::new();

        for quality in qualities {
            let config = PreviewConfig {
                resolution: (1920, 1080),
                target_fps: 60.0,
                quality,
                proxy_enabled: false,
                effects_enabled: true,
            };

            let benchmark = PreviewBenchmark::new(&format!("{:?}", quality), config, 10);
            let result = benchmark.run();
            fps_values.push(result.average_fps);
        }


        assert!(fps_values[0] >= fps_values[3] * 0.8);
    }

    #[test]
    fn test_effects_performance_impact() {
        let no_effects = PreviewConfig {
            resolution: (1920, 1080),
            target_fps: 30.0,
            quality: PreviewQuality::Full,
            proxy_enabled: false,
            effects_enabled: false,
        };

        let with_effects = PreviewConfig {
            effects_enabled: true,
            ..no_effects.clone()
        };

        let no_fx_benchmark = PreviewBenchmark::new("No effects", no_effects, 10);
        let fx_benchmark = PreviewBenchmark::new("With effects", with_effects, 10);

        let no_fx_result = no_fx_benchmark.run();
        let fx_result = fx_benchmark.run();


        assert!(no_fx_result.average_fps >= fx_result.average_fps * 0.9);
    }
}
