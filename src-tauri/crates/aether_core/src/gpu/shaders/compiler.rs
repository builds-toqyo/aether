use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wgpu::{Device, ShaderModule, ShaderStages};
use anyhow::{Result, anyhow};
use log::{debug, info, warn, error};

/// Shader compiler for WGSL shaders
pub struct ShaderCompiler {
    device: Arc<Device>,
    compiled_shaders: Arc<Mutex<HashMap<String, CompiledShader>>>,
    compilation_cache: Arc<Mutex<HashMap<String, Uuid>>>,
    compilation_count: std::sync::atomic::AtomicUsize,
}

impl ShaderCompiler {
    /// Create a new shader compiler
    pub fn new(device: Arc<Device>) -> Result<Self> {
        info!("Creating shader compiler");
        
        Ok(Self {
            device,
            compiled_shaders: Arc::new(Mutex::new(HashMap::new())),
            compilation_cache: Arc::new(Mutex::new(HashMap::new())),
            compilation_count: std::sync::atomic::AtomicUsize::new(0),
        })
    }
    
    /// Compile a shader from source code
    pub fn compile_shader(&self, name: &str, source: &str) -> Result<CompiledShader> {
        debug!("Compiling shader: {}", name);
        
        // Check cache first
        let cache_key = format!("{}-{}", name, self.hash_source(source));
        {
            let cache = self.compilation_cache.lock().map_err(|e| anyhow!("Cache lock error: {}", e))?;
            if let Some(shader_id) = cache.get(&cache_key) {
                let shaders = self.compiled_shaders.lock().map_err(|e| anyhow!("Shader lock error: {}", e))?;
                if let Some(compiled) = shaders.get(&shader_id.to_string()) {
                    debug!("Using cached compiled shader: {}", name);
                    return Ok(compiled.clone());
                }
            }
        }
        
        // Compile the shader
        let shader_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("Shader: {}", name)),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        
        // Validate the shader
        self.validate_shader(&shader_module)?;
        
        let compiled = CompiledShader {
            id: Uuid::new_v4(),
            name: name.to_string(),
            module: Arc::new(shader_module),
            source: source.to_string(),
            entry_point: "main".to_string(),
            compiled_at: std::time::Instant::now(),
        };
        
        // Cache the compiled shader
        {
            let mut shaders = self.compiled_shaders.lock().map_err(|e| anyhow!("Shader lock error: {}", e))?;
            shaders.insert(compiled.id.to_string(), compiled.clone());
        }
        
        {
            let mut cache = self.compilation_cache.lock().map_err(|e| anyhow!("Cache lock error: {}", e))?;
            cache.insert(cache_key, compiled.id);
        }
        
        self.compilation_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        info!("Compiled shader: {} ({})", name, compiled.id);
        
