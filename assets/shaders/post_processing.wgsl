#import bevy_core_pipeline::fullscreen_vertex_shader FullscreenVertexOutput

@group(0) @binding(0)
var background_texture: texture_2d<f32>;
@group(0) @binding(1)
var background_texture_sampler: sampler;

@group(0) @binding(2)
var ingame_background_texture: texture_2d<f32>;
@group(0) @binding(3)
var ingame_background_texture_sampler: sampler;

@group(0) @binding(4)
var world_texture: texture_2d<f32>;
@group(0) @binding(5)
var world_texture_sampler: sampler;

@group(0) @binding(8)
var lightmap_texture: texture_2d<f32>;
@group(0) @binding(9)
var lightmap_texture_sampler: sampler;

@group(0) @binding(10)
var<uniform> camera_params: CameraParams;

struct CameraParams {
    screen_size: vec2<f32>,
    screen_size_inv: vec2<f32>,
    inverse_view_proj: mat4x4<f32>,
}

fn screen_to_world(
    screen_pos: vec2<f32>,
    screen_size: vec2<f32>,
    inverse_view_proj: mat4x4<f32>,
    screen_size_inv: vec2<f32>
) -> vec2<f32> {
    return (inverse_view_proj * vec4<f32>(screen_to_ndc(screen_pos, screen_size, screen_size_inv), 0.0, 1.0)).xy;   
}

fn screen_to_ndc(
    screen_pos: vec2<f32>,
    screen_size: vec2<f32>,
    screen_size_inv: vec2<f32>
) -> vec2<f32> {
    let screen_pose_f32 = vec2<f32>(screen_pos.x, screen_size.y - screen_pos.y);
    return (screen_pose_f32 * screen_size_inv) * 2.0 - 1.0;
}

fn layer(foreground: vec4<f32>, background: vec4<f32>) -> vec4<f32> {
    return foreground * foreground.a + background * (1.0 - foreground.a);
}

const BRIGHTNESS: f32 = 1.0;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let world_pos = screen_to_world(
        in.uv * camera_params.screen_size,
        camera_params.screen_size,
        camera_params.inverse_view_proj,
        camera_params.screen_size_inv,
    );

    let light_uv = abs(world_pos) / (vec2(f32(#WORLD_WIDTH), f32(#WORLD_HEIGHT)) * 16.);

    var light = textureSampleLevel(lightmap_texture, lightmap_texture_sampler, light_uv, 0.0);
    light.r *= BRIGHTNESS;
    light.g *= BRIGHTNESS;
    light.b *= BRIGHTNESS;

    var world_sample: vec4<f32> = textureSampleLevel(world_texture, world_texture_sampler, in.uv, 0.0) * light;
    let background_sample: vec4<f32> = textureSampleLevel(background_texture, background_texture_sampler, in.uv, 0.0);
    let ingame_background_sample: vec4<f32> = textureSampleLevel(ingame_background_texture, ingame_background_texture_sampler, in.uv, 0.0) * light;

    let color = layer(world_sample, layer(ingame_background_sample, background_sample));

    return color;
}