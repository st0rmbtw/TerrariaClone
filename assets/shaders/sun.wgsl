#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let glow_color = vec4(1., 164. / 255., 8. / 255., 1.);
    let glow_size = 1.;
    let pixel = textureSample(texture, texture_sampler, uv);

    var sum = vec4(0.0);
    if (pixel.w <= 5.) {

        var uv_y = uv.y;

        var n = 0;
        loop {
            if n >= 9 { break; }

            uv_y = (uv.y) + (glow_size * f32(n) - 4.5);
            var h_sum = vec4(0.0);
            h_sum += textureSampleLevel(texture, texture_sampler, vec2(uv.x - (4.0 * glow_size), uv_y), 0.);
            h_sum += textureSampleLevel(texture, texture_sampler, vec2(uv.x - (3.0 * glow_size), uv_y), 0.);
            h_sum += textureSampleLevel(texture, texture_sampler, vec2(uv.x - (2.0 * glow_size), uv_y), 0.);
            h_sum += textureSampleLevel(texture, texture_sampler, vec2(uv.x - glow_size, uv_y), 0.);
            h_sum += textureSampleLevel(texture, texture_sampler, vec2(uv.x, uv_y), 0.);
            h_sum += textureSampleLevel(texture, texture_sampler, vec2(uv.x + glow_size, uv_y), 0.);
            h_sum += textureSampleLevel(texture, texture_sampler, vec2(uv.x + (2.0 * glow_size), uv_y), 0.);
            h_sum += textureSampleLevel(texture, texture_sampler, vec2(uv.x + (3.0 * glow_size), uv_y), 0.);
            h_sum += textureSampleLevel(texture, texture_sampler, vec2(uv.x + (4.0 * glow_size), uv_y), 0.);

            sum += h_sum / 9.;
            n += 1;
        }
    }

    return textureSample(texture, texture_sampler, uv);
    // return vec4(1.) * textureSample(texture, texture_sampler, uv);
    // return glow_color;
}