@group(0) @binding(0)
var tiles_texture: texture_storage_2d<r8uint, read>;

@group(0) @binding(1)
var light_texture: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(2)
var<uniform> min: vec2<u32>;

@group(0) @binding(3)
var<uniform> max: vec2<u32>;

#if LIGHT_SMOOTHNESS == 3
const DECAY_THROUGH_SOLID: f32 = 0.93;
const DECAY_THROUGH_AIR: f32 = 0.985;
#else if LIGHT_SMOOTHNESS == 2
const DECAY_THROUGH_SOLID: f32 = 0.86;
const DECAY_THROUGH_AIR: f32 = 0.975;
#else if LIGHT_SMOOTHNESS == 1
const DECAY_THROUGH_SOLID: f32 = 0.78;
const DECAY_THROUGH_AIR: f32 = 0.91;
#else
const DECAY_THROUGH_SOLID: f32 = 0.56;
const DECAY_THROUGH_AIR: f32 = 0.91;
#endif

@compute @workgroup_size(1, 16, 1)
fn left_to_right(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let y = min.y + invocation_id.y;

    var prev_light = vec3(0.);

    for (var x = min.x; x < max.x; x += 1u) {
        blur(vec2(x, y), &prev_light);
    }
}

@compute @workgroup_size(1, 16, 1)
fn right_to_left(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let y = min.y + invocation_id.y;

    var prev_light = vec3(0.);

    for (var x = max.x - 1u; x > min.x; x -= 1u) {
        blur(vec2(x, y), &prev_light);
    }
}

@compute @workgroup_size(16, 1, 1)
fn top_to_bottom(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = min.x + invocation_id.x;

    var prev_light = vec3(0.);

    for (var y = min.y; y < max.y; y += 1u) {
        blur(vec2(x, y), &prev_light);
    }
}

@compute @workgroup_size(16, 1, 1)
fn bottom_to_top(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = min.x + invocation_id.x;

    var prev_light = vec3(0.);

    for (var y = max.y - 1u; y > min.y; y -= 1u) {
        blur(vec2(x, y), &prev_light);
    }
}

fn get_decay(pos: vec2<u32>) -> f32 {
    let tile = textureLoad(tiles_texture, pos / u32(#SUBDIVISION)).r;

    if tile == 1u {
        return DECAY_THROUGH_SOLID;
    } else {
        return DECAY_THROUGH_AIR;
    }
}

fn blur(
    pos: vec2<u32>,
    prev_light_ptr: ptr<function, vec3<f32>>,
) {
    var this_light = textureLoad(light_texture, pos);

    var prev_light = *prev_light_ptr;

    if (prev_light.x < this_light.x) {
        prev_light.x = this_light.x;
    } else {
        this_light.x = prev_light.x;
    }

    if (prev_light.y < this_light.y) {
        prev_light.y = this_light.y;
    } else {
        this_light.y = prev_light.y;
    }

    if (prev_light.z < this_light.z) {
        prev_light.z = this_light.z;
    } else {
        this_light.z = prev_light.z;
    }

    prev_light *= get_decay(pos);

    textureStore(light_texture, pos, this_light);
    *prev_light_ptr = prev_light;
}