#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

// fn rand(co: vec4<f32>) -> f32 {
//     return fract(sin(dot(co, vec4(12.9898, 78.233, 1., 1.))) * 43758.5453);
// }

// let PHI: f32 = 1.61803398874989484820459;  // Φ = Golden Ratio   

// fn gold_noise(xy: vec2<f32>, seed: f32) -> f32 {
//     return fract(tan(distance(xy*PHI, xy)*seed)*xy.x);
// }

// fn rainbow(x: f32) -> vec3<f32> {
//     let level = floor(x * 6.0);
// 	let r = f32(level <= 2.0) + f32(level > 4.0) * 0.5;
// 	let g = max(1.0 - abs(level - 2.0) * 0.5, 0.0);
// 	let b = (1.0 - (level - 4.0) * 0.5) * f32(level >= 4.0);
// 	return vec3(r, g, b);
// }

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    return vec4(0.1, 0.1, 0.1, 1.) * textureSample(texture, texture_sampler, uv);
}