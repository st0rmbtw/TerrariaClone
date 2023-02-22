#define_import_path game::gi_types

struct LightSource {
    center:    vec2<f32>,
    intensity: f32,
    color:     vec3<f32>,
    falloff:   vec3<f32>,
}

struct LightSourceBuffer {
    count: u32,
    data:  array<LightSource>,
}

struct LightPassParams {
    frame_counter:          i32,
    probe_size:             i32,
    probe_atlas_cols:       i32,
    probe_atlas_rows:       i32,

    reservoir_size:         u32,
    smooth_kernel_size_h:   u32,
    smooth_kernel_size_w:   u32,
    direct_light_contrib:   f32,
    indirect_light_contrib: f32,

    indirect_rays_per_sample:    i32,
    indirect_rays_radius_factor: f32,
}
