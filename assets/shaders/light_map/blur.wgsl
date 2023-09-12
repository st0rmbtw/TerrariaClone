@group(0) @binding(0)
var tiles_texture: texture_storage_2d<r8uint, read>;

@group(0) @binding(1)
var light_texture: texture_storage_2d<r8unorm, read_write>;

@group(0) @binding(2)
var<uniform> min: vec2<u32>;

@group(0) @binding(3)
var<uniform> max: vec2<u32>;

@group(0) @binding(4)
var<uniform> decay_solid: f32;

@group(0) @binding(5)
var<uniform> decay_air: f32;

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
        decay = decay_solid;
    } else {
        decay = decay_air;
    }

    return decay;
}

@compute @workgroup_size(1, 16, 1)
fn left_to_right(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let y = min.y + invocation_id.y;

    var prev_light = 0.;
    var decay = 0.;

    for (var x = min.x; x < max.x; x += 1u) {
        let pos = vec2(x, y);
        let this_light = textureLoad(light_texture, pos).r;

        if (prev_light - this_light) > EPSILON {
            let new_light = prev_light * decay;
            textureStore(light_texture, pos, vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        decay = get_decay(pos);
    }
}

@compute @workgroup_size(1, 16, 1)
fn right_to_left(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let y = min.y + invocation_id.y;

    var prev_light = 0.;
    var decay = 0.;

    for (var x = max.x - 1u; x > min.x; x -= 1u) {
        let pos = vec2(x, y);
        let this_light = textureLoad(light_texture, pos).r;

        if (prev_light - this_light) > EPSILON {
            let new_light = prev_light * decay;
            textureStore(light_texture, pos, vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        decay = get_decay(pos);
    }
}

@compute @workgroup_size(16, 1, 1)
fn top_to_bottom(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = min.x + invocation_id.x;

    var prev_light = 0.;
    var decay = 0.;

    for (var y = min.y; y < max.y; y += 1u) {
        let pos = vec2(x, y);
        let this_light = textureLoad(light_texture, pos).r;

        if (prev_light - this_light) > EPSILON {
            let new_light = prev_light * decay;
            textureStore(light_texture, pos, vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        decay = get_decay(pos);
    }
}

@compute @workgroup_size(16, 1, 1)
fn bottom_to_top(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = min.x + invocation_id.x;

    var prev_light = 0.;
    var decay = 0.;

    for (var y = max.y - 1u; y > min.y; y -= 1u) {
        let pos = vec2(x, y);
        let this_light = textureLoad(light_texture, pos).r;

        if (prev_light - this_light) > EPSILON {
            let new_light = prev_light * decay;
            textureStore(light_texture, pos, vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        decay = get_decay(pos);
    }
}