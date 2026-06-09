#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use std::collections::VecDeque;


    #[derive(Debug, Clone)]
    pub struct GpuMetrics {
        pub utilization_percent: f32,
        pub memory_used_mb: u64,
        pub memory_total_mb: u64,
        pub temperature_c: f32,
        pub power_watts: f32,
        pub clock_mhz: u32,
    }

    impl GpuMetrics {
        pub fn memory_usage_percent(&self) -> f32 {
            if self.memory_total_mb == 0 {
                return 0.0;
            }
            self.memory_used_mb as f32 / self.memory_total_mb as f32 * 100.0
        }
    }


    pub struct GpuMonitor {
        samples: VecDeque<GpuMetrics>,
        max_samples: usize,
        sample_interval: Duration,
        last_sample: Instant,
    }

    impl GpuMonitor {
        pub fn new(max_samples: usize, sample_interval: Duration) -> Self {
            Self {
                samples: VecDeque::with_capacity(max_samples),
                max_samples,
                sample_interval,
                last_sample: Instant::now(),
            }
        }

        pub fn sample(&mut self, metrics: GpuMetrics) {
            if self.samples.len() >= self.max_samples {
                self.samples.pop_front();
            }
            self.samples.push_back(metrics);
            self.last_sample = Instant::now();
        }

        pub fn average_utilization(&self) -> f32 {
            if self.samples.is_empty() {
                return 0.0;
            }
            self.samples.iter().map(|m| m.utilization_percent).sum::<f32>() / self.samples.len() as f32
        }

        pub fn peak_utilization(&self) -> f32 {
            self.samples.iter()
                .map(|m| m.utilization_percent)
                .fold(0.0f32, |a, b| a.max(b))
        }

        pub fn average_memory_usage(&self) -> f32 {
            if self.samples.is_empty() {
                return 0.0;
            }
            self.samples.iter().map(|m| m.memory_usage_percent()).sum::<f32>() / self.samples.len() as f32
        }

        pub fn peak_memory_mb(&self) -> u64 {
            self.samples.iter()
                .map(|m| m.memory_used_mb)
                .max()
                .unwrap_or(0)
        }

        pub fn average_temperature(&self) -> f32 {
            if self.samples.is_empty() {
                return 0.0;
            }
            self.samples.iter().map(|m| m.temperature_c).sum::<f32>() / self.samples.len() as f32
        }

        pub fn average_power(&self) -> f32 {
            if self.samples.is_empty() {
                return 0.0;
            }
            self.samples.iter().map(|m| m.power_watts).sum::<f32>() / self.samples.len() as f32
        }

        pub fn sample_count(&self) -> usize {
            self.samples.len()
        }
    }


    pub struct MockGpuWorkload {
        pub name: String,
        pub compute_intensity: f32,
        pub memory_intensity: f32,
        pub duration_ms: u64,
    }

    impl MockGpuWorkload {
        pub fn new(name: &str, compute: f32, memory: f32, duration_ms: u64) -> Self {
            Self {
                name: name.to_string(),
                compute_intensity: compute.clamp(0.0, 1.0),
                memory_intensity: memory.clamp(0.0, 1.0),
                duration_ms,
            }
        }

        pub fn execute(&self) -> GpuMetrics {

            std::thread::sleep(Duration::from_millis(self.duration_ms.min(100)));

            GpuMetrics {
                utilization_percent: self.compute_intensity * 100.0,
                memory_used_mb: (self.memory_intensity * 8192.0) as u64,
                memory_total_mb: 8192,
                temperature_c: 40.0 + self.compute_intensity * 40.0,
                power_watts: 50.0 + self.compute_intensity * 200.0,
                clock_mhz: 1500 + (self.compute_intensity * 500.0) as u32,
            }
        }
    }


    pub struct GpuBenchmarkSuite {
        workloads: Vec<MockGpuWorkload>,
        monitor: GpuMonitor,
    }

    impl GpuBenchmarkSuite {
        pub fn new() -> Self {
            Self {
                workloads: Vec::new(),
                monitor: GpuMonitor::new(1000, Duration::from_millis(16)),
            }
        }

        pub fn add_workload(&mut self, workload: MockGpuWorkload) {
            self.workloads.push(workload);
        }

        pub fn run(&mut self) -> GpuBenchmarkResults {
            let start = Instant::now();
            let mut workload_results = Vec::new();

            for workload in &self.workloads {
                let workload_start = Instant::now();
                let iterations = 10;

                for _ in 0..iterations {
                    let metrics = workload.execute();
                    self.monitor.sample(metrics);
                }

                let workload_time = workload_start.elapsed();
                workload_results.push(WorkloadResult {
                    name: workload.name.clone(),
                    total_time: workload_time,
                    iterations,
                    avg_time_per_iteration: workload_time / iterations,
                });
            }

            GpuBenchmarkResults {
                total_time: start.elapsed(),
                workload_results,
                average_utilization: self.monitor.average_utilization(),
                peak_utilization: self.monitor.peak_utilization(),
                average_memory_percent: self.monitor.average_memory_usage(),
                peak_memory_mb: self.monitor.peak_memory_mb(),
                average_temperature: self.monitor.average_temperature(),
                average_power: self.monitor.average_power(),
            }
        }
    }

    #[derive(Debug)]
    pub struct WorkloadResult {
        pub name: String,
        pub total_time: Duration,
        pub iterations: u32,
        pub avg_time_per_iteration: Duration,
    }

    #[derive(Debug)]
    pub struct GpuBenchmarkResults {
        pub total_time: Duration,
        pub workload_results: Vec<WorkloadResult>,
        pub average_utilization: f32,
        pub peak_utilization: f32,
        pub average_memory_percent: f32,
        pub peak_memory_mb: u64,
        pub average_temperature: f32,
        pub average_power: f32,
    }

    impl GpuBenchmarkResults {
        pub fn print(&self) {
            println!("=== GPU Benchmark Results ===");
            println!("Total time: {:?}", self.total_time);
            println!("Average utilization: {:.1}%", self.average_utilization);
            println!("Peak utilization: {:.1}%", self.peak_utilization);
            println!("Average memory: {:.1}%", self.average_memory_percent);
            println!("Peak memory: {} MB", self.peak_memory_mb);
            println!("Average temperature: {:.1}°C", self.average_temperature);
            println!("Average power: {:.1}W", self.average_power);
            println!("\nWorkload Results:");
            for result in &self.workload_results {
                println!("  {}: {:?} ({} iterations)",
                    result.name, result.avg_time_per_iteration, result.iterations);
            }
        }
    }


    pub fn color_correction_workload() -> MockGpuWorkload {
        MockGpuWorkload::new("Color Correction", 0.4, 0.3, 5)
    }

    pub fn blur_workload() -> MockGpuWorkload {
        MockGpuWorkload::new("Gaussian Blur", 0.6, 0.4, 8)
    }

    pub fn composite_workload() -> MockGpuWorkload {
        MockGpuWorkload::new("Compositing", 0.5, 0.6, 6)
    }

    pub fn transform_workload() -> MockGpuWorkload {
        MockGpuWorkload::new("Transform", 0.3, 0.2, 3)
    }

    pub fn denoise_workload() -> MockGpuWorkload {
        MockGpuWorkload::new("Denoise", 0.9, 0.5, 15)
    }

    pub fn render_workload() -> MockGpuWorkload {
        MockGpuWorkload::new("Final Render", 0.8, 0.7, 10)
    }

    #[test]
    fn test_gpu_metrics() {
        let metrics = GpuMetrics {
            utilization_percent: 75.0,
            memory_used_mb: 4096,
            memory_total_mb: 8192,
            temperature_c: 65.0,
            power_watts: 180.0,
            clock_mhz: 1800,
        };

        assert_eq!(metrics.memory_usage_percent(), 50.0);
    }

    #[test]
    fn test_gpu_monitor() {
        let mut monitor = GpuMonitor::new(100, Duration::from_millis(16));

        for i in 0..10 {
            monitor.sample(GpuMetrics {
                utilization_percent: 50.0 + i as f32 * 5.0,
                memory_used_mb: 2000 + i as u64 * 100,
                memory_total_mb: 8192,
                temperature_c: 60.0,
                power_watts: 150.0,
                clock_mhz: 1700,
            });
        }

        assert_eq!(monitor.sample_count(), 10);
        assert!(monitor.average_utilization() > 50.0);
        assert!(monitor.peak_utilization() >= 95.0);
    }

    #[test]
    fn test_workload_execution() {
        let workload = MockGpuWorkload::new("Test", 0.5, 0.5, 10);
        let metrics = workload.execute();

        assert!(metrics.utilization_percent > 0.0);
        assert!(metrics.memory_used_mb > 0);
    }

    #[test]
    fn test_benchmark_suite() {
        let mut suite = GpuBenchmarkSuite::new();

        suite.add_workload(color_correction_workload());
        suite.add_workload(blur_workload());
        suite.add_workload(composite_workload());

        let results = suite.run();

        assert_eq!(results.workload_results.len(), 3);
        assert!(results.average_utilization > 0.0);
    }

    #[test]
    fn test_color_correction_benchmark() {
        let workload = color_correction_workload();
        let mut monitor = GpuMonitor::new(100, Duration::from_millis(16));

        for _ in 0..30 {
            let metrics = workload.execute();
            monitor.sample(metrics);
        }

        println!("Color Correction:");
        println!("  Avg utilization: {:.1}%", monitor.average_utilization());
        println!("  Avg memory: {:.1}%", monitor.average_memory_usage());
    }

    #[test]
    fn test_denoise_benchmark() {
        let workload = denoise_workload();
        let mut monitor = GpuMonitor::new(100, Duration::from_millis(16));

        for _ in 0..30 {
            let metrics = workload.execute();
            monitor.sample(metrics);
        }

        println!("Denoise:");
        println!("  Avg utilization: {:.1}%", monitor.average_utilization());
        println!("  Peak utilization: {:.1}%", monitor.peak_utilization());
        println!("  Avg temperature: {:.1}°C", monitor.average_temperature());
    }

    #[test]
    fn test_full_pipeline_benchmark() {
        let mut suite = GpuBenchmarkSuite::new();

        suite.add_workload(transform_workload());
        suite.add_workload(color_correction_workload());
        suite.add_workload(blur_workload());
        suite.add_workload(denoise_workload());
        suite.add_workload(composite_workload());
        suite.add_workload(render_workload());

        let results = suite.run();

        println!("\nFull Pipeline Benchmark:");
        results.print();
    }

    #[test]
    fn test_memory_intensive_workload() {
        let workload = MockGpuWorkload::new("Memory Intensive", 0.3, 0.9, 10);
        let mut monitor = GpuMonitor::new(100, Duration::from_millis(16));

        for _ in 0..20 {
            let metrics = workload.execute();
            monitor.sample(metrics);
        }

        assert!(monitor.average_memory_usage() > 80.0);
        println!("Memory intensive workload: {:.1}% memory usage", monitor.average_memory_usage());
    }

    #[test]
    fn test_compute_intensive_workload() {
        let workload = MockGpuWorkload::new("Compute Intensive", 0.95, 0.3, 10);
        let mut monitor = GpuMonitor::new(100, Duration::from_millis(16));

        for _ in 0..20 {
            let metrics = workload.execute();
            monitor.sample(metrics);
        }

        assert!(monitor.average_utilization() > 90.0);
        println!("Compute intensive workload: {:.1}% GPU utilization", monitor.average_utilization());
    }

    #[test]
    fn test_sustained_workload() {
        let workload = MockGpuWorkload::new("Sustained", 0.7, 0.5, 5);
        let mut monitor = GpuMonitor::new(1000, Duration::from_millis(16));

        let start = Instant::now();
        let target_duration = Duration::from_millis(500);

        while start.elapsed() < target_duration {
            let metrics = workload.execute();
            monitor.sample(metrics);
        }

        println!("Sustained workload ({:?}):", start.elapsed());
        println!("  Samples: {}", monitor.sample_count());
        println!("  Avg utilization: {:.1}%", monitor.average_utilization());
        println!("  Avg temperature: {:.1}°C", monitor.average_temperature());
        println!("  Avg power: {:.1}W", monitor.average_power());
    }

    #[test]
    fn test_workload_comparison() {
        let workloads = vec![
            ("Color Correction", color_correction_workload()),
            ("Blur", blur_workload()),
            ("Composite", composite_workload()),
            ("Transform", transform_workload()),
            ("Denoise", denoise_workload()),
            ("Render", render_workload()),
        ];

        println!("\nWorkload Comparison:");
        println!("{:<20} {:>12} {:>12} {:>12}", "Workload", "GPU %", "Memory %", "Power W");
        println!("{}", "-".repeat(60));

        for (name, workload) in workloads {
            let metrics = workload.execute();
            println!("{:<20} {:>11.1}% {:>11.1}% {:>11.1}",
                name,
                metrics.utilization_percent,
                metrics.memory_usage_percent(),
                metrics.power_watts);
        }
    }
}
