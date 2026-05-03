@group(0) @binding(0) var<storage, read> input_image: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> output_image: array<vec4<f32>>;
@group(0) @binding(2) var<uniform> transform_params: TransformParams;

struct TransformParams {
    transform_type: u32,
    image_size: vec2<u32>,
    output_size: vec2<u32>,
    // Translation
    translation: vec2<f32>,
    // Rotation (in radians)
    rotation: f32,
    // Scale
    scale: vec2<f32>,
    // Shear
    shear: vec2<f32>,
};

fn apply_identity(coord: vec2<f32>, params: TransformParams) -> vec2<f32> {
    return coord;
}

fn apply_translation(coord: vec2<f32>, params: TransformParams) -> vec2<f32> {
    return coord + params.translation;
}

fn apply_rotation(coord: vec2<f32>, params: TransformParams) -> vec2<f32> {
    let cos_r = cos(params.rotation);
    let sin_r = sin(params.rotation);
    
    let rotated_x = coord.x * cos_r - coord.y * sin_r;
    let rotated_y = coord.x * sin_r + coord.y * cos_r;
    
    return vec2<f32>(rotated_x, rotated_y);
}

fn apply_scale(coord: vec2<f32>, params: TransformParams) -> vec2<f32> {
    return coord * params.scale;
}

fn apply_shear(coord: vec2<f32>, params: TransformParams) -> vec2<f32> {
    let sheared_x = coord.x + params.shear.x * coord.y;
    let sheared_y = coord.y + params.shear.y * coord.x;
    
    return vec2<f32>(sheared_x, sheared_y);
}

fn apply_transform(coord: vec2<f32>, params: TransformParams) -> vec2<f32> {
    var transformed = coord;
    
    // Apply transformations in order: scale -> shear -> rotate -> translate
    transformed = apply_scale(transformed, params);
    transformed = apply_shear(transformed, params);
    transformed = apply_rotation(transformed, params);
    transformed = apply_translation(transformed, params);
    
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

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= transform_params.output_size.x || y >= transform_params.output_size.y) {
        return;
    }
    
    let output_index = y * transform_params.output_size.x + x;
    
    // Calculate source coordinate for this output pixel
    var source_coord = vec2<f32>(f32(x), f32(y));
    
    // Normalize to center of image
    let center = vec2<f32>(f32(transform_params.image_size.x), f32(transform_params.image_size.y)) * 0.5;
    source_coord = source_coord - center;
    
    // Apply transform
    switch (transform_params.transform_type) {
        case 0: { // Identity
            source_coord = apply_identity(source_coord, transform_params);
        }
        case 1: { // Translation
            source_coord = apply_translation(source_coord, transform_params);
        }
        case 2: { // Rotation
            source_coord = apply_rotation(source_coord, transform_params);
        }
        case 3: { // Scale
            source_coord = apply_scale(source_coord, transform_params);
        }
        case 4: { // Shear
            source_coord = apply_shear(source_coord, transform_params);
        }
        case 5: { // Combined transform
            source_coord = apply_transform(source_coord, transform_params);
        }
        default: {
            source_coord = apply_identity(source_coord, transform_params);
        }
    }
    
    // Convert back to image coordinates
    source_coord = source_coord + center;
    
    // Sample from input image
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    
    // Check if source coordinate is within bounds
    if (source_coord.x >= 0.0 && source_coord.x < f32(transform_params.image_size.x) &&
        source_coord.y >= 0.0 && source_coord.y < f32(transform_params.image_size.y)) {
        color = bilinear_sample(input_image, source_coord, transform_params.image_size);
    }
    
    output_image[output_index] = color;
}
