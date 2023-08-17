use bevy::prelude::{EventWriter, ResMut};

use crate::{plugins::audio::{PlaySoundEvent, SoundType}, common::IsVisible};

use super::resources::{ExtraUiVisibility, UiVisibility};

pub(super) fn toggle_extra_ui_visibility(
    mut visibility: ResMut<ExtraUiVisibility>,
    mut play_sound: EventWriter<PlaySoundEvent>
) {    
    visibility.toggle();

    let sound = match visibility.is_visible() {
        true => SoundType::MenuOpen,
        false => SoundType::MenuClose,
    };

    play_sound.send(PlaySoundEvent(sound));
}

pub(super) fn toggle_ui_visibility(mut ui_visibility: ResMut<UiVisibility>) {
    ui_visibility.toggle();
}