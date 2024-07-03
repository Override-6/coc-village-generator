@vertex
fn vs_main(@builtin(vertex_index) vertex_index : u32) -> @builtin(position) vec4<f32> {
    // Define vertices for a full-screen quad
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0)
    );

    var position = positions[vertex_index];

    // Pass the UV coordinates to the fragment shader
    var uv = (position + vec2<f32>(1.0, 1.0)) * 0.5;

    // Return the position in clip space and UV coordinates
    return vec4<f32>(position, 0.0, 1.0);
}

// Fragment Shader
@group(0) @binding(0) var texture1: texture_2d<f32>;
@group(0) @binding(1) var generator_sampler: sampler;
@group(0) @binding(2) var texture2: texture_2d<f32>;

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    var color1 = textureSample(texture1, generator_sampler, uv);
    var color2 = textureSample(texture2, generator_sampler, uv);
    return mix(color1, color2, color2.a);
}