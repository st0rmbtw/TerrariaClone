use bevy::prelude::Event;

use super::{SoundType, MusicType};

#[derive(Event)]
pub(crate) struct PlaySoundEvent(pub(crate) SoundType);

#[derive(Event)]
pub(crate) struct PlayMusicEvent(pub(crate) MusicType);

#[derive(Event)]
pub(crate) struct UpdateMusicVolume(pub(crate) f32);

#[derive(Event)]
pub(crate) struct UpdateSoundVolume(pub(crate) f32);