#import bevy_ecs_tilemap::common process_fragment
#import bevy_ecs_tilemap::vertex_output MeshVertexOutput

@group(3) @binding(0)
var light_map_texture: texture_2d<f32>;

@group(3) @binding(1)
var light_map_texture_sampler: sampler;

@group(3) @binding(2)
var<uniform> chunk_pos: vec2<u32>;

// https://stackoverflow.com/a/892640
fn hashUint2(value: vec2<u32>) -> u32
{
    var hash = 23u;
    hash = hash * 31u + value.x;
    hash = hash * 31u + value.y;
    return hash;
}

// https://stackoverflow.com/a/3887197
fn hashRemap(hash: u32) -> u32
{
    var h = (hash << 15u) ^ 0xffffcd7du;
    h ^= (h >> 10u);
    h += (h << 3u);
    h ^= (h >> 6u);
    h += (h << 2u) + (h << 14u);
    return (h ^ (h >> 16u));
}

fn uintToColor(x: u32) -> vec3<f32>
{
    return vec3(
        f32((x >> 0u) & 0xffu) / 255.0,
        f32((x >> 8u) & 0xffu) / 255.0,
        f32((x >> 16u) & 0xffu) / 255.0
    );
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let tile_chunk_pos: vec2<u32> = vec2(in.storage_position.x, 25u - in.storage_position.y);
    let tile_map_pos = (chunk_pos * 25u) + tile_chunk_pos;

    let light = textureSample(light_map_texture, light_map_texture_sampler, vec2<f32>(tile_map_pos) / vec2(1749., 901.));

    let color = process_fragment(in);

    return color * light;
}
