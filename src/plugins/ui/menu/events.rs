use bevy::prelude::Event;

use crate::common::state::MenuState;

#[derive(Event, Clone, Copy)]
pub(super) struct Back;

#[derive(Event, Clone, Copy)]
pub(super) struct EnterMenu(pub(super) MenuState);