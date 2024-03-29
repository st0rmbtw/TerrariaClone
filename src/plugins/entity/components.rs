use bevy::prelude::{Component, DerefMut, Deref, Vec2};

use crate::common::rect::FRect;

#[cfg(feature = "debug")]
use bevy::prelude::{ReflectComponent, Reflect};

#[derive(Component, Clone, Copy, Default, Deref, DerefMut)]
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::InspectorOptions))]
#[cfg_attr(feature = "debug", derive(Reflect))]
#[cfg_attr(feature = "debug", reflect(Component))]
pub(crate) struct EntityRect(pub(crate) FRect);

#[derive(Component, Clone, Copy, Default, Deref, DerefMut)]
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::InspectorOptions))]
#[cfg_attr(feature = "debug", derive(Reflect))]
#[cfg_attr(feature = "debug", reflect(Component))]
pub(crate) struct Velocity(pub(crate) Vec2);

impl From<Vec2> for Velocity {
    fn from(value: Vec2) -> Self {
        Self(value)
    }
}