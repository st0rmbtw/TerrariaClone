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
    let pixelate = 10.;

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
        // in_irradiance_uv = (floor(position.xy / pixelate) * pixelate + 0.5) / size;
    }

    let in_irradiance = blur(in_irradiance_texture, in_irradiance_texture_sampler, view.viewport.zw, in_irradiance_uv).rgb;

    var object_irradiance = in_irradiance;
    let k_size = 1;
    let k_width = 10;

    // for (var i = -k_size; i <= k_size; i++) {
    //     for (var j = -k_size; j < 0; j++) {
    //         let offset = vec2<f32>(f32(i * k_width), f32(j * k_width));
    //         let irradiance_uv = coords_to_viewport_uv(position.xy - offset, view.viewport);

    //         let sample_irradiance = textureSample(
    //             in_irradiance_texture,
    //             in_irradiance_texture_sampler,
    //             irradiance_uv
    //         ).rgb;

    //         if any(irradiance_uv < vec2<f32>(0.0)) || any(irradiance_uv > vec2<f32>(1.0)) {
    //             continue;
    //         }

    //         object_irradiance = max(object_irradiance, sample_irradiance);
    //     }
    // }

    let object_irradiance_srgb = lin_to_srgb(object_irradiance);
    let light_map_color = blur(light_map_texture, light_map_texture_sampler, view.viewport.zw, light_map_uv + player_uv);
    // let light_map_color = textureSample(light_map_texture, light_map_texture_sampler, light_map_uv + player_uv);
    
    let color = texture_diffuse * (light_map_color + vec4(object_irradiance, 1.));

    return color;
}