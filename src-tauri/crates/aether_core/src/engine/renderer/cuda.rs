use super::types::RendererError;

/// CUDA native acceleration requires NVIDIA CUDA SDK integration.
/// For cross-platform GPU acceleration including NVIDIA GPUs, use wgpu compute instead.
/// wgpu provides CUDA backend on NVIDIA GPUs, Metal on macOS, and Vulkan on Linux/Windows.
pub(super) fn initialize_cuda() -> Result<(), RendererError> {
    log::warn!("CUDA native acceleration not fully implemented. Use wgpu compute for cross-platform GPU acceleration.");
    Err(RendererError::HardwareAccelerationError(
        "CUDA requires NVIDIA CUDA SDK integration. Use wgpu compute for cross-platform GPU acceleration (includes CUDA backend on NVIDIA GPUs).".to_string(),
    ))
}
