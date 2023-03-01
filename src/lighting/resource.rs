use bevy::prelude::{Resource, Vec2, UVec2};

#[derive(Default, Resource, Copy, Clone)]
pub(super) struct ComputedTargetSizes {
    pub(super) primary_target_size: Vec2,
}

impl ComputedTargetSizes {
    pub(super) fn primary_target_usize(&self) -> UVec2 {
        self.primary_target_size.as_uvec2()
    }
}

#[derive(Default, Resource, Copy, Clone, Debug)]
pub(super) struct LightPassParams {
    pub(super) reservoir_size: u32,
}