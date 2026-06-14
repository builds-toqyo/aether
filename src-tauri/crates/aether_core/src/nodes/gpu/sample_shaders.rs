/// Sample compute shaders for GPU node execution

/// Simple color grading compute shader
pub const COLOR_GRADING_SHADER: &str = r#"
@group(0) @binding(0)
var<storage, read> input_image: array<vec4<f32>>;

@group(0) @binding(1)
var<storage, read_write> output_image: array<vec4<f32>>;

@group(0) @binding(2)
var<uniform> params: ColorGradingParams;

struct ColorGradingParams {
    exposure: f32,
    contrast: f32,
    saturation: f32,
    brightness: f32,
    padding: f32,
}

@compute @workgroup_size(16, 16, 1)
fn color_grading(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let width = 1920u;
    let height = 1080u;
    
    if (global_id.x >= width || global_id.y >= height) {
        return;
    }
    
    let index = global_id.y * width + global_id.x;
    var color = input_image[index];
    
    // Apply exposure
    color = color * params.exposure;
    
    // Apply contrast
    color = (color - 0.5) * params.contrast + 0.5;
    
    // Apply brightness
    color = color + params.brightness;
    
    // Apply saturation
    let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    color.rgb = mix(vec3<f32>(gray), color.rgb, params.saturation);
    
    // Clamp to valid range
    color = clamp(color, vec4<f32>(0.0), vec4<f32>(1.0));
    
    output_image[index] = color;
}
"#;

/// Simple blur compute shader
pub const BLUR_SHADER: &str = r#"
@group(0) @binding(0)
var<storage, read> input_image: array<vec4<f32>>;

@group(0) @binding(1)
var<storage, read_write> output_image: array<vec4<f32>>;

@group(0) @binding(2)
var<uniform> blur_params: BlurParams;

struct BlurParams {
    radius: u32,
    width: u32,
    height: u32,
    padding: u32,
}

@compute @workgroup_size(16, 16, 1)
fn blur(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let width = blur_params.width;
    let height = blur_params.height;
    let radius = blur_params.radius;
    
    if (global_id.x >= width || global_id.y >= height) {
        return;
    }
    
    let index = global_id.y * width + global_id.x;
    var accum = vec4<f32>(0.0);
    var count = 0.0;
    
    // Simple box blur
    for (var dy = -i32(radius); dy <= i32(radius); dy++) {
        for (var dx = -i32(radius); dx <= i32(radius); dx++) {
            let nx = i32(global_id.x) + dx;
            let ny = i32(global_id.y) + dy;
            
            if (nx >= 0 && nx < i32(width) && ny >= 0 && ny < i32(height)) {
                let neighbor_index = u32(ny) * width + u32(nx);
                accum = accum + input_image[neighbor_index];
                count = count + 1.0;
            }
        }
    }
    
    output_image[index] = accum / count;
}
"#;

/// Simple noise generation compute shader
pub const NOISE_SHADER: &str = r#"
@group(0) @binding(0)
var<storage, read_write> output_image: array<vec4<f32>>;

@group(0) @binding(1)
var<uniform> noise_params: NoiseParams;

struct NoiseParams {
    seed: f32,
    scale: f32,
    width: u32,
    height: u32,
}

fn hash(p: vec2<f32>) -> f32 {
    let p2 = fract(p * vec2<f32>(123.34, 456.21));
    p2 += dot(p2, p2 + 34.56);
    return fract(p2.x * p2.y);
}

fn noise_2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

@compute @workgroup_size(16, 16, 1)
fn generate_noise(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let width = noise_params.width;
    let height = noise_params.height;
    
    if (global_id.x >= width || global_id.y >= height) {
        return;
    }
    
    let index = global_id.y * width + global_id.x;
    let uv = vec2<f32>(f32(global_id.x), f32(global_id.y)) / vec2<f32>(f32(width), f32(height));
    
    let n = noise_2d(uv * noise_params.scale + noise_params.seed);
    
    output_image[index] = vec4<f32>(n, n, n, 1.0);
}
"#;
