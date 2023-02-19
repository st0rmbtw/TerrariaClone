#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var light_map_texture: texture_2d<f32>;

@group(1) @binding(3)
var light_map_texture_sampler: sampler;

@group(1) @binding(4)
var<uniform> player_position: vec2<f32>;

@group(1) @binding(5)
var<uniform> proj: vec4<f32>;

fn map_value(x: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> vec2<f32> {
    return end.x + ((end.y - end.x) / (start.y - start.x)) * (x - start.x);
}

fn rotateUV(uv: vec2<f32>) -> vec2<f32> {
    let mid = 0.5;
    let cosAngle = 0.;
    let sinAngle = 1.;

    return vec2(
        cosAngle * (1. - uv.x - mid) + sinAngle * (uv.y - mid) + mid,
        cosAngle * (uv.y - mid) - sinAngle * (1. - uv.x - mid) + mid
    );
}

fn scale(scale: vec2<f32>) -> mat2x2<f32> {
    return mat2x2(scale.x, 0.0,
                  0.0, scale.y);
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uv = coords_to_viewport_uv(position.xy, view.viewport);
    var player_uv = player_position.xy / vec2(1750. * 16., 900. * 16.);

    let scl = view.viewport.zw / vec2(1750. * 32., 900. * 32.);

    let rotated_uv = rotateUV(uv);

    // var light_map_uv = ((rotated_uv) - 0.5) * 2. + 0.5;
    // light_map_uv = (light_map_uv - vec2(0.5)) * scale(scl) + vec2(0.5);
    // light_map_uv = (light_map_uv - vec2<f32>(0.5, 0.5)) * 2. + 0.5;
    // light_map_uv = vec2(light_map_uv.y - 0.3, light_map_uv.x - 0.57);

    return textureSample(texture, texture_sampler, uv) * textureSample(light_map_texture, light_map_texture_sampler, rotated_uv);
}