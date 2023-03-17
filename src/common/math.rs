pub fn map_range_usize(from_range: (usize, usize), to_range: (usize, usize), s: usize) -> usize {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

pub fn map_range_f32(in_min: f32, in_max: f32, out_min: f32, out_max: f32, value: f32) -> f32 {
    out_min + (((value - in_min) / (in_max - in_min)) * (out_max - out_min))
}

pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        return target;
    }
    current + (target - current).signum() * max_delta
}

pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if a != b {
        ((value - a) / (b - a)).clamp(0., 1.)
    } else {
        0.
    }
}