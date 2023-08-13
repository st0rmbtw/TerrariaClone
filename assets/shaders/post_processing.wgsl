#import bevy_ecs_tilemap::common process_fragment
#import bevy_ecs_tilemap::vertex_output MeshVertexOutput

@group(1) @binding(0)
var<uniform> test: f32;

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let color = process_fragment(in);

    return color;
}
