pub(super) const COMPUTE_SHADER: &str = r#"
struct Params {
    width: u32,
    height: u32,
    gamma: f32,
    saturation: f32,
    contrast: f32,
    brightness: f32,
    temp_r: f32,
    temp_g: f32,
    temp_b: f32,
    vignette: f32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read> input_buffer: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<u32>;
@group(0) @binding(2) var<uniform> params: Params;

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> vec3<f32> {
    let c = (1.0 - abs(2.0 * l - 1.0)) * s;
    let h6 = h * 6.0;
    let x = c * (1.0 - abs(h6 % 2.0 - 1.0));
    let m = l - c / 2.0;
    var rgb = vec3<f32>(0.0);
    if h6 < 1.0 { rgb = vec3<f32>(c, x, 0.0); }
    else if h6 < 2.0 { rgb = vec3<f32>(x, c, 0.0); }
    else if h6 < 3.0 { rgb = vec3<f32>(0.0, c, x); }
    else if h6 < 4.0 { rgb = vec3<f32>(0.0, x, c); }
    else if h6 < 5.0 { rgb = vec3<f32>(x, 0.0, c); }
    else { rgb = vec3<f32>(c, 0.0, x); }
    return rgb + vec3<f32>(m);
}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> vec3<f32> {
    let maxc = max(max(r, g), b);
    let minc = min(min(r, g), b);
    let delta = maxc - minc;
    let l = (maxc + minc) / 2.0;
    var s = 0.0;
    if delta != 0.0 {
        s = delta / (1.0 - abs(2.0 * l - 1.0));
    }
    var h = 0.0;
    if delta != 0.0 {
        if maxc == r { h = ((g - b) / delta) % 6.0; }
        else if maxc == g { h = ((b - r) / delta) + 2.0; }
        else { h = ((r - g) / delta) + 4.0; }
    }
    if h < 0.0 { h = h + 6.0; }
    return vec3<f32>(h / 6.0, s, l);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if x >= params.width || y >= params.height { return; }

    let idx = (y * params.width + x) * 4u;

    var r = f32(input_buffer[idx]) / 255.0;
    var g = f32(input_buffer[idx + 1u]) / 255.0;
    var b = f32(input_buffer[idx + 2u]) / 255.0;
    let a = f32(input_buffer[idx + 3u]) / 255.0;

    // Gamma correction
    let gamma_inv = 1.0 / params.gamma;
    r = pow(r, gamma_inv);
    g = pow(g, gamma_inv);
    b = pow(b, gamma_inv);

    // Contrast
    r = ((r - 0.5) * params.contrast + 0.5);
    g = ((g - 0.5) * params.contrast + 0.5);
    b = ((b - 0.5) * params.contrast + 0.5);

    // Brightness
    r = r * params.brightness;
    g = g * params.brightness;
    b = b * params.brightness;

    // Saturation
    let hsl = rgb_to_hsl(r, g, b);
    let rgb = hsl_to_rgb(hsl.x, hsl.y * params.saturation, hsl.z);
    r = rgb.x; g = rgb.y; b = rgb.z;

    // Color temperature
    r = r * params.temp_r;
    g = g * params.temp_g;
    b = b * params.temp_b;

    // Vignette
    let cx = f32(params.width) * 0.5;
    let cy = f32(params.height) * 0.5;
    let dx = (f32(x) - cx) / cx;
    let dy = (f32(y) - cy) / cy;
    let dist = sqrt(dx * dx + dy * dy);
    let vignette_factor = 1.0 - dist * dist * params.vignette;
    r = r * vignette_factor;
    g = g * vignette_factor;
    b = b * vignette_factor;

    // Clamp and write
    output_buffer[idx] = u32(clamp(r * 255.0, 0.0, 255.0));
    output_buffer[idx + 1u] = u32(clamp(g * 255.0, 0.0, 255.0));
    output_buffer[idx + 2u] = u32(clamp(b * 255.0, 0.0, 255.0));
    output_buffer[idx + 3u] = u32(clamp(a * 255.0, 0.0, 255.0));
}
"#;
