// Basic compute shaders for Aether node system
// This file contains fundamental operations for image processing

// Common binding layout for basic operations
@group(0) @binding(0) var<storage, read> input_image: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> output_image: array<vec4<f32>>;
@group(0) @binding(2) var<uniform> params: BasicParams;

struct BasicParams {
    operation_type: u32,
    image_size: vec2<u32>,
    blend_mode: u32,
    opacity: f32,
    transform_type: u32,
    translation: vec2<f32>,
    rotation: f32,
    scale: vec2<f32>,
    color_space: u32,
    gamma: f32,
};

fn blend_normal(base: vec4<f32>, overlay: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, overlay, opacity);
}

fn blend_multiply(base: vec4<f32>, overlay: vec4<f32>, opacity: f32) -> vec4<f32> {
    let result = base * overlay;
    return mix(base, result, opacity);
}

fn blend_screen(base: vec4<f32>, overlay: vec4<f32>, opacity: f32) -> vec4<f32> {
    let result = vec4<f32>(1.0) - (vec4<f32>(1.0) - base) * (vec4<f32>(1.0) - overlay);
    return mix(base, result, opacity);
}

fn blend_overlay(base: vec4<f32>, overlay: vec4<f32>, opacity: f32) -> vec4<f32> {
    let overlay_result = vec4<f32>(
        select(2.0 * base.r * overlay.r, 1.0 - 2.0 * (1.0 - base.r) * (1.0 - overlay.r), base.r < 0.5),
        select(2.0 * base.g * overlay.g, 1.0 - 2.0 * (1.0 - base.g) * (1.0 - overlay.g), base.g < 0.5),
        select(2.0 * base.b * overlay.b, 1.0 - 2.0 * (1.0 - base.b) * (1.0 - overlay.b), base.b < 0.5),
        overlay.a
    );
    return mix(base, overlay_result, opacity);
}

fn apply_blend(base: vec4<f32>, overlay: vec4<f32>, blend_mode: u32, opacity: f32) -> vec4<f32> {
    switch (blend_mode) {
        case 0: { return blend_normal(base, overlay, opacity); }
        case 1: { return blend_multiply(base, overlay, opacity); }
        case 2: { return blend_screen(base, overlay, opacity); }
        case 3: { return blend_overlay(base, overlay, opacity); }
        default: { return base; }
    }
}

fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let r = rgb.r;
    let g = rgb.g;
    let b = rgb.b;
    
    let cmax = max(max(r, g), b);
    let cmin = min(min(r, g), b);
    let delta = cmax - cmin;
    
    var h: f32 = 0.0;
    var s: f32 = 0.0;
    let v = cmax;
    
    if (delta > 0.0) {
        s = delta / cmax;
        
        if (cmax == r) {
            h = (g - b) / delta;
            if (g < b) { h = h + 6.0; }
        } else if (cmax == g) {
            h = (b - r) / delta + 2.0;
        } else {
            h = (r - g) / delta + 4.0;
        }
        
        h = h / 6.0;
    }
    
    return vec3<f32>(h, s, v);
}

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = hsv.r * 6.0;
    let s = hsv.g;
    let v = hsv.b;
    
    let c = v * s;
    let x = c * (1.0 - abs((h % 2.0) - 1.0));
    let m = v - c;
    
    var rgb: vec3<f32>;
    
    if (h < 1.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h < 2.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h < 3.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h < 4.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h < 5.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }
    
    return rgb + vec3<f32>(m, m, m);
}

fn rgb_to_ycbcr(rgb: vec3<f32>) -> vec3<f32> {
    let y = 0.299 * rgb.r + 0.587 * rgb.g + 0.114 * rgb.b;
    let cb = -0.168736 * rgb.r - 0.331264 * rgb.g + 0.5 * rgb.b;
    let cr = 0.5 * rgb.r - 0.418688 * rgb.g - 0.081312 * rgb.b;
    
    return vec3<f32>(y, cb, cr);
}

fn ycbcr_to_rgb(ycbcr: vec3<f32>) -> vec3<f32> {
    let y = ycbcr.r;
    let cb = ycbcr.g;
    let cr = ycbcr.b;
    
    let r = y + 1.402 * cr;
    let g = y - 0.344136 * cb - 0.714136 * cr;
    let b = y + 1.772 * cb;
    
    return vec3<f32>(r, g, b);
}

fn apply_gamma(color: vec3<f32>, gamma: f32) -> vec3<f32> {
    return pow(color, vec3<f32>(1.0 / gamma));
}

fn remove_gamma(color: vec3<f32>, gamma: f32) -> vec3<f32> {
    return pow(color, vec3<f32>(gamma));
}

