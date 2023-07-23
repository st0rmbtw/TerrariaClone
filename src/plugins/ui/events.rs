use bevy::prelude::Event;

#[derive(Event)]
pub(crate) struct ToggleExtraUiEvent(pub(crate) bool);