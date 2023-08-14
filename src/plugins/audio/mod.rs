use bevy::prelude::{Plugin, App, PostUpdate, Event, Component, IntoSystemConfigs, not, in_state};

use crate::{world::block::BlockType, items::Tool, common::state::GameState};

mod systems;

pub(crate) struct AudioPlugin;
impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySoundEvent>();
        app.add_event::<PlayMusicEvent>();
        app.add_event::<UpdateMusicVolume>();
        app.add_event::<UpdateSoundVolume>();

        app.add_systems(
            PostUpdate,
            (
                systems::handle_play_sound_event,
                systems::handle_play_music_event,
                systems::handle_update_music_volume_event,
                systems::handle_update_sound_volume_event
            )
            .run_if(not(in_state(GameState::AssetLoading)))
        );
    }
}

#[derive(Clone, Copy)]
pub(crate) enum SoundType {
    MenuTick,
    MenuOpen,
    MenuClose,

    BlockHit(BlockType),
    BlockPlace(BlockType),

    PlayerToolSwing(Tool)
}

#[derive(Clone, Copy)]
pub(crate) enum MusicType {
    TitleScreen
}

#[derive(Component)]
pub(self) struct MusicAudio;

#[derive(Component)]
pub(self) struct SoundAudio;

#[derive(Event)]
pub(crate) struct PlaySoundEvent(pub(crate) SoundType);

#[derive(Event)]
pub(crate) struct PlayMusicEvent(pub(crate) MusicType);

#[derive(Event)]
pub(crate) struct UpdateMusicVolume(pub(crate) f32);

#[derive(Event)]
pub(crate) struct UpdateSoundVolume(pub(crate) f32);