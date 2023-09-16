@group(0) @binding(0)
var tiles_texture: texture_storage_2d<r8uint, read>;

@group(0) @binding(1)
var light_texture: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(2)
var<uniform> min: vec2<u32>;

@group(0) @binding(3)
var<uniform> max: vec2<u32>;

@group(0) @binding(4)
var<uniform> decay_solid: f32;

@group(0) @binding(5)
var<uniform> decay_air: f32;

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

const EPSILON: f32 = 0.00185000002384186;

fn get_decay(pos: vec2<u32>) -> f32 {
    let tile = textureLoad(tiles_texture, pos / u32(#SUBDIVISION)).r;
    let n1 = textureLoad(tiles_texture, (pos + vec2(0u, 1u)) / u32(#SUBDIVISION)).r;
    let n2 = textureLoad(tiles_texture, (pos + vec2(1u, 0u)) / u32(#SUBDIVISION)).r;
    let n3 = textureLoad(tiles_texture, (pos - vec2(0u, 1u)) / u32(#SUBDIVISION)).r;
    let n4 = textureLoad(tiles_texture, (pos - vec2(1u, 0u)) / u32(#SUBDIVISION)).r;

    let n_light = n1 * n2 * n3 * n4;
    if n_light == 0u {
        return 1.;
    }

    var decay = 0.;

    if tile == 1u {
        decay = DECAY_THROUGH_SOLID;
    } else {
        decay = DECAY_THROUGH_AIR;
    }

    return decay;
}

@compute @workgroup_size(1, 16, 1)
fn left_to_right(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let y = min.y + invocation_id.y;

    var prev_light = vec4(0.);
    var decay = 0.;

    var flag1 = false;
    var flag2 = false;
    var flag3 = false;
    var flag4 = false;

    for (var x = min.x; x < max.x; x += 1u) {
        let pos = vec2(x, y);
        var this_light = textureLoad(light_texture, pos);
        let is_not_air = textureLoad(tiles_texture, pos / u32(#SUBDIVISION)).r != 0u;

        if !is_not_air {
            decay = get_decay(pos);
            prev_light = this_light;
            continue;
        }

        if (prev_light.x < this_light.x) {
            prev_light.x = this_light.x;
            flag1 = false;
        } else if (!flag1) {
            if (prev_light.x < EPSILON) {
                flag1 = true;
            } else {
                this_light.x = prev_light.x;
            }
        }

        if (prev_light.y < this_light.y) {
            prev_light.y = this_light.y;
            flag2 = false;
        } else if (!flag2) {
            if (prev_light.y < EPSILON) {
                flag2 = true;
            } else {
                this_light.y = prev_light.y;
            }
        }

        if (prev_light.z < this_light.z) {
            prev_light.z = this_light.z;
            flag3 = false;
        } else if (!flag3) {
            if (prev_light.z < EPSILON) {
                flag3 = true;
            } else {
                this_light.z = prev_light.z;
            }
        }

        if !flag1 {
            prev_light.x *= decay;
        }

        if !flag2 {
            prev_light.y *= decay;
        }

        if !flag3 {
            prev_light.z *= decay;
        }

       textureStore(light_texture, pos, this_light);
       decay = get_decay(pos);
    }
}

@compute @workgroup_size(1, 16, 1)
fn right_to_left(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let y = min.y + invocation_id.y;

    var prev_light = vec4(0.);
    var decay = 0.;

    var flag1 = false;
    var flag2 = false;
    var flag3 = false;
    var flag4 = false;

    var is_prev_block = false;

    for (var x = max.x - 1u; x > min.x; x -= 1u) {
        let pos = vec2(x, y);
        var this_light = textureLoad(light_texture, pos);
        let is_not_air = textureLoad(tiles_texture, pos / u32(#SUBDIVISION)).r != 0u;

        if !is_not_air {
            decay = get_decay(pos);
            prev_light = this_light;
            continue;
        }

        if (prev_light.x < this_light.x) {
            prev_light.x = this_light.x;
            flag1 = false;
        } else if (!flag1) {
            if (prev_light.x < EPSILON) {
                flag1 = true;
            } else {
                this_light.x = prev_light.x;
            }
        }

        if (prev_light.y < this_light.y) {
            prev_light.y = this_light.y;
            flag2 = false;
        } else if (!flag2) {
            if (prev_light.y < EPSILON) {
                flag2 = true;
            } else {
                this_light.y = prev_light.y;
            }
        }

        if (prev_light.z < this_light.z) {
            prev_light.z = this_light.z;
            flag3 = false;
        } else if (!flag3) {
            if (prev_light.z < EPSILON) {
                flag3 = true;
            } else {
                this_light.z = prev_light.z;
            }
        }

        if !flag1 {
            prev_light.x *= decay;
        }

        if !flag2 {
            prev_light.y *= decay;
        }

        if !flag3 {
            prev_light.z *= decay;
        }

       textureStore(light_texture, pos, this_light);
       decay = get_decay(pos);
    }
}

@compute @workgroup_size(16, 1, 1)
fn top_to_bottom(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = min.x + invocation_id.x;

    var prev_light = vec4(0.);
    var decay = 0.;

    var flag1 = false;
    var flag2 = false;
    var flag3 = false;
    var flag4 = false;

    for (var y = min.y; y < max.y; y += 1u) {
        let pos = vec2(x, y);
        var this_light = textureLoad(light_texture, pos);
        let is_not_air = textureLoad(tiles_texture, pos / u32(#SUBDIVISION)).r != 0u;

        if !is_not_air {
            decay = get_decay(pos);
            prev_light = this_light;
            continue;
        }

        if (prev_light.x < this_light.x) {
            prev_light.x = this_light.x;
            flag1 = false;
        } else if (!flag1) {
            if (prev_light.x < EPSILON) {
                flag1 = true;
            } else {
                this_light.x = prev_light.x;
            }
        }

        if (prev_light.y < this_light.y) {
            prev_light.y = this_light.y;
            flag2 = false;
        } else if (!flag2) {
            if (prev_light.y < EPSILON) {
                flag2 = true;
            } else {
                this_light.y = prev_light.y;
            }
        }

        if (prev_light.z < this_light.z) {
            prev_light.z = this_light.z;
            flag3 = false;
        } else if (!flag3) {
            if (prev_light.z < EPSILON) {
                flag3 = true;
            } else {
                this_light.z = prev_light.z;
            }
        }

        if !flag1 {
            prev_light.x *= decay;
        }

        if !flag2 {
            prev_light.y *= decay;
        }

        if !flag3 {
            prev_light.z *= decay;
        }

       textureStore(light_texture, pos, this_light);
       decay = get_decay(pos);
    }
}

@compute @workgroup_size(16, 1, 1)
fn bottom_to_top(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = min.x + invocation_id.x;

    var prev_light = vec4(0.);
    var decay = 0.;

    var flag1 = false;
    var flag2 = false;
    var flag3 = false;
    var flag4 = false;

    for (var y = max.y - 1u; y > min.y; y -= 1u) {
        let pos = vec2(x, y);
        var this_light = textureLoad(light_texture, pos);
        let is_not_air = textureLoad(tiles_texture, pos / u32(#SUBDIVISION)).r != 0u;

        if !is_not_air {
            decay = get_decay(pos);
            prev_light = this_light;
            continue;
        }

        if (prev_light.x < this_light.x) {
            prev_light.x = this_light.x;
            flag1 = false;
        } else if (!flag1) {
            if (prev_light.x < EPSILON) {
                flag1 = true;
            } else {
                this_light.x = prev_light.x;
            }
        }

        if (prev_light.y < this_light.y) {
            prev_light.y = this_light.y;
            flag2 = false;
        } else if (!flag2) {
            if (prev_light.y < EPSILON) {
                flag2 = true;
            } else {
                this_light.y = prev_light.y;
            }
        }

        if (prev_light.z < this_light.z) {
            prev_light.z = this_light.z;
            flag3 = false;
        } else if (!flag3) {
            if (prev_light.z < EPSILON) {
                flag3 = true;
            } else {
                this_light.z = prev_light.z;
            }
        }

        if !flag1 {
            prev_light.x *= decay;
        }

        if !flag2 {
            prev_light.y *= decay;
        }

        if !flag3 {
            prev_light.z *= decay;
        }

       textureStore(light_texture, pos, this_light);
       decay = get_decay(pos);
    }
}