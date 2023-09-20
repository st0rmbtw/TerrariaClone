use bevy::prelude::Component;

#[cfg(feature = "debug")]
use bevy::prelude::{ReflectComponent, Reflect};

#[derive(Component, Clone, Copy, Default)]
#[cfg_attr(feature = "debug", derive(Reflect))]
#[cfg_attr(feature = "debug", reflect(Component))]
pub(crate) struct UseItemAnimationData(pub usize);

#[derive(Component)]
pub(crate) struct ItemInHand;