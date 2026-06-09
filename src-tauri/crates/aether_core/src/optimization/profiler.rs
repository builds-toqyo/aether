use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};


#[derive(Debug, Clone)]
pub struct ProfileSample {
    pub name: String,
    pub start_time: Instant,
    pub duration: Duration,
    pub depth: usize,
    pub metadata: HashMap<String, String>,
}


#[derive(Debug, Clone, Default)]
pub struct ProfileStats {
    pub name: String,
    pub call_count: u64,
    pub total_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub avg_time: Duration,
}

impl ProfileStats {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            call_count: 0,
            total_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
            avg_time: Duration::ZERO,
        }
    }

    pub fn record(&mut self, duration: Duration) {
        self.call_count += 1;
        self.total_time += duration;
        self.min_time = self.min_time.min(duration);
        self.max_time = self.max_time.max(duration);
        self.avg_time = self.total_time / self.call_count as u32;
    }

    pub fn calls_per_second(&self, elapsed: Duration) -> f64 {
        if elapsed.as_secs_f64() == 0.0 {
            return 0.0;
        }
        self.call_count as f64 / elapsed.as_secs_f64()
    }
}


pub struct Profiler {
    enabled: bool,
    samples: Arc<Mutex<Vec<ProfileSample>>>,
    stats: Arc<Mutex<HashMap<String, ProfileStats>>>,
    start_time: Instant,
    current_depth: Arc<Mutex<usize>>,
    max_samples: usize,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            enabled: true,
            samples: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
            current_depth: Arc::new(Mutex::new(0)),
            max_samples: 10000,
        }
    }

    pub fn with_max_samples(mut self, max: usize) -> Self {
        self.max_samples = max;
        self
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }


    pub fn begin(&self, name: &str) -> ProfileScope {
        if !self.enabled {
            return ProfileScope::disabled();
        }

        let depth = {
            let mut d = self.current_depth.lock().unwrap();
            let current = *d;
            *d += 1;
            current
        };

        ProfileScope {
            name: name.to_string(),
            start: Instant::now(),
            depth,
            profiler: Some(ProfilerRef {
                samples: Arc::clone(&self.samples),
                stats: Arc::clone(&self.stats),
                current_depth: Arc::clone(&self.current_depth),
                max_samples: self.max_samples,
            }),
            metadata: HashMap::new(),
        }
    }


    pub fn record(&self, name: &str, duration: Duration) {
        if !self.enabled {
            return;
        }

        let mut stats = self.stats.lock().unwrap();
        stats.entry(name.to_string())
            .or_insert_with(|| ProfileStats::new(name))
            .record(duration);
    }


    pub fn get_stats(&self) -> HashMap<String, ProfileStats> {
        self.stats.lock().unwrap().clone()
    }


    pub fn get_section_stats(&self, name: &str) -> Option<ProfileStats> {
        self.stats.lock().unwrap().get(name).cloned()
    }


    pub fn get_samples(&self) -> Vec<ProfileSample> {
        self.samples.lock().unwrap().clone()
    }


    pub fn clear(&self) {
        self.samples.lock().unwrap().clear();
        self.stats.lock().unwrap().clear();
    }


    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }


    pub fn report(&self) -> ProfileReport {
        let stats = self.get_stats();
        let elapsed = self.elapsed();

        let mut sections: Vec<_> = stats.into_iter()
            .map(|(name, stats)| ProfileReportSection {
                name,
                stats,
                percent_of_total: 0.0,
            })
            .collect();


        let total_time: Duration = sections.iter()
            .map(|s| s.stats.total_time)
            .sum();

        for section in &mut sections {
            if total_time.as_nanos() > 0 {
                section.percent_of_total =
                    section.stats.total_time.as_secs_f64() / total_time.as_secs_f64() * 100.0;
            }
        }


        sections.sort_by(|a, b| b.stats.total_time.cmp(&a.stats.total_time));

        ProfileReport {
            elapsed,
            total_profiled_time: total_time,
            sections,
        }
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}


struct ProfilerRef {
    samples: Arc<Mutex<Vec<ProfileSample>>>,
    stats: Arc<Mutex<HashMap<String, ProfileStats>>>,
    current_depth: Arc<Mutex<usize>>,
    max_samples: usize,
}


pub struct ProfileScope {
    name: String,
    start: Instant,
    depth: usize,
    profiler: Option<ProfilerRef>,
    metadata: HashMap<String, String>,
}

