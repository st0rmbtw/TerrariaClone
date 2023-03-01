#import game::gi_camera

struct LightMap {
    width: u32,
    colors: array<f32>
}

@group(0) @binding(0) var<uniform> camera_params: CameraParams;
@group(0) @binding(1) var<storage> light_map: LightMap;
@group(0) @binding(2) var light_map_texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let tile_xy = vec2<i32>(invocation_id.xy);

    let index = (tile_xy.y * i32(light_map.width)) + tile_xy.x;

    let tint = light_map.colors[index];

    var color = vec4(tint, tint, tint, 1.);

    textureStore(light_map_texture, tile_xy, color);
}