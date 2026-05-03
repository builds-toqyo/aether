use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use anyhow::Result;

use super::PreviewQuality;

/// Performance monitor for preview sessions
#[derive(Debug)]
pub struct PerformanceMonitor {
    session_performance: HashMap<Uuid, SessionPerformance>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            session_performance: HashMap::new(),
        }
    }
    
    /// Record frame render performance
    pub fn record_frame_render(&mut self, session_id: Uuid, render_time: Duration, quality: PreviewQuality) {
        let performance = self.session_performance.entry(session_id).or_insert_with(|| SessionPerformance::new());
        performance.record_frame(render_time, quality);
    }
    
    /// Get session performance metrics
    pub fn get_session_performance(&self, session_id: &Uuid) -> SessionPerformance {
        self.session_performance.get(session_id).cloned().unwrap_or_else(|| SessionPerformance::new())
    }
    
    /// Remove session performance data
    pub fn remove_session(&mut self, session_id: &Uuid) {
        self.session_performance.remove(session_id);
    }
    
    /// Get all session performance data
    pub fn get_all_performance(&self) -> &HashMap<Uuid, SessionPerformance> {
        &self.session_performance
    }
    
    /// Clear all performance data
    pub fn clear(&mut self) {
        self.session_performance.clear();
    }
    
    /// Get total number of sessions
    pub fn session_count(&self) -> usize {
        self.session_performance.len()
    }
    
    /// Get average performance across all sessions
    pub fn average_performance(&self) -> Option<Duration> {
        if self.session_performance.is_empty() {
            return None;
        }
        
        let total_frames: u64 = self.session_performance.values()
            .map(|p| p.frame_count)
            .sum();
        
        if total_frames == 0 {
            return None;
        }
        
        let total_time: Duration = self.session_performance.values()
            .map(|p| p.total_render_time)
            .sum();
        
        Some(total_time / total_frames as u32)
    }
}

/// Session performance metrics
#[derive(Debug, Clone)]
pub struct SessionPerformance {
    pub frame_count: u64,
    pub total_render_time: Duration,
    pub recent_render_times: Vec<Duration>,
    pub quality_distribution: HashMap<PreviewQuality, u64>,
    pub dropped_frames: u64,
    pub peak_render_time: Duration,
    pub min_render_time: Duration,
}

impl SessionPerformance {
    /// Create new session performance
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            total_render_time: Duration::ZERO,
            recent_render_times: Vec::new(),
            quality_distribution: HashMap::new(),
            dropped_frames: 0,
            peak_render_time: Duration::ZERO,
            min_render_time: Duration::MAX,
        }
    }
    
    /// Record a frame render
    pub fn record_frame(&mut self, render_time: Duration, quality: PreviewQuality) {
        self.frame_count += 1;
        self.total_render_time += render_time;
        
        // Update recent render times
        self.recent_render_times.push(render_time);
        if self.recent_render_times.len() > 20 {
            self.recent_render_times.remove(0);
        }
        
        // Update quality distribution
        *self.quality_distribution.entry(quality).or_insert(0) += 1;
        
        // Update peak and min times
        if render_time > self.peak_render_time {
            self.peak_render_time = render_time;
        }
        if render_time < self.min_render_time {
            self.min_render_time = render_time;
        }
    }
    
    /// Record a dropped frame
    pub fn record_dropped_frame(&mut self) {
        self.dropped_frames += 1;
    }
    
    /// Get average render time
    pub fn average_render_time(&self) -> Duration {
        if self.frame_count == 0 {
            Duration::ZERO
        } else {
            self.total_render_time / self.frame_count as u32
        }
    }
    
    /// Get recent average render time
    pub fn recent_average_render_time(&self) -> Duration {
        if self.recent_render_times.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = self.recent_render_times.iter().sum();
            total / self.recent_render_times.len() as u32
        }
    }
    
    /// Get frame rate (FPS)
    pub fn frame_rate(&self) -> f64 {
        let avg_time = self.average_render_time();
        if avg_time.as_secs_f64() > 0.0 {
            1.0 / avg_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get recent frame rate
    pub fn recent_frame_rate(&self) -> f64 {
        let avg_time = self.recent_average_render_time();
        if avg_time.as_secs_f64() > 0.0 {
            1.0 / avg_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get drop rate
    pub fn drop_rate(&self) -> f64 {
        let total_frames = self.frame_count + self.dropped_frames;
        if total_frames > 0 {
            self.dropped_frames as f64 / total_frames as f64
        } else {
            0.0
        }
    }
    
    /// Get most used quality
    pub fn most_used_quality(&self) -> Option<PreviewQuality> {
        self.quality_distribution
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&quality, _)| quality)
    }
    
    /// Get quality distribution as percentages
    pub fn quality_distribution_percentages(&self) -> HashMap<PreviewQuality, f64> {
        let total_frames = self.frame_count;
        if total_frames == 0 {
            return HashMap::new();
        }
        
        self.quality_distribution
            .iter()
            .map(|(&quality, &count)| (quality, count as f64 / total_frames as f64))
            .collect()
    }
    
    /// Check if performance is stable (low variance)
    pub fn is_stable(&self, threshold: f64) -> bool {
        if self.recent_render_times.len() < 3 {
            return false;
        }
        
        let avg = self.recent_average_render_time().as_secs_f64();
        let variance: f64 = self.recent_render_times
            .iter()
            .map(|&time| {
                let diff = time.as_secs_f64() - avg;
                diff * diff
            })
            .sum::<f64>() / self.recent_render_times.len() as f64;
        
        let std_dev = variance.sqrt();
        (std_dev / avg) < threshold
    }
    
    /// Reset performance metrics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    /// Get performance summary
    pub fn summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            frame_count: self.frame_count,
            dropped_frames: self.dropped_frames,
            frame_rate: self.frame_rate(),
            recent_frame_rate: self.recent_frame_rate(),
            average_render_time: self.average_render_time(),
            peak_render_time: self.peak_render_time,
            min_render_time: if self.min_render_time == Duration::MAX { Duration::ZERO } else { self.min_render_time },
            drop_rate: self.drop_rate(),
            most_used_quality: self.most_used_quality(),
            is_stable: self.is_stable(0.2), // 20% variance threshold
        }
    }
}

