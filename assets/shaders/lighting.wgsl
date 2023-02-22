#import game::gi_camera
#import game::gi_types
#import game::gi_halton
#import game::gi_raymarch
#import game::gi_attenuation

@group(0) @binding(0) var<uniform> camera_params:         CameraParams;
@group(0) @binding(1) var<uniform> cfg:                   LightPassParams;
@group(0) @binding(2) var<storage> lights_source_buffer:  LightSourceBuffer;
@group(0) @binding(3) var          ss_probe_out:          texture_storage_2d<rgba16float, write>;


@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let tile_xy      = vec2<i32>(invocation_id.xy);

    // Screen-space position of the probe.
    let reservoir_size           = i32(cfg.reservoir_size);
    let frame_index              = cfg.frame_counter % reservoir_size;
    let probe_tile_origin_screen = tile_xy * cfg.probe_size;
    let probe_size_f32           = f32(cfg.probe_size);
    let halton_jitter            = hammersley2d(frame_index, reservoir_size);

    // Get current frame.
    let probe_offset_world  = halton_jitter * probe_size_f32;
    let probe_center_world_unbiased = screen_to_world(
                                              probe_tile_origin_screen,
                                              camera_params.screen_size,
                                              camera_params.inverse_view_proj,
                                              camera_params.screen_size_inv,
                                          );
    let probe_center_world  =  probe_center_world_unbiased + probe_offset_world;

    var probe_irradiance = vec3<f32>(0.0);

    for (var i: i32 = 0; i < i32(lights_source_buffer.count); i++) {
        let light = lights_source_buffer.data[i];

        let ray_result = raymarch_primary(
            probe_center_world,
            light.center,
            32,
            camera_params,
            0.3
        );

        let att = light_attenuation_r2(
            probe_center_world,
            light.center,
            light.falloff.x,
            light.falloff.y,
            light.falloff.z,
        );

        // let att = 1.;

        if (ray_result.success > 0) {
            probe_irradiance += light.color * att * light.intensity;
        }
    }

    // Coordinates of the screen-space cache output tile.
    let atlas_row  = frame_index / cfg.probe_size;
    let atlas_col  = frame_index % cfg.probe_size;

    let out_atlas_tile_offset = vec2<i32>(
        cfg.probe_atlas_cols * atlas_col,
        cfg.probe_atlas_rows * atlas_row,
    );

    let out_atlas_tile_pose = out_atlas_tile_offset + tile_xy;

    textureStore(ss_probe_out, out_atlas_tile_pose, vec4(probe_irradiance, 1.));
    // textureStore(ss_probe_out, out_atlas_tile_pose, vec4(1., 1., 1., 1.));
}