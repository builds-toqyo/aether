use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use log::{debug, info, warn};

use super::{PipelineHandle, CompiledShader};

/// Cache for compiled shaders and pipelines
pub struct ShaderCache {
    pipelines: HashMap<CacheKey, CachedPipeline>,
    shaders: HashMap<String, CachedShader>,
    cache_stats: CacheStats,
    max_cache_size: usize,
    max_age: Duration,
}

impl ShaderCache {
    /// Create a new shader cache
    pub fn new() -> Self {
        info!("Creating shader cache");
        
        Self {
            pipelines: HashMap::new(),
            shaders: HashMap::new(),
            cache_stats: CacheStats::new(),
            max_cache_size: 100, // Maximum number of cached items
            max_age: Duration::from_secs(3600), // 1 hour
        }
    }
    
    /// Cache a pipeline
    pub fn cache_pipeline(&mut self, key: CacheKey, pipeline: PipelineHandle) {
        debug!("Caching pipeline: {}", key.name);
        
        let cached = CachedPipeline {
            pipeline,
            cached_at: Instant::now(),
            access_count: 1,
            last_accessed: Instant::now(),
        };
        
        self.pipelines.insert(key, cached);
        self.cache_stats.pipeline_hits += 1;
        
        // Cleanup old entries if cache is full
        self.cleanup_pipelines();
        
        debug!("Pipeline cached: {} (total: {})", 
            key.name, self.pipelines.len());
    }
    
    /// Get a cached pipeline
    pub fn get_pipeline(&self, key: &CacheKey) -> Option<PipelineHandle> {
        debug!("Looking for cached pipeline: {}", key.name);
        
        if let Some(cached) = self.pipelines.get(key) {
            // Check if cache entry is still valid
            if cached.cached_at.elapsed() < self.max_age {
                debug!("Pipeline cache hit: {}", key.name);
                self.cache_stats.pipeline_hits += 1;
                return Some(cached.pipeline.clone());
            } else {
                debug!("Pipeline cache expired: {}", key.name);
                self.cache_stats.pipeline_misses += 1;
            }
        } else {
            debug!("Pipeline cache miss: {}", key.name);
            self.cache_stats.pipeline_misses += 1;
        }
        
        None
    }
    
    /// Cache a compiled shader
    pub fn cache_shader(&mut self, key: String, shader: CompiledShader) {
        debug!("Caching shader: {}", key);
        
        let cached = CachedShader {
            shader,
            cached_at: Instant::now(),
            access_count: 1,
            last_accessed: Instant::now(),
        };
        
        self.shaders.insert(key, cached);
        self.cache_stats.shader_hits += 1;
        
        // Cleanup old entries if cache is full
        self.cleanup_shaders();
        
        debug!("Shader cached (total: {})", self.shaders.len());
    }
    
    /// Get a cached shader
    pub fn get_shader(&self, key: &str) -> Option<CompiledShader> {
        debug!("Looking for cached shader: {}", key);
        
        if let Some(cached) = self.shaders.get(key) {
            // Check if cache entry is still valid
            if cached.cached_at.elapsed() < self.max_age {
                debug!("Shader cache hit: {}", key);
                self.cache_stats.shader_hits += 1;
                return Some(cached.shader.clone());
            } else {
                debug!("Shader cache expired: {}", key);
                self.cache_stats.shader_misses += 1;
            }
        } else {
            debug!("Shader cache miss: {}", key);
            self.cache_stats.shader_misses += 1;
        }
        
        None
    }
    
    /// Remove a cached pipeline
    pub fn remove_pipeline(&mut self, key: &CacheKey) -> Result<()> {
        debug!("Removing cached pipeline: {}", key.name);
        
        if self.pipelines.remove(key).is_some() {
            info!("Removed cached pipeline: {}", key.name);
        }
        
        Ok(())
    }
    
    /// Remove a cached shader
    pub fn remove_shader(&mut self, key: &str) -> Result<()> {
        debug!("Removing cached shader: {}", key);
        
        if self.shaders.remove(key).is_some() {
            info!("Removed cached shader: {}", key);
        }
        
        Ok(())
    }
    
    /// Clear all cached items
    pub fn clear(&mut self) {
        warn!("Clearing shader cache");
        
        let pipeline_count = self.pipelines.len();
        let shader_count = self.shaders.len();
        
        self.pipelines.clear();
        self.shaders.clear();
        
        info!("Cleared {} pipelines and {} shaders from cache", 
            pipeline_count, shader_count);
    }
    
    /// Get pipeline count
    pub fn pipeline_count(&self) -> usize {
        self.pipelines.len()
    }
    
