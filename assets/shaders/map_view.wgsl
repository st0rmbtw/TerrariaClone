#import bevy_pbr::mesh_vertex_output MeshVertexOutput

@group(1) @binding(0)
var<uniform> tile_map_texure: texture_2d<f32>

@group(1) @binding(1)
var<uniform> tile_map_texure_sampler: sampler

@fragment
fn fragment(
    mesh: MeshVertexOutput,
) -> @location(0) vec4<f32> {
    let uv = mesh.uv;

    return textureSampleLevel(tile_map_texure, tile_map_texure_sampler, uv, 0.);
}