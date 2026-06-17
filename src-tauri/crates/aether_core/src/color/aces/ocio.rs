use std::ffi::{c_char, c_void, CString};
use anyhow::Result;
use log::{debug, info};

#[allow(non_camel_case_types)]
pub type OCIO_Config = *mut c_void;
#[allow(non_camel_case_types)]
pub type OCIO_Processor = *mut c_void;
#[allow(non_camel_case_types)]
pub type OCIO_CPUProcessor = *mut c_void;

type FnConfigCreateFromFile = unsafe extern "C" fn(*const c_char) -> OCIO_Config;
type FnConfigRelease = unsafe extern "C" fn(OCIO_Config);
type FnConfigGetProcessor = unsafe extern "C" fn(OCIO_Config, *const c_char, *const c_char) -> OCIO_Processor;
type FnProcessorRelease = unsafe extern "C" fn(OCIO_Processor);
type FnProcessorGetCPUProcessor = unsafe extern "C" fn(OCIO_Processor) -> OCIO_CPUProcessor;
type FnCpuProcessorRelease = unsafe extern "C" fn(OCIO_CPUProcessor);
type FnCpuProcessorApplyRGB = unsafe extern "C" fn(OCIO_CPUProcessor, *mut f32);

pub struct OcioLib {
    _lib: libloading::Library,
    config_create_from_file: FnConfigCreateFromFile,
    config_release: FnConfigRelease,
    config_get_processor: FnConfigGetProcessor,
    processor_release: FnProcessorRelease,
    processor_get_cpu_processor: FnProcessorGetCPUProcessor,
    cpu_processor_release: FnCpuProcessorRelease,
    cpu_processor_apply_rgb: FnCpuProcessorApplyRGB,
}

impl OcioLib {
    pub fn load() -> Option<Self> {
        let lib_names = [
            "libOpenColorIO.so.2.2",
            "libOpenColorIO.so.2.1",
            "libOpenColorIO.so.2.0",
            "libOpenColorIO.so",
            "libOpenColorIO.dylib",
            "OpenColorIO.dll",
        ];

        let lib = lib_names.iter().find_map(|name| {
            debug!("Trying to load OpenColorIO library: {}", name);
            unsafe { libloading::Library::new(*name) }.ok()
        })?;

        debug!("OpenColorIO library loaded successfully");

        unsafe {
            let config_create_from_file: FnConfigCreateFromFile = *lib.get(b"OCIO_configCreateFromFile\0").ok()?;
            let config_release: FnConfigRelease = *lib.get(b"OCIO_configRelease\0").ok()?;
            let config_get_processor: FnConfigGetProcessor = *lib.get(b"OCIO_configGetProcessor\0").ok()?;
            let processor_release: FnProcessorRelease = *lib.get(b"OCIO_processorRelease\0").ok()?;
            let processor_get_cpu_processor: FnProcessorGetCPUProcessor = *lib.get(b"OCIO_processorGetDefaultCPUProcessor\0").ok()?;
            let cpu_processor_release: FnCpuProcessorRelease = *lib.get(b"OCIO_cpuProcessorRelease\0").ok()?;
            let cpu_processor_apply_rgb: FnCpuProcessorApplyRGB = *lib.get(b"OCIO_cpuProcessorApplyRGB\0").ok()?;

            Some(Self {
                _lib: lib,
                config_create_from_file,
                config_release,
                config_get_processor,
                processor_release,
                processor_get_cpu_processor,
                cpu_processor_release,
                cpu_processor_apply_rgb,
            })
        }
    }
}

pub struct OcioConfig {
    lib: std::sync::Arc<OcioLib>,
    inner: OCIO_Config,
}

impl OcioConfig {
    pub fn from_file(lib: std::sync::Arc<OcioLib>, path: &str) -> Result<Self> {
        let c_path = CString::new(path)?;
        let config = unsafe { (lib.config_create_from_file)(c_path.as_ptr()) };
        if config.is_null() {
            return Err(anyhow::anyhow!("Failed to load OCIO config from: {}", path));
        }
        debug!("OCIO config loaded from: {}", path);
        Ok(Self { lib, inner: config })
    }

    pub fn get_processor(&self, src: &str, dst: &str) -> Result<OcioProcessor> {
        let src_c = CString::new(src)?;
        let dst_c = CString::new(dst)?;
        let processor = unsafe {
            (self.lib.config_get_processor)(self.inner, src_c.as_ptr(), dst_c.as_ptr())
        };
        if processor.is_null() {
            return Err(anyhow::anyhow!(
                "Failed to create OCIO processor: {} -> {}",
                src,
                dst
            ));
        }
        debug!("OCIO processor created: {} -> {}", src, dst);
        Ok(OcioProcessor {
            lib: self.lib.clone(),
            inner: processor,
        })
    }
}

impl Drop for OcioConfig {
    fn drop(&mut self) {
        unsafe {
            (self.lib.config_release)(self.inner);
        }
    }
}

pub struct OcioProcessor {
    lib: std::sync::Arc<OcioLib>,
    inner: OCIO_Processor,
}

impl OcioProcessor {
    pub fn get_cpu_processor(&self) -> Result<OcioCpuProcessor> {
        let cpu = unsafe { (self.lib.processor_get_cpu_processor)(self.inner) };
        if cpu.is_null() {
            return Err(anyhow::anyhow!("Failed to create OCIO CPU processor"));
        }
        Ok(OcioCpuProcessor {
            lib: self.lib.clone(),
            inner: cpu,
        })
    }
}

impl Drop for OcioProcessor {
    fn drop(&mut self) {
        unsafe {
            (self.lib.processor_release)(self.inner);
        }
    }
}

pub struct OcioCpuProcessor {
    lib: std::sync::Arc<OcioLib>,
    inner: OCIO_CPUProcessor,
}

impl OcioCpuProcessor {
    pub fn apply_rgb(&self, rgb: &mut [f32; 3]) {
        unsafe {
            (self.lib.cpu_processor_apply_rgb)(self.inner, rgb.as_mut_ptr());
        }
    }
}

impl Drop for OcioCpuProcessor {
    fn drop(&mut self) {
        unsafe {
            (self.lib.cpu_processor_release)(self.inner);
        }
    }
}

/// High-level OCIO color pipeline manager
pub struct OcioPipeline {
    config: OcioConfig,
}

impl OcioPipeline {
    /// Attempt to initialize the OCIO pipeline from a config file
    pub fn new(config_path: &str) -> Option<Self> {
        let lib = OcioLib::load()?;
        let lib = std::sync::Arc::new(lib);
        let config = OcioConfig::from_file(lib, config_path).ok()?;
        info!("OpenColorIO pipeline initialized from: {}", config_path);
        Some(Self { config })
    }

    /// Convert a single RGB pixel from source to destination color space
    pub fn transform_pixel(&self, src_space: &str, dst_space: &str, rgb: &mut [f32; 3]) -> Result<()> {
        let processor = self.config.get_processor(src_space, dst_space)?;
        let cpu = processor.get_cpu_processor()?;
        cpu.apply_rgb(rgb);
        Ok(())
    }
}

/// Global OCIO runtime availability check
pub fn is_ocio_available() -> bool {
    OcioLib::load().is_some()
}
