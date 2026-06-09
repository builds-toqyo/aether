@group(0) @binding(0) var<storage, read> input_image: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> blend_image: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> output_image: array<vec4<f32>>;
@group(0) @binding(3) var<uniform> blend_params: BlendParams;

struct BlendParams {
    blend_mode: u32,
    opacity: f32,
    image_size: vec2<u32>,
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= blend_params.image_size.x || y >= blend_params.image_size.y) {
        return;
    }
    
    let index = y * blend_params.image_size.x + x;
    let input_color = input_image[index];
    let blend_color = blend_image[index];
    
    var result: vec4<f32>;
    
    switch (blend_params.blend_mode) {
        case 0: { // Normal
            result = mix(input_color, blend_color, blend_params.opacity);
        }
        case 1: { // Multiply
            result = mix(input_color, input_color * blend_color, blend_params.opacity);
        }
        case 2: { // Screen
            result = mix(input_color, vec4<f32>(1.0) - (vec4<f32>(1.0) - input_color) * (vec4<f32>(1.0) - blend_color), blend_params.opacity);
        }
        case 3: { // Overlay
            let overlay = vec4<f32>(
                select(2.0 * input_color.r * blend_color.r, 1.0 - 2.0 * (1.0 - input_color.r) * (1.0 - blend_color.r), input_color.r < 0.5),
                select(2.0 * input_color.g * blend_color.g, 1.0 - 2.0 * (1.0 - input_color.g) * (1.0 - blend_color.g), input_color.g < 0.5),
                select(2.0 * input_color.b * blend_color.b, 1.0 - 2.0 * (1.0 - input_color.b) * (1.0 - blend_color.b), input_color.b < 0.5),
                blend_color.a
            );
            result = mix(input_color, overlay, blend_params.opacity);
        }
        default: {
            result = input_color;
        }
    }
    
    output_image[index] = result;
}
