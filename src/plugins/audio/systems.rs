use bevy::{prelude::{Commands, EventReader, Res, AudioBundle, PlaybackSettings, Query, Entity, With, ResMut, AudioSink, AudioSinkPlayback, EventWriter}, audio::Volume};

use crate::plugins::{assets::{MusicAssets, SoundAssets}, config::{MusicVolume, SoundVolume}};

use super::{PlaySoundEvent, PlayMusicEvent, MusicAudio, UpdateMusicVolume, UpdateSoundVolume, SoundAudio, MusicType, ToBeDespawned};

pub(super) fn handle_play_sound_event(
    mut commands: Commands,
    mut event_reader: EventReader<PlaySoundEvent>,
    sound_assets: Res<SoundAssets>,
    sound_volume: Res<SoundVolume>
) {
    for event in event_reader.iter() {
        commands.spawn((
            SoundAudio,
            AudioBundle {
                source: sound_assets.get_handle_by_sound_type(event.0),
                settings: PlaybackSettings::DESPAWN.with_volume(Volume::Relative(**sound_volume)),
            }
        ));
    }
}

pub(super) fn handle_play_music_event(
    mut commands: Commands,
    mut event_reader: EventReader<PlayMusicEvent>,
    music_assets: Res<MusicAssets>,
    music_volume: Res<MusicVolume>,
    query_music: Query<Entity, With<MusicAudio>>,
) {
    for event in event_reader.iter() {
        if let Ok(entity) = query_music.get_single() {
            commands.entity(entity).insert(ToBeDespawned);
        }

        commands.spawn((
            MusicAudio,
            AudioBundle {
                source: music_assets.get_handle_by_music_type(event.0),
                settings: PlaybackSettings::LOOP.with_volume(Volume::Relative(**music_volume)),
            }
        ));
    }
}

pub(super) fn update_to_be_despawned_audio(
    mut commands: Commands,
    mut query_music: Query<(Entity, &mut AudioSink), (With<MusicAudio>, With<ToBeDespawned>)>
) {
    for (entity, sink) in &mut query_music {
        sink.set_volume(sink.volume() - 0.25e-3);

        if sink.volume() <= 0. {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn handle_update_music_volume_event(
    mut event_reader: EventReader<UpdateMusicVolume>,
    mut music_volume: ResMut<MusicVolume>,
    query_music: Query<&AudioSink, With<MusicAudio>>
) {
    if let Some(event) = event_reader.iter().last() {
        *music_volume = MusicVolume::new(event.0);
        if let Ok(sink) = query_music.get_single() {
            if event.0 > 0. {
                sink.play();
                sink.set_volume(event.0);
            } else {
                sink.pause();
            }
        }
    }
}

pub(super) fn handle_update_sound_volume_event(
    mut event_reader: EventReader<UpdateSoundVolume>,
    mut sound_volume: ResMut<SoundVolume>,
    query_music: Query<&AudioSink, With<SoundAudio>>
) {
    if let Some(event) = event_reader.iter().last() {
        *sound_volume = SoundVolume::new(event.0);
        if let Ok(sink) = query_music.get_single() {
            sink.set_volume(event.0);
        }
    }
}

pub(super) fn play_menu_music(
    mut play_music: EventWriter<PlayMusicEvent>
) {
    play_music.send(PlayMusicEvent(MusicType::TitleScreen));
}

pub(super) fn play_ingame_music(
    mut play_music: EventWriter<PlayMusicEvent>
) {
    play_music.send(PlayMusicEvent(MusicType::OverworldDay));
}