    /// Get shader count
    pub fn shader_count(&self) -> usize {
        self.shaders.len()
    }
    
    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total_requests = self.cache_stats.pipeline_hits + self.cache_stats.pipeline_misses;
        if total_requests == 0 {
            0.0
        } else {
            self.cache_stats.pipeline_hits as f64 / total_requests as f64
        }
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let mut stats = self.cache_stats.clone();
        stats.cached_pipelines = self.pipeline_count();
        stats.cached_shaders = self.shader_count();
        stats.hit_rate = self.hit_rate();
        stats
    }
    
    /// Cleanup old pipeline entries
    fn cleanup_pipelines(&mut self) {
        if self.pipelines.len() <= self.max_cache_size {
            return;
        }
        
        debug!("Cleaning up old pipeline cache entries");
        
        // Remove entries that are too old
        let mut to_remove = Vec::new();
        for (key, cached) in &self.pipelines {
            if cached.cached_at.elapsed() > self.max_age {
                to_remove.push(key.clone());
            }
        }
        
        for key in to_remove {
            self.pipelines.remove(&key);
        }
        
        // If still too many, remove least recently used
        if self.pipelines.len() > self.max_cache_size {
            let mut entries: Vec<_> = self.pipelines.iter().collect();
            entries.sort_by_key(|(_, cached)| cached.last_accessed);
            
            let remove_count = self.pipelines.len() - self.max_cache_size;
            for (key, _) in entries.iter().take(remove_count) {
                self.pipelines.remove(key);
            }
        }
        
        debug!("Pipeline cleanup completed ({} entries)", self.pipelines.len());
    }
    
    /// Cleanup old shader entries
    fn cleanup_shaders(&mut self) {
        if self.shaders.len() <= self.max_cache_size {
            return;
        }
        
        debug!("Cleaning up old shader cache entries");
        
        // Remove entries that are too old
        let mut to_remove = Vec::new();
        for (key, cached) in &self.shaders {
            if cached.cached_at.elapsed() > self.max_age {
                to_remove.push(key.clone());
            }
        }
        
        for key in to_remove {
            self.shaders.remove(&key);
        }
        
        // If still too many, remove least recently used
        if self.shaders.len() > self.max_cache_size {
            let mut entries: Vec<_> = self.shaders.iter().collect();
            entries.sort_by_key(|(_, cached)| cached.last_accessed);
            
            let remove_count = self.shaders.len() - self.max_cache_size;
            for (key, _) in entries.iter().take(remove_count) {
                self.shaders.remove(key);
            }
        }
        
        debug!("Shader cleanup completed ({} entries)", self.shaders.len());
    }
}

/// Cache key for pipelines
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub name: String,
    pub source_hash: u64,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(name: &str, source: &str) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        source.hash(&mut hasher);
        
        Self {
            name: name.to_string(),
            source_hash: hasher.finish(),
        }
    }
}

/// Cached pipeline entry
#[derive(Debug, Clone)]
pub struct CachedPipeline {
    pub pipeline: PipelineHandle,
    pub cached_at: Instant,
    pub access_count: u64,
    pub last_accessed: Instant,
}

/// Cached shader entry
#[derive(Debug, Clone)]
pub struct CachedShader {
    pub shader: CompiledShader,
    pub cached_at: Instant,
    pub access_count: u64,
    pub last_accessed: Instant,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub pipeline_hits: u64,
    pub pipeline_misses: u64,
    pub shader_hits: u64,
    pub shader_misses: u64,
    pub cached_pipelines: usize,
    pub cached_shaders: usize,
    pub hit_rate: f64,
}

impl CacheStats {
    /// Create new cache statistics
    pub fn new() -> Self {
        Self {
            pipeline_hits: 0,
            pipeline_misses: 0,
            shader_hits: 0,
            shader_misses: 0,
            cached_pipelines: 0,
            cached_shaders: 0,
            hit_rate: 0.0,
        }
    }
    
    /// Format statistics for display
    pub fn format(&self) -> String {
        format!(
            "Pipelines: {} hits, {} misses | Shaders: {} hits, {} misses | Cache: {} pipelines, {} shaders | Hit rate: {:.1}%",
            self.pipeline_hits,
            self.pipeline_misses,
            self.shader_hits,
            self.shader_misses,
            self.cached_pipelines,
            self.cached_shaders,
            self.hit_rate * 100.0
        )
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_key() {
        let source1 = "fn main() {}";
        let source2 = "fn main() {}";
        let source3 = "fn other() {}";
        
        let key1 = CacheKey::new("test", source1);
        let key2 = CacheKey::new("test", source2);
        let key3 = CacheKey::new("test", source3);
        
        assert_eq!(key1, key2); // Same source should have same key
        assert_ne!(key1, key3); // Different source should have different key
    }
    
    #[test]
    fn test_cache_stats() {
        let stats = CacheStats {
            pipeline_hits: 10,
            pipeline_misses: 2,
            shader_hits: 5,
            shader_misses: 1,
            cached_pipelines: 3,
            cached_shaders: 2,
            hit_rate: 0.833,
        };
        
        let formatted = stats.format();
        assert!(formatted.contains("10 hits"));
        assert!(formatted.contains("2 misses"));
        assert!(formatted.contains("83.3%"));
    }
    
    #[test]
    fn test_shader_cache() {
        let mut cache = ShaderCache::new();
        
        assert_eq!(cache.pipeline_count(), 0);
        assert_eq!(cache.shader_count(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
        
        // Test cache operations
        let key = CacheKey::new("test", "fn main() {}");
        assert!(cache.get_pipeline(&key).is_none());
        
        let stats = cache.get_stats();
        assert_eq!(stats.pipeline_hits, 0);
        assert_eq!(stats.pipeline_misses, 1);
    }
}
