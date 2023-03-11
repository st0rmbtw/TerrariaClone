#define_import_path game::gi_raymarch

#import game::gi_math

struct RayMarchResult {
    success:  i32,      //
    step: i32,          // steps
    pose: vec2<f32>,    // curr spot
}

fn raymarch(
    ray_origin:         vec2<f32>,
    ray_target:         vec2<f32>,
    max_steps:          i32,
    camera_params:      CameraParams,
    rm_jitter_contrib:  f32,
) -> RayMarchResult {

    var ray_origin  = ray_origin;
    var ray_target  = ray_target;

    let ray_direction          = fast_normalize_2d(ray_target - ray_origin);
    let stop_at                = distance_squared(ray_origin, ray_target);

    var ray_progress:   f32    = 0.0;
    var h                      = vec2<f32>(0.0);
    var h_prev                 = h;
    let min_sdf                = 1e-4;
    var inside                 = true;
    let max_inside_dist        = 20.0;
    let max_inside_dist_sq     = max_inside_dist * max_inside_dist;

    for (var i: i32 = 0; i < max_steps; i++) {

        h_prev = h;
        h = ray_origin + ray_progress * ray_direction;

        if ((ray_progress * ray_progress >= stop_at) || (inside && (ray_progress * ray_progress > max_inside_dist))) {
            return RayMarchResult(1, i, h_prev);
        }


        let uv = world_to_sdf_uv(h, camera_params.view_proj, camera_params.inv_sdf_scale);
        if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
            return RayMarchResult(0, i, h_prev);
        }

        let ray_travel = 0.5;

        if (rm_jitter_contrib > 0.0) {
            // Jitter step.
            let jitter = hash(h);
            ray_progress += ray_travel * (1.0 - rm_jitter_contrib) + rm_jitter_contrib * ray_travel * jitter;
        } else {
            ray_progress += ray_travel;
        }
    }

    return RayMarchResult(0, max_steps, h);
}

fn raymarch_primary(
    ray_origin:         vec2<f32>,
    ray_target:         vec2<f32>,
    max_steps:          i32,
    camera_params:      CameraParams,
    rm_jitter_contrib:  f32,
) -> RayMarchResult {

    var ray_origin  = ray_origin;
    var ray_target  = ray_target;

    let ray_direction          = normalize(ray_target - ray_origin);
    let stop_at                = distance_squared(ray_origin, ray_target);

    var ray_progress:   f32    = 0.0;
    var h                      = vec2<f32>(0.0);
    var h_prev                 = h;
    let min_sdf                = 1e-4;

    for (var i: i32 = 0; i < max_steps; i++) {

        h_prev = h;
        h = ray_origin + ray_progress * ray_direction;

        if ray_progress * ray_progress >= stop_at {
            return RayMarchResult(1, i, h_prev);
        }


        let uv = world_to_sdf_uv(h, camera_params.view_proj, camera_params.inv_sdf_scale);
        if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
            return RayMarchResult(0, i, h_prev);
        }

        let ray_travel = 0.0;

        ray_progress += ray_travel * (1.0 - rm_jitter_contrib) + rm_jitter_contrib * ray_travel * hash(h);
   }

    return RayMarchResult(1, max_steps, h);
}