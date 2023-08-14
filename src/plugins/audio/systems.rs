use bevy::prelude::{Commands, EventReader, Res, AudioBundle, PlaybackSettings, Query, Entity, With, DespawnRecursiveExt};

use crate::plugins::assets::{MusicAssets, SoundAssets};

use super::{PlaySoundEvent, PlayMusicEvent, MusicAudio};

pub(super) fn handle_play_sound_event(
    mut commands: Commands,
    mut event_reader: EventReader<PlaySoundEvent>,
    sound_assets: Res<SoundAssets>
) {
    for event in event_reader.iter() {
        commands.spawn(AudioBundle {
            source: sound_assets.get_handle_by_sound_type(event.0),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}

pub(super) fn handle_play_music_event(
    mut commands: Commands,
    mut event_reader: EventReader<PlayMusicEvent>,
    music_assets: Res<MusicAssets>,
    query_music: Query<Entity, With<MusicAudio>>
) {
    for event in event_reader.iter() {
        if let Ok(entity) = query_music.get_single() {
            commands.entity(entity).despawn_recursive();
        }

        commands.spawn((
            MusicAudio,
            AudioBundle {
                source: music_assets.get_handle_by_music_type(event.0),
                settings: PlaybackSettings::LOOP,
            }
        ));
    }
}