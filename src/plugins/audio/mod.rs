use bevy::{prelude::{Plugin, App, PostUpdate, IntoSystemConfigs, not, in_state, OnEnter, Commands, World, AudioBundle, PlaybackSettings}, ecs::system::Command, audio::Volume};

use crate::{world::block::BlockType, items::ItemTool, common::state::GameState};

mod systems;
mod events;
mod components;

pub(crate) use events::*;
pub(crate) use components::*;

use super::{config::SoundVolume, assets::SoundAssets};

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
                    systems::handle_update_music_volume_event,
                    systems::handle_play_music_event,
                )
                .chain(),

                systems::handle_update_sound_volume_event,
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
    WallHit,
    BlockPlace(BlockType),

    PlayerToolSwing(ItemTool),

    ItemGrab
}

#[derive(Clone, Copy)]
pub(crate) enum MusicType {
    TitleScreen,
    OverworldDay
}

struct PlaySoundCommand {
    sound: SoundType
}

impl Command for PlaySoundCommand {
    fn apply(self, world: &mut World) {
        let sound_volume = world.resource::<SoundVolume>();
        let sound_assets = world.resource::<SoundAssets>();

        // Don't spawn a sound if the volume is 0
        if sound_volume.get() < f32::EPSILON {
            return;
        }

        world.spawn((
            SoundAudio,
            AudioBundle {
                source: sound_assets.get_handle_by_sound_type(self.sound),
                settings: PlaybackSettings::DESPAWN.with_volume(Volume::Relative(**sound_volume)),
            }
        ));
    }
}

pub(crate) trait AudioCommandsExt {
    fn play_sound(&mut self, sound: SoundType);
}

impl AudioCommandsExt for Commands<'_, '_> {
    #[inline(always)]
    fn play_sound(&mut self, sound: SoundType) {
        self.add(PlaySoundCommand {
            sound
        });
    }
}