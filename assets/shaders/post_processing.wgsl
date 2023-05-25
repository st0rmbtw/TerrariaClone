#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils
#import game::math
#import bevy_core_pipeline::fullscreen_vertex_shader

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var light_map_texture: texture_2d<f32>;

@group(1) @binding(3)
var light_map_texture_sampler: sampler;

@group(1) @binding(4) 
var lighting_texture: texture_2d<f32>;

@group(1) @binding(5) 
var lighting_texture_sampler: sampler;

@group(1) @binding(6)
var<uniform> player_position: vec2<f32>;

@group(1) @binding(7)
var<uniform> camera_scale: f32;

@group(1) @binding(8)
var<uniform> world_size: vec2<f32>;

@group(1) @binding(9)
var<uniform> enabled: u32;

const Size = 1.5; // BLUR SIZE (Radius)
const Pi = 6.28318530718; // Pi*2

fn blur(texture: texture_2d<f32>, texture_sampler: sampler, resolution: vec2<f32>, uv: vec2<f32>, directions: f32, quality: f32) -> vec4<f32> {
   
    let Radius = Size / resolution;
    // Pixel colour
    var Color = textureSample(texture, texture_sampler, uv);
    
    // Blur calculations
    for (var d = 0.0; d < Pi; d += Pi / directions) {
		for (var i = 1.0 / quality; i <= 1.0; i += 1.0 / quality) {
			Color += textureSample(texture, texture_sampler, uv+vec2(cos(d),sin(d))*Radius*i);
        }
    }
    
    // Output to screen
    Color /= quality * directions - 15.0;

    return Color;
}

// #define BLUR

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    var color = textureSample(texture, texture_sampler, uv);

    if enabled == 0u {
        return color;
    }
    
    // World size in pixels
    let world_size_px = world_size * 16.;

    // Light map is shifted a bit without this offset, i have no idea why this is happening
    //                                                            ↓
    let player_uv = player_position.xy / (world_size_px - vec2(16., 16.));

    var light_map_uv = vec2(0.);
    {
        let scale = view.viewport.zw / world_size_px;

        light_map_uv = uv - 0.5;
        light_map_uv *= scale * camera_scale;
    }

    var lighting_uv = uv / 16.;

    var lighting_color = vec4(0.);
    var light_map_color = vec4(1.);

#ifdef BLUR
    if (lighting_uv.x >= 0. && in_irradiance_uv.x <= 1.) && (in_irradiance_uv.y >= 0. && in_irradiance_uv.y <= 1.) {
        lighting_color = blur(lighting_texture, lighting_texture_sampler, view.viewport.zw, lighting_uv, 16.0, 3.0);
    }
#else
    lighting_color = textureSample(lighting_texture, lighting_texture_sampler, lighting_uv);
#endif

    {
        let uv = light_map_uv + player_uv;
#ifdef BLUR
        if (uv.x >= -0.00025) && (uv.x <= 1.00025) && (uv.y >= 0.) && (uv.y <= 1.0015) {
            light_map_color = blur(light_map_texture, light_map_texture_sampler, view.viewport.zw, uv, 16.0, 3.0);
        }
#else
        light_map_color = textureSample(light_map_texture, light_map_texture_sampler, uv);
#endif
    }

    if (uv.x >= 0.) && (uv.x <= 1.) && (uv.y >= 0.) && (uv.y <= 1.) {
        color *= (light_map_color + lighting_color);
    }

    return color;
}