/// Performance summary for easy display
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub frame_count: u64,
    pub dropped_frames: u64,
    pub frame_rate: f64,
    pub recent_frame_rate: f64,
    pub average_render_time: Duration,
    pub peak_render_time: Duration,
    pub min_render_time: Duration,
    pub drop_rate: f64,
    pub most_used_quality: Option<PreviewQuality>,
    pub is_stable: bool,
}

impl PerformanceSummary {
    /// Format summary for display
    pub fn format(&self) -> String {
        format!(
            "Frames: {} | FPS: {:.1} (recent: {:.1}) | Avg: {:.1}ms | Drop: {:.1}% | Quality: {:?} | Stable: {}",
            self.frame_count,
            self.frame_rate,
            self.recent_frame_rate,
            self.average_render_time.as_secs_f64() * 1000.0,
            self.drop_rate * 100.0,
            self.most_used_quality.unwrap_or(PreviewQuality::Medium),
            self.is_stable
        )
    }
}

/// Preview system statistics
#[derive(Debug, Clone)]
pub struct PreviewStats {
    pub active_sessions: usize,
    pub total_sessions: usize,
    pub frames_rendered: u64,
    pub total_render_time: Duration,
    pub average_fps: f64,
    pub total_dropped_frames: u64,
    pub peak_concurrent_sessions: usize,
}

impl PreviewStats {
    /// Create new preview stats
    pub fn new() -> Self {
        Self {
            active_sessions: 0,
            total_sessions: 0,
            frames_rendered: 0,
            total_render_time: Duration::ZERO,
            average_fps: 0.0,
            total_dropped_frames: 0,
            peak_concurrent_sessions: 0,
        }
    }
    
    /// Format statistics for display
    pub fn format(&self) -> String {
        format!(
            "Sessions: {}/{} | Frames: {} | Avg FPS: {:.1} | Total Render Time: {:.2}s | Drop Rate: {:.1}%",
            self.active_sessions,
            self.total_sessions,
            self.frames_rendered,
            self.average_fps,
            self.total_render_time.as_secs_f64(),
            if self.frames_rendered > 0 {
                (self.total_dropped_frames as f64 / (self.frames_rendered + self.total_dropped_frames) as f64) * 100.0
            } else {
                0.0
            }
        )
    }
    
    /// Update frame statistics
    pub fn update_frame_stats(&mut self, render_time: Duration, dropped: bool) {
        self.frames_rendered += 1;
        self.total_render_time += render_time;
        
        if dropped {
            self.total_dropped_frames += 1;
        }
        
        // Update average FPS
        if self.total_render_time.as_secs_f64() > 0.0 {
            self.average_fps = self.frames_rendered as f64 / self.total_render_time.as_secs_f64();
        }
    }
    
    /// Update session statistics
    pub fn update_session_stats(&mut self, active: usize, total: usize) {
        self.active_sessions = active;
        self.total_sessions = total;
        
        if active > self.peak_concurrent_sessions {
            self.peak_concurrent_sessions = active;
        }
    }
}

impl Default for PreviewStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_performance() {
        let mut performance = SessionPerformance::new();
        
        // Record some frames
        performance.record_frame(Duration::from_millis(16), PreviewQuality::High);
        performance.record_frame(Duration::from_millis(20), PreviewQuality::Medium);
        performance.record_frame(Duration::from_millis(12), PreviewQuality::High);
        
        assert_eq!(performance.frame_count, 3);
        assert_eq!(performance.average_render_time(), Duration::from_millis(16));
        assert_eq!(performance.frame_rate(), 62.5); // 1/0.016
        assert_eq!(performance.most_used_quality(), Some(PreviewQuality::High));
    }
    
    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        let session_id = Uuid::new_v4();
        
        monitor.record_frame_render(session_id, Duration::from_millis(16), PreviewQuality::High);
        
        let performance = monitor.get_session_performance(&session_id);
        assert_eq!(performance.frame_count, 1);
        
        assert_eq!(monitor.session_count(), 1);
    }
    
    #[test]
    fn test_preview_stats() {
        let mut stats = PreviewStats::new();
        
        stats.update_frame_stats(Duration::from_millis(16), false);
        stats.update_frame_stats(Duration::from_millis(20), true);
        
        assert_eq!(stats.frames_rendered, 2);
        assert_eq!(stats.total_dropped_frames, 1);
        
        let formatted = stats.format();
        assert!(formatted.contains("Sessions: 0/0"));
        assert!(formatted.contains("Frames: 2"));
    }
    
    #[test]
    fn test_performance_summary() {
        let mut performance = SessionPerformance::new();
        
        performance.record_frame(Duration::from_millis(16), PreviewQuality::High);
        performance.record_frame(Duration::from_millis(20), PreviewQuality::Medium);
        
        let summary = performance.summary();
        assert_eq!(summary.frame_count, 2);
        assert_eq!(summary.frame_rate, 56.25); // 2 / (0.016 + 0.020)
        
        let formatted = summary.format();
        assert!(formatted.contains("Frames: 2"));
        assert!(formatted.contains("FPS: 56.3"));
    }
}
