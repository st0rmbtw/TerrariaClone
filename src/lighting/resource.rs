use bevy::prelude::{Resource, Vec2, IVec2, UVec2};

#[derive(Default, Resource, Copy, Clone)]
pub struct ComputedTargetSizes {
    pub primary_target_size:  Vec2,
}

impl ComputedTargetSizes {
    pub fn primary_target_size(&self) -> Vec2 {
        self.primary_target_size
    }

    pub fn primary_target_isize(&self) -> IVec2 {
        self.primary_target_size.as_ivec2()
    }

    pub fn primary_target_usize(&self) -> UVec2 {
        self.primary_target_size.as_uvec2()
    }
}

#[derive(Default, Resource, Copy, Clone, Debug)]
pub struct LightPassParams {
    pub reservoir_size: u32,

    pub smooth_kernel_size: (u32, u32),

    pub direct_light_contrib: f32,

    pub indirect_light_contrib: f32,

    pub indirect_rays_per_sample: i32,

    pub indirect_rays_radius_factor: f32,
}