@group(0) @binding(0)
var light_texture: texture_storage_2d<r8unorm, write>;

@group(0) @binding(1)
var<storage> lights_source_buffer: LightSourceBuffer;

struct LightSource {
    pos: vec2<u32>,
    size: vec2<u32>,
}

struct LightSourceBuffer {
    count: u32,
    data: array<LightSource>,
}

@compute @workgroup_size(1, 1, 1)
fn light_sources(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    for (var i = 0; i < i32(lights_source_buffer.count); i++) {
        let light = lights_source_buffer.data[i];
        let width = i32(light.size.x);
        let height = i32(light.size.y);
        for (var x: i32 = -width / 2; x < height / 2; x++) {
            for (var y: i32 = -height / 2; y < height / 2; y++) {
                textureStore(light_texture, vec2<i32>(light.pos) + vec2(x, y), vec4(vec3(1.), 1.));
            }   
        }
    }
}