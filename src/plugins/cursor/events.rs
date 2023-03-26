use bevy::prelude::{Deref, DerefMut};

use super::components::Hoverable;

#[derive(Deref, DerefMut)]
pub(crate) struct UpdateHoverableInfoEvent(pub Hoverable);