use bevy::{prelude::{Component, Deref, DerefMut}, ui::Interaction};

#[derive(Component)]
pub(super) struct MainUiContainer;

#[derive(Component)]
pub(super) struct MusicVolumeSlider;

#[derive(Component)]
pub(super) struct SoundVolumeSlider;

#[derive(Component)]
pub(super) struct ZoomSlider;

#[derive(Component)]
pub(super) struct MusicVolumeSliderOutput;

#[derive(Component)]
pub(super) struct SoundVolumeSliderOutput;

#[derive(Component)]
pub(super) struct ZoomSliderOutput;

#[derive(Component)]
pub(super) struct ToggleTileGridButton;

#[derive(Component, Default, Deref, DerefMut, Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct PreviousInteraction(pub(crate) Interaction);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(crate) struct MouseOver;