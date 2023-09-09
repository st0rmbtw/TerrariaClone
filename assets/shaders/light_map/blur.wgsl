@group(0) @binding(0)
var tiles_texture: texture_storage_2d<r8uint, read>;

@group(0) @binding(1)
var texture: texture_storage_2d<r8unorm, read_write>;

@group(0) @binding(2)
var<uniform> min: vec2<u32>;

@group(0) @binding(3)
var<uniform> max: vec2<u32>;

@compute @workgroup_size(1, 8, 1)
fn left_to_right(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let y = min.y + invocation_id.y;

    let tile = textureLoad(tiles_texture, vec2(0u, y / u32(#SUBDIVISION))).r;

    var prev_light = 0.;
    var decay = 0.;

    if tile == 1u {
        decay = 0.56;
    } else if tile == 0u {
        decay = 0.91;
    }

    for (var x = min.x; x < max.x; x += 1u) {
        let this_light = textureLoad(texture, vec2(x, y)).r;

        if this_light < prev_light {
            let new_light = prev_light * decay;
            textureStore(texture, vec2(x, y), vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        let tile = textureLoad(tiles_texture, vec2(x / u32(#SUBDIVISION), y / u32(#SUBDIVISION))).r;
        if tile == 1u {
            decay = 0.56;
        } else if tile == 0u {
            decay = 0.91;
        }
    }
}

@compute @workgroup_size(1, 8, 1)
fn right_to_left(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let y = min.y + invocation_id.y;

    let tile = textureLoad(tiles_texture, vec2(0u, y / u32(#SUBDIVISION))).r;

    var prev_light = 0.;
    var decay = 0.;

    if tile == 1u {
        decay = 0.56;
    } else if tile == 0u {
        decay = 0.91;
    }

    for (var x = max.x - 1u; x > min.x; x -= 1u) {
        let this_light = textureLoad(texture, vec2(x, y)).r;

        if this_light < prev_light {
            let new_light = prev_light * decay;
            textureStore(texture, vec2(x, y), vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        let tile = textureLoad(tiles_texture, vec2(x / u32(#SUBDIVISION), y / u32(#SUBDIVISION))).r;
        if tile == 1u {
            decay = 0.56;
        } else if tile == 0u {
            decay = 0.91;
        }
    }
}

@compute @workgroup_size(8, 1, 1)
fn top_to_bottom(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = min.x + invocation_id.x;

    let tile = textureLoad(tiles_texture, vec2(x / u32(#SUBDIVISION), 0u)).r;

    var prev_light = 0.;
    var decay = 0.;

    if tile == 1u {
        decay = 0.56;
    } else if tile == 0u {
        decay = 0.91;
    }

    for (var y = min.y; y < max.y; y += 1u) {
        let this_light = textureLoad(texture, vec2(x, y)).r;

        if this_light < prev_light {
            let new_light = prev_light * decay;
            textureStore(texture, vec2(x, y), vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        let tile = textureLoad(tiles_texture, vec2(x / u32(#SUBDIVISION), y / u32(#SUBDIVISION))).r;
        if tile == 1u {
            decay = 0.56;
        } else if tile == 0u {
            decay = 0.91;
        }
    }
}

@compute @workgroup_size(8, 1, 1)
fn bottom_to_top(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = min.x + invocation_id.x;

    let tile = textureLoad(tiles_texture, vec2(x / u32(#SUBDIVISION), 0u)).r;

    var prev_light = 0.;
    var decay = 0.;

    if tile == 1u {
        decay = 0.56;
    } else if tile == 0u {
        decay = 0.91;
    }

    for (var y = max.y - 1u; y > min.y; y -= 1u) {
        let this_light = textureLoad(texture, vec2(x, y)).r;

        if this_light < prev_light {
            let new_light = prev_light * decay;
            textureStore(texture, vec2(x, y), vec4(vec3(new_light), 1.));
            prev_light = new_light;
        } else {
            prev_light = this_light;
        }

        let tile = textureLoad(tiles_texture, vec2(x / u32(#SUBDIVISION), y / u32(#SUBDIVISION))).r;
        if tile == 1u {
            decay = 0.56;
        } else if tile == 0u {
            decay = 0.91;
        }
    }
}