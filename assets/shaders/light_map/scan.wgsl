@group(0) @binding(0)
var tiles_texture: texture_storage_2d<r8uint, read>;

@group(0) @binding(1)
var light_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var<uniform> min: vec2<u32>;

@group(0) @binding(3)
var<uniform> sky_color: vec4<f32>;

@compute @workgroup_size(16, 16, 1)
fn scan(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = min + invocation_id.xy;

    var light = vec3(0.);

    if pos.y < (u32(#WORLD_UNDERGROUND_LEVEL) * u32(#SUBDIVISION)) {
        let tile = textureLoad(tiles_texture, pos / u32(#SUBDIVISION)).r;

        if tile == 0u {
            light = sky_color.rgb;
        }
    }
    
    textureStore(light_texture, pos, vec4(light, 1.));
}