use super::types::RendererError;

/// Stub: CUDA native acceleration is not implemented.
/// Use wgpu compute for cross-platform GPU post-processing on NVIDIA GPUs.
pub(super) fn initialize_cuda() -> Result<(), RendererError> {
    Err(RendererError::HardwareAccelerationError(
        "CUDA native acceleration not yet implemented — use wgpu compute instead".to_string(),
    ))
}
