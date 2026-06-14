use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

use crate::state::AppState;
use aether_core::color::{
    analyze_histogram, analyze_vectorscope, analyze_waveform,
    histogram_to_map, vectorscope_to_list, waveform_to_list
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorScopeRequest {
    pub image_data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistogramResponse {
    pub luminance: Vec<u32>,
    pub red: Vec<u32>,
    pub green: Vec<u32>,
    pub blue: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorscopeResponse {
    pub uv_points: Vec<(f32, f32)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WaveformResponse {
    pub luminance_per_scanline: Vec<Vec<u8>>,
}

/// Analyze image data to generate histogram
#[tauri::command]
pub async fn color_analyze_histogram(
    request: ColorScopeRequest,
    _state: State<'_, AppState>,
) -> Result<HistogramResponse, String> {
    let histogram = analyze_histogram(&request.image_data, request.width, request.height);
    let map = histogram_to_map(&histogram);
    
    Ok(HistogramResponse {
        luminance: map.get("luminance").cloned().unwrap_or_default(),
        red: map.get("red").cloned().unwrap_or_default(),
        green: map.get("green").cloned().unwrap_or_default(),
        blue: map.get("blue").cloned().unwrap_or_default(),
    })
}

/// Analyze image data to generate vectorscope
#[tauri::command]
pub async fn color_analyze_vectorscope(
    request: ColorScopeRequest,
    _state: State<'_, AppState>,
) -> Result<VectorscopeResponse, String> {
    let vectorscope = analyze_vectorscope(&request.image_data, request.width, request.height);
    let uv_points = vectorscope_to_list(&vectorscope);
    
    Ok(VectorscopeResponse { uv_points })
}

/// Analyze image data to generate waveform
#[tauri::command]
pub async fn color_analyze_waveform(
    request: ColorScopeRequest,
    _state: State<'_, AppState>,
) -> Result<WaveformResponse, String> {
    let waveform = analyze_waveform(&request.image_data, request.width, request.height);
    let luminance_per_scanline = waveform_to_list(&waveform);
    
    Ok(WaveformResponse { luminance_per_scanline })
}
