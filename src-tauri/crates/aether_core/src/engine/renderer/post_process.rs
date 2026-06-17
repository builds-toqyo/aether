use super::types::{RendererError, PostProcessUniforms};
use super::gpu::WgpuComputeState;

pub(super) fn apply_post_processing_gpu(
    state: &WgpuComputeState,
    frame_data: &mut [u8],
    width: usize,
    height: usize,
) -> Result<(), RendererError> {
    let device = &state.device;
    let queue = &state.queue;
    let pipeline = &state.pipeline;
    let bind_group_layout = &state.bind_group_layout;
    let input_buffer = &state.input_buffer;
    let output_buffer = &state.output_buffer;
    let uniform_buffer = &state.uniform_buffer;
    let staging_buffer = &state.staging_buffer;

    let uniforms = PostProcessUniforms {
        width: width as u32,
        height: height as u32,
        gamma: 1.1,
        saturation: 1.1,
        contrast: 1.05,
        brightness: 1.0,
        temp_r: 1.05,
        temp_g: 1.0,
        temp_b: 0.95,
        vignette: 0.3,
        _pad: [0; 2],
    };

    queue.write_buffer(input_buffer, 0, frame_data);
    queue.write_buffer(uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("post_process_bind_group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: input_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: output_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: uniform_buffer.as_entire_binding() },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("post_process_encoder") });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("post_process_pass"),
            ..Default::default()
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(
            ((width as u32 + 7) / 8).max(1),
            ((height as u32 + 7) / 8).max(1),
            1,
        );
    }

    encoder.copy_buffer_to_buffer(output_buffer, 0, staging_buffer, 0, frame_data.len() as u64);
    queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| RendererError::RenderError(format!("GPU poll failed: {:?}", e)))?;
    rx.recv().map_err(|_| RendererError::RenderError("GPU map channel closed".to_string()))?
        .map_err(|e| RendererError::RenderError(format!("GPU map failed: {}", e)))?;

    let data = slice.get_mapped_range();
    frame_data.copy_from_slice(&data);
    drop(data);
    staging_buffer.unmap();

    Ok(())
}

pub(super) fn apply_post_processing_cpu(frame_data: &mut [u8], width: usize, height: usize) {
    apply_gamma_correction(frame_data, width, height);
    apply_color_grading(frame_data, width, height);
    apply_vignette(frame_data, width, height);
}

fn apply_gamma_correction(frame_data: &mut [u8], width: usize, height: usize) {
    let gamma = 1.1;
    let gamma_inv = 1.0 / gamma;
    let mut gamma_table = [0u8; 256];
    for i in 0..256 {
        let normalized = i as f32 / 255.0;
        let corrected = normalized.powf(gamma_inv);
        gamma_table[i] = (corrected * 255.0).clamp(0.0, 255.0) as u8;
    }
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            frame_data[idx] = gamma_table[frame_data[idx] as usize];
            frame_data[idx + 1] = gamma_table[frame_data[idx + 1] as usize];
            frame_data[idx + 2] = gamma_table[frame_data[idx + 2] as usize];
        }
    }
}

fn apply_color_grading(frame_data: &mut [u8], width: usize, height: usize) {
    let saturation = 1.1;
    let contrast = 1.05;
    let brightness = 1.0;
    let temp_r = 1.05;
    let temp_g = 1.0;
    let temp_b = 0.95;
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let mut r = frame_data[idx] as f32 / 255.0;
            let mut g = frame_data[idx + 1] as f32 / 255.0;
            let mut b = frame_data[idx + 2] as f32 / 255.0;
            r = ((r - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
            g = ((g - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
            b = ((b - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
            r = (r * brightness).clamp(0.0, 1.0);
            g = (g * brightness).clamp(0.0, 1.0);
            b = (b * brightness).clamp(0.0, 1.0);
            let (h, s, l) = rgb_to_hsl(r, g, b);
            let (r_new, g_new, b_new) = hsl_to_rgb(h, (s * saturation).clamp(0.0, 1.0), l);
            r = r_new; g = g_new; b = b_new;
            r = (r * temp_r).clamp(0.0, 1.0);
            g = (g * temp_g).clamp(0.0, 1.0);
            b = (b * temp_b).clamp(0.0, 1.0);
            frame_data[idx] = (r * 255.0) as u8;
            frame_data[idx + 1] = (g * 255.0) as u8;
            frame_data[idx + 2] = (b * 255.0) as u8;
        }
    }
}

fn apply_vignette(frame_data: &mut [u8], width: usize, height: usize) {
    let vignette_strength = 0.3;
    let vignette_radius = 0.75;
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let max_dist = (center_x.powi(2) + center_y.powi(2)).sqrt() * vignette_radius;
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx.powi(2) + dy.powi(2)).sqrt();
            let factor = if distance > max_dist {
                1.0 - vignette_strength
            } else {
                1.0 - vignette_strength * (distance / max_dist).powi(2)
            };
            frame_data[idx] = (frame_data[idx] as f32 * factor) as u8;
            frame_data[idx + 1] = (frame_data[idx + 1] as f32 * factor) as u8;
            frame_data[idx + 2] = (frame_data[idx + 2] as f32 * factor) as u8;
        }
    }
}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let l = (max + min) / 2.0;
    let s = if delta == 0.0 { 0.0 } else { delta / (1.0 - (2.0 * l - 1.0).abs()) };
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    (h / 360.0, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 { return (l, l, l); }
    let h = h * 360.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = if h < 60.0 { (c, x, 0.0) }
    else if h < 120.0 { (x, c, 0.0) }
    else if h < 180.0 { (0.0, c, x) }
    else if h < 240.0 { (0.0, x, c) }
    else if h < 300.0 { (x, 0.0, c) }
    else { (c, 0.0, x) };
    (r1 + m, g1 + m, b1 + m)
}
