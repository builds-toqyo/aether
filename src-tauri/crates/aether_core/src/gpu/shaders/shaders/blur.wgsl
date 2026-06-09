@group(0) @binding(0) var<storage, read> input_image: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> output_image: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> temp_image: array<vec4<f32>>;
@group(0) @binding(3) var<uniform> blur_params: BlurParams;

struct BlurParams {
    blur_type: u32,
    radius: f32,
    image_size: vec2<u32>,
    direction: vec2<f32>, // For directional blur
    sigma: f32, // For Gaussian blur
};

fn gaussian_weight(distance: f32, sigma: f32) -> f32 {
    return exp(-(distance * distance) / (2.0 * sigma * sigma)) / (2.0 * 3.14159265 * sigma * sigma);
}

fn box_blur_sample(image: array<vec4<f32>>, coord: vec2<f32>, size: vec2<u32>, radius: f32) -> vec4<f32> {
    var color = vec4<f32>(0.0);
    var weight_sum = 0.0;
    
    let radius_i = i32(radius);
    
    for (let dy = -radius_i; dy <= radius_i; dy = dy + 1) {
        for (let dx = -radius_i; dx <= radius_i; dx = dx + 1) {
            let sample_coord = coord + vec2<f32>(f32(dx), f32(dy));
            
            if (sample_coord.x >= 0.0 && sample_coord.x < f32(size.x) &&
                sample_coord.y >= 0.0 && sample_coord.y < f32(size.y)) {
                
                let x = u32(sample_coord.x);
                let y = u32(sample_coord.y);
                let index = y * size.x + x;
                
                color = color + image[index];
                weight_sum = weight_sum + 1.0;
            }
        }
    }
    
    return color / weight_sum;
}

fn gaussian_blur_sample(image: array<vec4<f32>>, coord: vec2<f32>, size: vec2<u32>, radius: f32, sigma: f32) -> vec4<f32> {
    var color = vec4<f32>(0.0);
    var weight_sum = 0.0;
    
    let radius_i = i32(radius);
    
    for (let dy = -radius_i; dy <= radius_i; dy = dy + 1) {
        for (let dx = -radius_i; dx <= radius_i; dx = dx + 1) {
            let sample_coord = coord + vec2<f32>(f32(dx), f32(dy));
            
            if (sample_coord.x >= 0.0 && sample_coord.x < f32(size.x) &&
                sample_coord.y >= 0.0 && sample_coord.y < f32(size.y)) {
                
                let distance = length(vec2<f32>(f32(dx), f32(dy)));
                let weight = gaussian_weight(distance, sigma);
                
                let x = u32(sample_coord.x);
                let y = u32(sample_coord.y);
                let index = y * size.x + x;
                
                color = color + image[index] * weight;
                weight_sum = weight_sum + weight;
            }
        }
    }
    
    return color / weight_sum;
}

fn directional_blur_sample(image: array<vec4<f32>>, coord: vec2<f32>, size: vec2<u32>, radius: f32, direction: vec2<f32>) -> vec4<f32> {
    var color = vec4<f32>(0.0);
    var weight_sum = 0.0;
    
    let normalized_dir = normalize(direction);
    let radius_i = i32(radius);
    
    for (let i = -radius_i; i <= radius_i; i = i + 1) {
        let offset = normalized_dir * f32(i);
        let sample_coord = coord + offset;
        
        if (sample_coord.x >= 0.0 && sample_coord.x < f32(size.x) &&
            sample_coord.y >= 0.0 && sample_coord.y < f32(size.y)) {
            
            let x = u32(sample_coord.x);
            let y = u32(sample_coord.y);
            let index = y * size.x + x;
            
            color = color + image[index];
            weight_sum = weight_sum + 1.0;
        }
    }
    
    return color / weight_sum;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= blur_params.image_size.x || y >= blur_params.image_size.y) {
        return;
    }
    
    let index = y * blur_params.image_size.x + x;
    let coord = vec2<f32>(f32(x), f32(y));
    
    var color = vec4<f32>(0.0);
    
    switch (blur_params.blur_type) {
        case 0: { // Box blur
            color = box_blur_sample(input_image, coord, blur_params.image_size, blur_params.radius);
        }
        case 1: { // Gaussian blur
            color = gaussian_blur_sample(input_image, coord, blur_params.image_size, blur_params.radius, blur_params.sigma);
        }
        case 2: { // Horizontal blur (first pass of separable Gaussian)
            let horizontal_dir = vec2<f32>(1.0, 0.0);
            color = directional_blur_sample(input_image, coord, blur_params.image_size, blur_params.radius, horizontal_dir);
        }
        case 3: { // Vertical blur (second pass of separable Gaussian)
            let vertical_dir = vec2<f32>(0.0, 1.0);
            color = directional_blur_sample(input_image, coord, blur_params.image_size, blur_params.radius, vertical_dir);
        }
        case 4: { // Motion blur
            color = directional_blur_sample(input_image, coord, blur_params.image_size, blur_params.radius, blur_params.direction);
        }
        default: {
            color = input_image[index];
        }
    }
    
    output_image[index] = color;
}
