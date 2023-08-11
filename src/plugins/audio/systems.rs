use bevy::prelude::{Commands, EventReader, Res, AudioBundle, PlaybackSettings};

use crate::plugins::assets::SoundAssets;

use super::PlaySoundEvent;

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