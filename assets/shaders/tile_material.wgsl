#import bevy_sprite::mesh2d_vertex_output MeshVertexOutput

@group(1) @binding(0)
var light_map_texture: texture_2d<f32>;

@group(1) @binding(1)
var light_map_texture_sampler: sampler;

@group(1) @binding(2)
var<uniform> chunk_pos: vec2<u32>;

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    var tile_chunk_pos = in.uv * 25. * f32(#SUBDIVISION);
    let tile_map_pos = vec2<f32>(chunk_pos) * 25. * f32(#SUBDIVISION) + tile_chunk_pos;

    let c = tile_map_pos / vec2<f32>(textureDimensions(light_map_texture));

    let light = textureSample(light_map_texture, light_map_texture_sampler, c).r;

    return vec4(vec3(0.), 1. - light);
}
