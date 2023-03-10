#define_import_path game::types

struct LightSource {
    center: vec2<f32>,
    intensity: f32,
    color: vec3<f32>,
    radius: f32,
}

struct LightSourceBuffer {
    count: u32,
    data: array<LightSource>,
}

struct LightPassParams {
    frame_counter: i32,
    probe_size: i32,
    reservoir_size: u32,
}
