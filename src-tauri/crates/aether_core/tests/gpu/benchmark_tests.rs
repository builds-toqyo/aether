

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};


    #[derive(Debug, Clone)]
    pub struct BenchmarkResult {
        pub name: String,
        pub iterations: u32,
        pub total_time: Duration,
        pub min_time: Duration,
        pub max_time: Duration,
        pub avg_time: Duration,
        pub std_dev: Duration,
        pub throughput: Option<f64>,
    }

    impl BenchmarkResult {
        pub fn fps(&self) -> f64 {
            1.0 / self.avg_time.as_secs_f64()
        }

        pub fn ms_per_frame(&self) -> f64 {
            self.avg_time.as_secs_f64() * 1000.0
        }
    }


    pub struct Benchmarker {
        warmup_iterations: u32,
        benchmark_iterations: u32,
    }

    impl Benchmarker {
        pub fn new(warmup: u32, iterations: u32) -> Self {
            Self {
                warmup_iterations: warmup,
                benchmark_iterations: iterations,
            }
        }

        pub fn run<F>(&self, name: &str, mut f: F) -> BenchmarkResult
        where
            F: FnMut(),
        {

            for _ in 0..self.warmup_iterations {
                f();
            }


            let mut times = Vec::with_capacity(self.benchmark_iterations as usize);
            let start_total = Instant::now();

            for _ in 0..self.benchmark_iterations {
                let start = Instant::now();
                f();
                times.push(start.elapsed());
            }

            let total_time = start_total.elapsed();


            let min_time = *times.iter().min().unwrap();
            let max_time = *times.iter().max().unwrap();
            let sum: Duration = times.iter().sum();
            let avg_time = sum / self.benchmark_iterations;


            let variance: f64 = times.iter()
                .map(|t| {
                    let diff = t.as_secs_f64() - avg_time.as_secs_f64();
                    diff * diff
                })
                .sum::<f64>() / self.benchmark_iterations as f64;
            let std_dev = Duration::from_secs_f64(variance.sqrt());

            let throughput = Some(self.benchmark_iterations as f64 / total_time.as_secs_f64());

            BenchmarkResult {
                name: name.to_string(),
                iterations: self.benchmark_iterations,
                total_time,
                min_time,
                max_time,
                avg_time,
                std_dev,
                throughput,
            }
        }
    }


    pub struct MockGpuOperation {
        pub operation_type: OperationType,
        pub resolution: (u32, u32),
        pub complexity: f32,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum OperationType {
        ColorCorrection,
        Blur,
        Composite,
        Transform,
        Render,
    }

    impl MockGpuOperation {
        pub fn new(op_type: OperationType, width: u32, height: u32) -> Self {
            Self {
                operation_type: op_type,
                resolution: (width, height),
                complexity: 0.5,
            }
        }

        pub fn with_complexity(mut self, complexity: f32) -> Self {
            self.complexity = complexity.clamp(0.0, 1.0);
            self
        }

        pub fn execute(&self) {

            let pixels = self.resolution.0 as u64 * self.resolution.1 as u64;
            let base_ops = match self.operation_type {
                OperationType::ColorCorrection => 10,
                OperationType::Blur => 50,
                OperationType::Composite => 20,
                OperationType::Transform => 30,
                OperationType::Render => 100,
            };

            let total_ops = (pixels * base_ops as u64) as f64 * self.complexity as f64;


            let sleep_us = (total_ops / 1_000_000.0).max(1.0) as u64;
            std::thread::sleep(Duration::from_micros(sleep_us.min(1000)));
        }

        pub fn estimated_time_ms(&self) -> f64 {
            let pixels = self.resolution.0 as f64 * self.resolution.1 as f64;
            let base_time = match self.operation_type {
                OperationType::ColorCorrection => 0.5,
                OperationType::Blur => 2.0,
                OperationType::Composite => 1.0,
                OperationType::Transform => 1.5,
                OperationType::Render => 5.0,
            };
            base_time * (pixels / (1920.0 * 1080.0)) * self.complexity as f64
        }
    }


    #[derive(Debug, Default)]
    pub struct PerformanceMetrics {
        pub frame_times: Vec<Duration>,
        pub gpu_utilization: Vec<f32>,
        pub memory_usage: Vec<usize>,
    }

    impl PerformanceMetrics {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn record_frame(&mut self, time: Duration, gpu_util: f32, memory: usize) {
            self.frame_times.push(time);
            self.gpu_utilization.push(gpu_util);
            self.memory_usage.push(memory);
        }

        pub fn average_fps(&self) -> f64 {
            if self.frame_times.is_empty() {
                return 0.0;
            }
            let avg_time: Duration = self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32;
            1.0 / avg_time.as_secs_f64()
        }

        pub fn percentile_frame_time(&self, percentile: f32) -> Duration {
            if self.frame_times.is_empty() {
                return Duration::ZERO;
            }
            let mut sorted: Vec<_> = self.frame_times.clone();
            sorted.sort();
            let index = ((sorted.len() as f32 * percentile / 100.0) as usize).min(sorted.len() - 1);
            sorted[index]
        }

        pub fn dropped_frames(&self, target_fps: f64) -> usize {
            let target_time = Duration::from_secs_f64(1.0 / target_fps);
            self.frame_times.iter().filter(|&&t| t > target_time).count()
        }

        pub fn average_gpu_utilization(&self) -> f32 {
            if self.gpu_utilization.is_empty() {
                return 0.0;
            }
            self.gpu_utilization.iter().sum::<f32>() / self.gpu_utilization.len() as f32
        }

        pub fn peak_memory_usage(&self) -> usize {
            self.memory_usage.iter().copied().max().unwrap_or(0)
        }
    }

    #[test]
    fn test_benchmarker_basic() {
        let benchmarker = Benchmarker::new(5, 10);

        let result = benchmarker.run("test_operation", || {
            std::thread::sleep(Duration::from_micros(100));
        });

        assert_eq!(result.name, "test_operation");
        assert_eq!(result.iterations, 10);
        assert!(result.avg_time >= Duration::from_micros(100));
    }

    #[test]
    fn test_benchmark_statistics() {
        let benchmarker = Benchmarker::new(2, 20);

        let result = benchmarker.run("stats_test", || {
            std::thread::sleep(Duration::from_micros(50));
        });

        assert!(result.min_time <= result.avg_time);
        assert!(result.avg_time <= result.max_time);
        assert!(result.throughput.unwrap() > 0.0);
    }

    #[test]
    fn test_fps_calculation() {
        let result = BenchmarkResult {
            name: "test".to_string(),
            iterations: 100,
            total_time: Duration::from_secs(1),
            min_time: Duration::from_millis(8),
            max_time: Duration::from_millis(12),
            avg_time: Duration::from_millis(10),
            std_dev: Duration::from_millis(1),
            throughput: Some(100.0),
        };

        assert!((result.fps() - 100.0).abs() < 0.1);
        assert!((result.ms_per_frame() - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_mock_gpu_operation() {
        let op = MockGpuOperation::new(OperationType::ColorCorrection, 1920, 1080);

        let start = Instant::now();
        op.execute();
        let elapsed = start.elapsed();


        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn test_operation_complexity() {
        let simple = MockGpuOperation::new(OperationType::Blur, 1920, 1080)
            .with_complexity(0.1);
        let complex = MockGpuOperation::new(OperationType::Blur, 1920, 1080)
            .with_complexity(1.0);

        assert!(simple.estimated_time_ms() < complex.estimated_time_ms());
    }

    #[test]
    fn test_resolution_scaling() {
        let hd = MockGpuOperation::new(OperationType::Render, 1920, 1080);
        let uhd = MockGpuOperation::new(OperationType::Render, 3840, 2160);


        let ratio = uhd.estimated_time_ms() / hd.estimated_time_ms();
        assert!(ratio > 3.5 && ratio < 4.5);
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();

        for i in 0..100 {
            metrics.record_frame(
                Duration::from_millis(16 + (i % 5) as u64),
                0.7 + (i as f32 % 10.0) / 100.0,
                1024 * 1024 * (100 + i),
            );
        }

        assert!(metrics.average_fps() > 0.0);
        assert!(metrics.average_gpu_utilization() > 0.0);
        assert!(metrics.peak_memory_usage() > 0);
    }

    #[test]
    fn test_percentile_frame_time() {
        let mut metrics = PerformanceMetrics::new();

        for i in 0..100 {
            metrics.record_frame(
                Duration::from_millis(i as u64 + 1),
                0.5,
                1024,
            );
        }

        let p50 = metrics.percentile_frame_time(50.0);
        let p99 = metrics.percentile_frame_time(99.0);

        assert!(p50 < p99);
        assert!(p50 >= Duration::from_millis(50));
    }

    #[test]
    fn test_dropped_frames() {
        let mut metrics = PerformanceMetrics::new();


        metrics.record_frame(Duration::from_millis(10), 0.5, 1024);
        metrics.record_frame(Duration::from_millis(15), 0.5, 1024);
        metrics.record_frame(Duration::from_millis(20), 0.5, 1024);
        metrics.record_frame(Duration::from_millis(30), 0.5, 1024);
        metrics.record_frame(Duration::from_millis(16), 0.5, 1024);

        let dropped = metrics.dropped_frames(60.0);
        assert_eq!(dropped, 2);
    }

    #[test]
    fn test_benchmark_multiple_operations() {
        let benchmarker = Benchmarker::new(2, 10);

        let operations = vec![
            MockGpuOperation::new(OperationType::ColorCorrection, 1920, 1080),
            MockGpuOperation::new(OperationType::Blur, 1920, 1080),
            MockGpuOperation::new(OperationType::Composite, 1920, 1080),
        ];

        let mut results = Vec::new();
        for (i, op) in operations.iter().enumerate() {
            let result = benchmarker.run(&format!("op_{}", i), || {
                op.execute();
            });
            results.push(result);
        }

        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(result.avg_time > Duration::ZERO);
        }
    }

    #[test]
    fn test_4k_playback_benchmark() {
        let benchmarker = Benchmarker::new(2, 10);

        let decode = MockGpuOperation::new(OperationType::Transform, 3840, 2160);
        let color = MockGpuOperation::new(OperationType::ColorCorrection, 3840, 2160);
        let render = MockGpuOperation::new(OperationType::Render, 3840, 2160);

        let result = benchmarker.run("4k_playback", || {
            decode.execute();
            color.execute();
            render.execute();
        });


        let fps = result.fps();
        println!("4K playback benchmark: {:.1} FPS", fps);

    }

    #[test]
    fn test_multi_track_performance() {
        let benchmarker = Benchmarker::new(2, 10);
        let track_count = 4;

        let result = benchmarker.run("multi_track", || {
            for _ in 0..track_count {
                let op = MockGpuOperation::new(OperationType::Composite, 1920, 1080)
                    .with_complexity(0.3);
                op.execute();
            }
        });

        println!("Multi-track ({} tracks) benchmark: {:.1} FPS", track_count, result.fps());
    }
}
