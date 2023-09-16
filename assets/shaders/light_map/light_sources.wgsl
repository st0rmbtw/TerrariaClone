@group(0) @binding(0)
var light_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<storage> lights_source_buffer: LightSourceBuffer;

struct LightSource {
    pos: vec2<u32>,
    size: vec2<u32>,
    color: vec3<f32>
}

struct LightSourceBuffer {
    data: array<LightSource>,
}

@compute @workgroup_size(1, 1, 1)
fn light_sources(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let light = lights_source_buffer.data[invocation_id.x];
    let width = i32(light.size.x);
    let height = i32(light.size.y);
    
    for (var x: i32 = 0; x < width; x++) {
        for (var y: i32 = 0; y < height; y++) {
            textureStore(light_texture, vec2<i32>(light.pos) + vec2(x, y), vec4(light.color, 1.));
        }
    }
}