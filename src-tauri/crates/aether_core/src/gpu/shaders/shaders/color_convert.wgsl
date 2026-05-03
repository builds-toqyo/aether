@group(0) @binding(0) var<storage, read> input_image: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> output_image: array<vec4<f32>>;
@group(0) @binding(2) var<uniform> convert_params: ConvertParams;

struct ConvertParams {
    conversion_type: u32,
    image_size: vec2<u32>,
    gamma: f32,
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

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= convert_params.image_size.x || y >= convert_params.image_size.y) {
        return;
    }
    
    let index = y * convert_params.image_size.x + x;
    var input_color = input_image[index].rgb;
    
    switch (convert_params.conversion_type) {
        case 0: { // RGB to HSV
            input_color = rgb_to_hsv(input_color);
        }
        case 1: { // HSV to RGB
            input_color = hsv_to_rgb(input_color);
        }
        case 2: { // RGB to YCbCr
            input_color = rgb_to_ycbcr(input_color);
        }
        case 3: { // YCbCr to RGB
            input_color = ycbcr_to_rgb(input_color);
        }
        case 4: { // Apply gamma
            input_color = apply_gamma(input_color, convert_params.gamma);
        }
        case 5: { // Remove gamma
            input_color = pow(input_color, vec3<f32>(convert_params.gamma));
        }
        default: {
            // No conversion
        }
    }
    
    output_image[index] = vec4<f32>(input_color, input_image[index].a);
}
