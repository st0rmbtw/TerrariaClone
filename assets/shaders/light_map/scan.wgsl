@group(0) @binding(0)
var tiles_texture: texture_storage_2d<r8uint, read>;

@group(0) @binding(1)
var texture: texture_storage_2d<r8unorm, write>;

@group(0) @binding(2)
var<uniform> min: vec2<u32>;

@compute @workgroup_size(8, 8, 1)
fn scan(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = min + invocation_id.xy;

    let tile = textureLoad(tiles_texture, pos / u32(#SUBDIVISION)).r;

    var light = 0.;

    if tile == 0u {
        light = 1.;
    } else {
        light = 0.;
    }

    textureStore(texture, pos, vec4(vec3(light), 1.));
}