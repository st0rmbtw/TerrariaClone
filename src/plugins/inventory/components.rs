use bevy::{prelude::{Component, ReflectComponent}, reflect::Reflect};

#[derive(Reflect, Component, Clone, Copy, Default)]
#[reflect(Component)]
pub(crate) struct UseItemAnimationData(pub usize);

#[derive(Component)]
pub(crate) struct ItemInHand;