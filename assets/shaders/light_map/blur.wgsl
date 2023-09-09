@group(0) @binding(0)
var tiles_texture: texture_storage_2d<r8uint, read>;

@group(0) @binding(1)
var light_texture: texture_storage_2d<r8unorm, read_write>;

@group(0) @binding(2)
var<uniform> min: vec2<u32>;

@group(0) @binding(3)
var<uniform> max: vec2<u32>;

const DECAY_THROUGH_SOLID: f32 = 0.56;
const DECAY_THROUGH_AIR: f32 = 0.91;

fn get_decay(pos: vec2<u32>) -> f32 {
    let tile = textureLoad(tiles_texture, pos).r;
    var decay = 0.;

    if tile == 1u {
        decay = DECAY_THROUGH_SOLID;
    } else if tile == 0u {
        decay = DECAY_THROUGH_AIR;
    }

    return decay;
}

@compute @workgroup_size(1, 8, 1)
fn left_to_right(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let y = min.y + invocation_id.y;

    var prev_light = 0.;
    var decay = get_decay(vec2(min.x, y / u32(#SUBDIVISION)));

    for (var x = min.x; x < max.x; x += 1u) {
        let pos = vec2(x, y);
        let this_light = textureLoad(light_texture, pos).r;

        if this_light < prev_light {
            let new_light = prev_light * decay;
            textureStore(light_texture, pos, vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        decay = get_decay(pos / u32(#SUBDIVISION));
    }
}

@compute @workgroup_size(1, 8, 1)
fn right_to_left(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let y = min.y + invocation_id.y;

    var prev_light = 0.;
    var decay = get_decay(vec2(max.x - 1u, y / u32(#SUBDIVISION)));

    for (var x = max.x - 1u; x > min.x; x -= 1u) {
        let pos = vec2(x, y);
        let this_light = textureLoad(light_texture, pos).r;

        if this_light < prev_light {
            let new_light = prev_light * decay;
            textureStore(light_texture, pos, vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        decay = get_decay(pos / u32(#SUBDIVISION));
    }
}

@compute @workgroup_size(8, 1, 1)
fn top_to_bottom(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = min.x + invocation_id.x;

    var prev_light = 0.;
    var decay = get_decay(vec2(x / u32(#SUBDIVISION), min.y));

    for (var y = min.y; y < max.y; y += 1u) {
        let pos = vec2(x, y);
        let this_light = textureLoad(light_texture, pos).r;

        if this_light < prev_light {
            let new_light = prev_light * decay;
            textureStore(light_texture, pos, vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        decay = get_decay(pos / u32(#SUBDIVISION));
    }
}

@compute @workgroup_size(8, 1, 1)
fn bottom_to_top(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = min.x + invocation_id.x;

    var prev_light = 0.;
    var decay = get_decay(vec2(x / u32(#SUBDIVISION), max.y - 1u));

    for (var y = max.y - 1u; y > min.y; y -= 1u) {
        let pos = vec2(x, y);
        let this_light = textureLoad(light_texture, pos).r;

        if this_light < prev_light {
            let new_light = prev_light * decay;
            textureStore(light_texture, pos, vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        decay = get_decay(pos / u32(#SUBDIVISION));
    }
}