        Ok(compiled)
    }
    
    /// Compile shader with custom entry point
    pub fn compile_shader_with_entry(
        &self,
        name: &str,
        source: &str,
        entry_point: &str,
    ) -> Result<CompiledShader> {
        debug!("Compiling shader with custom entry: {}::{}", name, entry_point);
        
        let mut compiled = self.compile_shader(name, source)?;
        compiled.entry_point = entry_point.to_string();
        
        Ok(compiled)
    }
    
    /// Validate a compiled shader
    fn validate_shader(&self, shader_module: &ShaderModule) -> Result<()> {
        // In a real implementation, you might want to validate the shader
        // by checking reflection information or running validation passes
        debug!("Validating shader module");
        
        // For now, we'll assume the shader is valid if it compiles
        // In a production system, you'd want more thorough validation
        
        Ok(())
    }
    
    /// Get a compiled shader by ID
    pub fn get_shader(&self, shader_id: &str) -> Result<CompiledShader> {
        let shaders = self.compiled_shaders.lock().map_err(|e| anyhow!("Shader lock error: {}", e))?;
        
        shaders.get(shader_id)
            .cloned()
            .ok_or_else(|| anyhow!("Shader not found: {}", shader_id))
    }
    
    /// Remove a compiled shader
    pub fn remove_shader(&self, shader_id: &str) -> Result<()> {
        debug!("Removing compiled shader: {}", shader_id);
        
        let mut shaders = self.compiled_shaders.lock().map_err(|e| anyhow!("Shader lock error: {}", e))?;
        if shaders.remove(shader_id).is_some() {
            info!("Removed compiled shader: {}", shader_id);
        }
        
        // Remove from cache as well
        let mut cache = self.compilation_cache.lock().map_err(|e| anyhow!("Cache lock error: {}", e))?;
        cache.retain(|_, v| v.to_string() != shader_id);
        
        Ok(())
    }
    
    /// Clear all compiled shaders
    pub fn clear_shaders(&self) -> Result<()> {
        warn!("Clearing all compiled shaders");
        
        let mut shaders = self.compiled_shaders.lock().map_err(|e| anyhow!("Shader lock error: {}", e))?;
        shaders.clear();
        
        let mut cache = self.compilation_cache.lock().map_err(|e| anyhow!("Cache lock error: {}", e))?;
        cache.clear();
        
        info!("Cleared all compiled shaders");
        
        Ok(())
    }
    
    /// Get compilation count
    pub fn compiled_count(&self) -> usize {
        self.compilation_count.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Get shader count
    pub fn shader_count(&self) -> usize {
        self.compiled_shaders.lock().map(|s| s.len()).unwrap_or(0)
    }
    
    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.compilation_cache.lock().map(|c| c.len()).unwrap_or(0)
    }
    
    /// Generate hash for shader source
    fn hash_source(&self, source: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Preload common shaders
    pub fn preload_common_shaders(&self) -> Result<()> {
        info!("Preloading common compute shaders");
        
        // Common image processing shaders
        let common_shaders = [
            ("image_blend", include_str!("shaders/image_blend.wgsl")),
            ("color_convert", include_str!("shaders/color_convert.wgsl")),
            ("transform", include_str!("shaders/transform.wgsl")),
            ("blur", include_str!("shaders/blur.wgsl")),
        ];
        
        for (name, source) in common_shaders {
            if let Err(e) = self.compile_shader(name, source) {
                warn!("Failed to preload shader {}: {}", name, e);
            } else {
                debug!("Preloaded shader: {}", name);
            }
        }
        
        Ok(())
    }
    
    /// Get compiler statistics
    pub fn get_stats(&self) -> CompilerStats {
        CompilerStats {
            compiled_shaders: self.shader_count(),
            cache_entries: self.cache_size(),
            total_compilations: self.compiled_count(),
        }
    }
}

/// Compiled shader representation
#[derive(Debug, Clone)]
pub struct CompiledShader {
    pub id: Uuid,
    pub name: String,
    pub module: Arc<ShaderModule>,
    pub source: String,
    pub entry_point: String,
    pub compiled_at: std::time::Instant,
}

/// Shader source with metadata
#[derive(Debug, Clone)]
pub struct ShaderSource {
    pub name: String,
    pub source: String,
    pub entry_point: String,
    pub stage: ShaderStages,
}

/// Compiler statistics
#[derive(Debug, Clone)]
pub struct CompilerStats {
    pub compiled_shaders: usize,
    pub cache_entries: usize,
    pub total_compilations: usize,
}

impl CompilerStats {
    /// Format statistics for display
    pub fn format(&self) -> String {
        format!(
            "Compiled shaders: {}, Cache entries: {}, Total compilations: {}",
            self.compiled_shaders, self.cache_entries, self.total_compilations
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shader_source() {
        let source = ShaderSource {
            name: "test_shader".to_string(),
            source: "@compute @workgroup_size(1) fn main() {}", 
            entry_point: "main".to_string(),
            stage: ShaderStages::COMPUTE,
        };
        
        assert_eq!(source.name, "test_shader");
        assert_eq!(source.entry_point, "main");
        assert_eq!(source.stage, ShaderStages::COMPUTE);
    }
    
    #[test]
    fn test_compiler_stats() {
        let stats = CompilerStats {
            compiled_shaders: 5,
            cache_entries: 3,
            total_compilations: 7,
        };
        
        let formatted = stats.format();
        assert!(formatted.contains("Compiled shaders: 5"));
        assert!(formatted.contains("Cache entries: 3"));
        assert!(formatted.contains("Total compilations: 7"));
    }
    
    #[test]
    fn test_source_hashing() {
        let compiler = ShaderCompiler::new(Arc::new(create_mock_device())).unwrap();
        
        let source1 = "fn main() {}";
        let source2 = "fn main() {}";
        let source3 = "fn other() {}";
        
        let hash1 = compiler.hash_source(source1);
        let hash2 = compiler.hash_source(source2);
        let hash3 = compiler.hash_source(source3);
        
        assert_eq!(hash1, hash2); // Same source should have same hash
        assert_ne!(hash1, hash3); // Different source should have different hash
    }
    
    // Mock device for testing
    fn create_mock_device() -> wgpu::Device {
        // This would need a proper mock implementation
        panic!("Mock device implementation needed for tests")
    }
}
