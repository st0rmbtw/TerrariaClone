#import bevy_core_pipeline::fullscreen_vertex_shader FullscreenVertexOutput

@group(1) @binding(0)
var background_texture: texture_2d<f32>;
@group(1) @binding(1)
var background_texture_sampler: sampler;

@group(1) @binding(2)
var ingame_background_texture: texture_2d<f32>;
@group(1) @binding(3)
var ingame_background_texture_sampler: sampler;

@group(1) @binding(4)
var world_texture: texture_2d<f32>;
@group(1) @binding(5)
var world_texture_sampler: sampler;

@group(1) @binding(6)
var main_texture: texture_2d<f32>;
@group(1) @binding(7)
var main_texture_sampler: sampler;


fn layer(foreground: vec4<f32>, background: vec4<f32>) -> vec4<f32> {
    return foreground * foreground.a + background * (1.0 - foreground.a);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let main_sample: vec4<f32> = textureSample(main_texture, main_texture_sampler, in.uv);
    var world_sample: vec4<f32> = textureSample(world_texture, world_texture_sampler, in.uv);
    let background_sample: vec4<f32> = textureSample(background_texture, background_texture_sampler, in.uv);
    let ingame_background_sample: vec4<f32> = textureSample(ingame_background_texture, ingame_background_texture_sampler, in.uv);

    if world_sample.r == 0. && world_sample.g == 0. && world_sample.b == 0. && world_sample.a < 1. {
        world_sample.a = 0.;
    }

    let color = layer(main_sample, layer(world_sample, layer(ingame_background_sample, background_sample)));

    return color;
}