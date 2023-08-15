use bevy::prelude::Event;

use crate::common::state::GameState;

#[derive(Event, Clone, Copy)]
pub(super) struct Back;

#[derive(Event, Clone, Copy)]
pub(super) struct Enter(pub(super) GameState);