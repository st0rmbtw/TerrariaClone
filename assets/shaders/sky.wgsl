#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

var texture_size: vec2<f32>;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let u = texture_size.x;

    return textureSample(texture, texture_sampler, uv)
}