fn apply_color_conversion(color: vec3<f32>, color_space: u32, gamma: f32) -> vec3<f32> {
    switch (color_space) {
        case 0: { return rgb_to_hsv(color); }
        case 1: { return hsv_to_rgb(color); }
        case 2: { return rgb_to_ycbcr(color); }
        case 3: { return ycbcr_to_rgb(color); }
        case 4: { return apply_gamma(color, gamma); }
        case 5: { return remove_gamma(color, gamma); }
        default: { return color; }
    }
}

fn apply_translation(coord: vec2<f32>, translation: vec2<f32>) -> vec2<f32> {
    return coord + translation;
}

fn apply_rotation(coord: vec2<f32>, rotation: f32) -> vec2<f32> {
    let cos_r = cos(rotation);
    let sin_r = sin(rotation);
    
    let rotated_x = coord.x * cos_r - coord.y * sin_r;
    let rotated_y = coord.x * sin_r + coord.y * cos_r;
    
    return vec2<f32>(rotated_x, rotated_y);
}

fn apply_scale(coord: vec2<f32>, scale: vec2<f32>) -> vec2<f32> {
    return coord * scale;
}

fn apply_transform(coord: vec2<f32>, transform_type: u32, translation: vec2<f32>, rotation: f32, scale: vec2<f32>) -> vec2<f32> {
    var transformed = coord;
    
    switch (transform_type) {
        case 0: { // Identity
            transformed = coord;
        }
        case 1: { // Translation only
            transformed = apply_translation(coord, translation);
        }
        case 2: { // Rotation only
            transformed = apply_rotation(coord, rotation);
        }
        case 3: { // Scale only
            transformed = apply_scale(coord, scale);
        }
        case 4: { // Combined: Scale -> Rotate -> Translate
            transformed = apply_scale(coord, scale);
            transformed = apply_rotation(transformed, rotation);
            transformed = apply_translation(transformed, translation);
        }
        default: {
            transformed = coord;
        }
    }
    
    return transformed;
}

fn bilinear_sample(image: array<vec4<f32>>, coord: vec2<f32>, size: vec2<u32>) -> vec4<f32> {
    let x = coord.x;
    let y = coord.y;
    
    // Clamp to image bounds
    let clamped_x = clamp(x, 0.0, f32(size.x - 1));
    let clamped_y = clamp(y, 0.0, f32(size.y - 1));
    
    let x0 = u32(clamped_x);
    let y0 = u32(clamped_y);
    let x1 = min(x0 + 1, size.x - 1);
    let y1 = min(y0 + 1, size.y - 1);
    
    let fx = clamped_x - f32(x0);
    let fy = clamped_y - f32(y0);
    
    let p00 = image[y0 * size.x + x0];
    let p10 = image[y0 * size.x + x1];
    let p01 = image[y1 * size.x + x0];
    let p11 = image[y1 * size.x + x1];
    
    let p0 = mix(p00, p10, fx);
    let p1 = mix(p01, p11, fx);
    
    return mix(p0, p1, fy);
}

// ===== MAIN COMPUTE FUNCTION =====

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= params.image_size.x || y >= params.image_size.y) {
        return;
    }
    
    let index = y * params.image_size.x + x;
    let input_color = input_image[index];
    var output_color = input_color;
    
    switch (params.operation_type) {
        case 0: { // Normal blend (pass-through)
            output_color = input_color;
        }
        case 1: { // Blend with second image (would need second binding)
            output_color = input_color;
        }
        
        case 10: { // RGB to HSV
            let converted = apply_color_conversion(input_color.rgb, 0, params.gamma);
            output_color = vec4<f32>(converted, input_color.a);
        }
        case 11: { // HSV to RGB
            let converted = apply_color_conversion(input_color.rgb, 1, params.gamma);
            output_color = vec4<f32>(converted, input_color.a);
        }
        case 12: { // RGB to YCbCr
            let converted = apply_color_conversion(input_color.rgb, 2, params.gamma);
            output_color = vec4<f32>(converted, input_color.a);
        }
        case 13: { // YCbCr to RGB
            let converted = apply_color_conversion(input_color.rgb, 3, params.gamma);
            output_color = vec4<f32>(converted, input_color.a);
        }
        case 14: { // Apply gamma
            let converted = apply_color_conversion(input_color.rgb, 4, params.gamma);
            output_color = vec4<f32>(converted, input_color.a);
        }
        case 15: { // Remove gamma
            let converted = apply_color_conversion(input_color.rgb, 5, params.gamma);
            output_color = vec4<f32>(converted, input_color.a);
        }
        
        case 20: { // Identity
            output_color = input_color;
        }
        case 21: { // Translation
            output_color = input_color;
        }
        case 22: { // Rotation
            output_color = input_color;
        }
        case 23: { // Scale
            output_color = input_color;
        }
        case 24: { // Combined transform
            output_color = input_color;
        }
        
        default: {
            output_color = input_color;
        }
    }
    
    output_image[index] = output_color;
}
