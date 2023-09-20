use bevy::prelude::{Plugin, App, PostUpdate, IntoSystemConfigs, not, in_state, OnEnter, SystemSet};

use crate::{world::block::BlockType, items::Tool, common::state::GameState};

mod systems;
mod events;
mod components;

pub(crate) use events::*;
pub(crate) use components::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, SystemSet)]
pub(crate) struct HandleAudioEvents;

pub(crate) struct AudioPlugin;
impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySoundEvent>();
        app.add_event::<PlayMusicEvent>();
        app.add_event::<UpdateMusicVolume>();
        app.add_event::<UpdateSoundVolume>();

        app.add_systems(OnEnter(GameState::Menu), systems::play_menu_music);
        app.add_systems(OnEnter(GameState::InGame), systems::play_ingame_music);

        app.add_systems(
            PostUpdate,
            (
                (
                    (
                        systems::handle_update_music_volume_event,
                        systems::handle_update_sound_volume_event,
                    ),
                    (
                        systems::handle_play_sound_event,
                        systems::handle_play_music_event,
                    )
                )
                .in_set(HandleAudioEvents)
                .chain(),
                systems::update_to_be_despawned_audio
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
    TitleScreen,
    OverworldDay
}