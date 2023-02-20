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
var<uniform> camera_scale: f32;

fn scale(scale: vec2<f32>) -> mat2x2<f32> {
    return mat2x2(scale.x, 0.0,
                  0.0, scale.y);
}

fn blur(texture: texture_2d<f32>, texture_sampler: sampler, resolution: vec2<f32>, uv: vec2<f32>) -> vec4<f32> {
    let Pi = 6.28318530718; // Pi*2
    
    // GAUSSIAN BLUR SETTINGS {{{
    let Directions = 16.; // BLUR DIRECTIONS (Default 16.0 - More is better but slower)
    let Quality = 5.; // BLUR QUALITY (Default 4.0 - More is better but slower)
    let Size = 2.; // BLUR SIZE (Radius)
    // GAUSSIAN BLUR SETTINGS }}}
   
    let Radius = Size/resolution;
    // Pixel colour
    var Color = textureSample(texture, texture_sampler, uv);
    
    // Blur calculations
    for (var d = 0.0; d < Pi; d += Pi / Directions) {
		for (var i = 1.0 / Quality; i <= 1.0; i += 1.0 / Quality) {
			Color += textureSample(texture, texture_sampler, uv+vec2(cos(d),sin(d))*Radius*i);
        }
    }
    
    // Output to screen
    Color /= Quality * Directions - 15.0;

    return Color;
}


@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uv = coords_to_viewport_uv(position.xy, view.viewport);
    let player_uv = player_position.xy / (vec2(1750. * 16., 900. * 16.) - vec2(16., 32.));

    let scl = view.viewport.zw / vec2(1750. * 16., 900. * 16.);

    let light_map_uv = (uv - 0.5) * scale(scl * camera_scale);

    return textureSample(texture, texture_sampler, uv) * blur(light_map_texture, light_map_texture_sampler, view.viewport.zw, light_map_uv + player_uv);
}