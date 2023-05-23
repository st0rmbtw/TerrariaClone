#define_import_path game::camera

struct CameraParams {
    screen_size:       vec2<f32>,
    screen_size_inv:   vec2<f32>,
    view_proj:         mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
}

fn screen_to_ndc(
    screen_pos:     vec2<f32>,
    screen_size:     vec2<f32>,
    screen_size_inv: vec2<f32>
) -> vec2<f32> {
    let screen_pos = vec2<f32>(0.0, screen_size.y) + vec2<f32>(screen_pos.x, -screen_pos.y);
    return (screen_pos * screen_size_inv) * 2.0 - 1.0;
}

fn screen_to_world(
    screen_pos:        vec2<f32>,
    screen_size:       vec2<f32>,
    inverse_view_proj: mat4x4<f32>,
    screen_size_inv:   vec2<f32>
) -> vec2<f32> {
    return (inverse_view_proj * vec4<f32>(screen_to_ndc(screen_pos, screen_size, screen_size_inv), 0.0, 1.0)).xy;
}