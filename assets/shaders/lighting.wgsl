#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(0) @binding(1) var<uniform> colors : array<vec3>;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let x = position.x * 16.;
    let y = position.y * 16.;

    let color = colors[x * y];

    let uv = coords_to_viewport_uv(position.xy, view.viewport);

    return textureSample(texture, texture_sampler, uv) * vec4(color, 1.);
}