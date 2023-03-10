#import game::camera
#import game::types
#import game::halton
#import game::attenuation

@group(0) @binding(0) var<uniform> camera_params:         CameraParams;
@group(0) @binding(1) var<uniform> cfg:                   LightPassParams;
@group(0) @binding(2) var<storage> lights_source_buffer:  LightSourceBuffer;
@group(0) @binding(3) var          ss_probe_out:          texture_storage_2d<rgba16float, write>;


@compute @workgroup_size(10, 10, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let tile_xy      = vec2<i32>(invocation_id.xy);

    // Screen-space position of the probe.
    let probe_tile_origin_screen = tile_xy * cfg.probe_size;

    // Get current frame.
    var probe_center_world = screen_to_world(
        probe_tile_origin_screen,
        camera_params.screen_size,
        camera_params.inverse_view_proj,
        camera_params.screen_size_inv,
    );

    probe_center_world = probe_center_world + vec2(4., -4.);

    var probe_irradiance = vec3<f32>(0.0);

    for (var i: i32 = 0; i < i32(lights_source_buffer.count); i++) {
        let light = lights_source_buffer.data[i];

        let att = smoothstep(light.radius, 0., fast_distance_2d(light.center, probe_center_world));

        probe_irradiance += light.color * att * light.intensity;
    }

    let reservoir_size = i32(cfg.reservoir_size);
    let frame_index = cfg.frame_counter % reservoir_size;
    let halton_jitter = hammersley2d(frame_index, reservoir_size);
    let out_halton_jitter = pack2x16float(halton_jitter);
    var out_color = vec4<f32>(probe_irradiance, bitcast<f32>(out_halton_jitter));

    textureStore(ss_probe_out, tile_xy, out_color);
}