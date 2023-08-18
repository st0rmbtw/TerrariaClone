use bevy::prelude::{Event, States};

#[derive(Event, Clone, Copy)]
pub(super) struct Back;

#[derive(Event, Clone, Copy)]
pub(super) struct Enter<S: States + Clone + Copy>(pub(super) S);