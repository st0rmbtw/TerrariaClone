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
var in_irradiance_texture: texture_2d<f32>;

@group(1) @binding(5) 
var in_irradiance_texture_sampler: sampler;

@group(1) @binding(6)
var<uniform> player_position: vec2<f32>;

@group(1) @binding(7)
var<uniform> camera_scale: f32;

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

fn lin_to_srgb(color: vec3<f32>) -> vec3<f32> {
    let x = color * 12.92;
    let y = 1.055 * pow(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(0.4166667)) - vec3<f32>(0.055);
    var clr = color;
    clr.x = select(x.x, y.x, (color.x < 0.0031308));
    clr.y = select(x.y, y.y, (color.y < 0.0031308));
    clr.z = select(x.z, y.z, (color.z < 0.0031308));
    return clr;
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    let player_uv = player_position.xy / (vec2(1750. * 16., 900. * 16.) - vec2(16., 32.));

    let texture_diffuse = textureSample(texture, texture_sampler, uv);
    
    var light_map_uv = vec2(0.);
    {
        let size = vec2(1750. * 16., 900. * 16.);

        let scl = view.viewport.zw / size;

        let scale = scale(scl * camera_scale);

        light_map_uv = (uv - 0.5) * scale;
    }

    var in_irradiance_uv = vec2(0.);
    {
        let size = vec2(view.viewport.z * 16., view.viewport.w * 16.);

        let scl = view.viewport.zw / size;

        let scale = scale(scl);

        in_irradiance_uv = uv * scale;
    }

    var in_irradiance = vec4(0.);

    if (in_irradiance_uv.x >= 0. && in_irradiance_uv.x <= 1.) && (in_irradiance_uv.y >= 0. && in_irradiance_uv.y <= 1.) {
        in_irradiance = blur(in_irradiance_texture, in_irradiance_texture_sampler, view.viewport.zw, in_irradiance_uv, 16.0, 3.0);
    }

    var light_map_color = vec4(1.);
    {
        let uv = light_map_uv + player_uv;
        // let color = textureSample(light_map_texture, light_map_texture_sampler, uv);
        if (uv.x >= -0.00025) && (uv.x <= 1.00025) && (uv.y >= 0.) && (uv.y <= 1.0015) {
            light_map_color = blur(light_map_texture, light_map_texture_sampler, view.viewport.zw, uv, 16.0, 3.0);
            // light_map_color = color;
        }
    }

    var color = texture_diffuse;

    if (uv.x >= 0.) && (uv.x <= 1.) && (uv.y >= 0.) && (uv.y <= 1.) {
        color = texture_diffuse * (light_map_color + in_irradiance);
    }

    return color;
}