impl ProfileScope {
    fn disabled() -> Self {
        Self {
            name: String::new(),
            start: Instant::now(),
            depth: 0,
            profiler: None,
            metadata: HashMap::new(),
        }
    }


    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        if let Some(ref profiler) = self.profiler {
            let duration = self.start.elapsed();


            {
                let mut d = profiler.current_depth.lock().unwrap();
                if *d > 0 {
                    *d -= 1;
                }
            }


            {
                let mut samples = profiler.samples.lock().unwrap();
                if samples.len() < profiler.max_samples {
                    samples.push(ProfileSample {
                        name: self.name.clone(),
                        start_time: self.start,
                        duration,
                        depth: self.depth,
                        metadata: self.metadata.clone(),
                    });
                }
            }


            {
                let mut stats = profiler.stats.lock().unwrap();
                stats.entry(self.name.clone())
                    .or_insert_with(|| ProfileStats::new(&self.name))
                    .record(duration);
            }
        }
    }
}


#[derive(Debug)]
pub struct ProfileReport {
    pub elapsed: Duration,
    pub total_profiled_time: Duration,
    pub sections: Vec<ProfileReportSection>,
}

#[derive(Debug)]
pub struct ProfileReportSection {
    pub name: String,
    pub stats: ProfileStats,
    pub percent_of_total: f64,
}

impl ProfileReport {
    pub fn print(&self) {
        println!("=== Profile Report ===");
        println!("Total elapsed: {:?}", self.elapsed);
        println!("Total profiled: {:?}", self.total_profiled_time);
        println!();
        println!("{:<30} {:>10} {:>12} {:>12} {:>12} {:>8}",
            "Section", "Calls", "Total", "Avg", "Max", "%");
        println!("{}", "-".repeat(90));

        for section in &self.sections {
            println!("{:<30} {:>10} {:>12?} {:>12?} {:>12?} {:>7.1}%",
                section.name,
                section.stats.call_count,
                section.stats.total_time,
                section.stats.avg_time,
                section.stats.max_time,
                section.percent_of_total);
        }
    }
}


#[macro_export]
macro_rules! profile_scope {
    ($profiler:expr, $name:expr) => {
        let _scope = $profiler.begin($name);
    };
    ($profiler:expr, $name:expr, $($key:expr => $value:expr),+) => {
        let _scope = $profiler.begin($name)
            $(.with_metadata($key, $value))+;
    };
}


pub struct FrameProfiler {
    profiler: Profiler,
    frame_times: Vec<Duration>,
    max_frame_history: usize,
    current_frame_start: Option<Instant>,
}

impl FrameProfiler {
    pub fn new(max_history: usize) -> Self {
        Self {
            profiler: Profiler::new(),
            frame_times: Vec::with_capacity(max_history),
            max_frame_history: max_history,
            current_frame_start: None,
        }
    }

    pub fn begin_frame(&mut self) {
        self.current_frame_start = Some(Instant::now());
    }

    pub fn end_frame(&mut self) {
        if let Some(start) = self.current_frame_start.take() {
            let duration = start.elapsed();
            if self.frame_times.len() >= self.max_frame_history {
                self.frame_times.remove(0);
            }
            self.frame_times.push(duration);
        }
    }

    pub fn profiler(&self) -> &Profiler {
        &self.profiler
    }

    pub fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32
    }

    pub fn average_fps(&self) -> f64 {
        let avg = self.average_frame_time();
        if avg.as_secs_f64() == 0.0 {
            return 0.0;
        }
        1.0 / avg.as_secs_f64()
    }

    pub fn frame_count(&self) -> usize {
        self.frame_times.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profiler_basic() {
        let profiler = Profiler::new();

        {
            let _scope = profiler.begin("test_section");
            thread::sleep(Duration::from_millis(10));
        }

        let stats = profiler.get_section_stats("test_section");
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().call_count, 1);
    }

    #[test]
    fn test_profiler_multiple_calls() {
        let profiler = Profiler::new();

        for _ in 0..5 {
            let _scope = profiler.begin("repeated");
            thread::sleep(Duration::from_millis(1));
        }

        let stats = profiler.get_section_stats("repeated").unwrap();
        assert_eq!(stats.call_count, 5);
    }

    #[test]
    fn test_profiler_nested() {
        let profiler = Profiler::new();

        {
            let _outer = profiler.begin("outer");
            thread::sleep(Duration::from_millis(5));
            {
                let _inner = profiler.begin("inner");
                thread::sleep(Duration::from_millis(5));
            }
        }

        let samples = profiler.get_samples();
        assert_eq!(samples.len(), 2);
    }

    #[test]
    fn test_frame_profiler() {
        let mut frame_profiler = FrameProfiler::new(60);

        for _ in 0..10 {
            frame_profiler.begin_frame();
            thread::sleep(Duration::from_millis(16));
            frame_profiler.end_frame();
        }

        assert_eq!(frame_profiler.frame_count(), 10);
        assert!(frame_profiler.average_fps() > 0.0);
    }
}
