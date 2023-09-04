use bevy::prelude::{EventWriter, Res, Resource};

use crate::{plugins::audio::{PlaySoundEvent, SoundType}, common::BoolValue};

pub(crate) fn play_sound_on_toggle<R: BoolValue + Resource>(
    res: Res<R>,
    mut play_sound: EventWriter<PlaySoundEvent>
) {
    let sound = match res.value() {
        true => SoundType::MenuOpen,
        false => SoundType::MenuClose,
    };

    play_sound.send(PlaySoundEvent(sound));
}