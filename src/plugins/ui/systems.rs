use bevy::{prelude::{EventWriter, Res, Resource, With, Changed, Query, Component, Color}, text::Text, ui::{Interaction, BackgroundColor}};

use crate::{plugins::{audio::{PlaySoundEvent, SoundType, UpdateMusicVolume, UpdateSoundVolume}, slider::Slider}, common::BoolValue};

use super::components::{SoundVolumeSlider, MusicVolumeSlider};

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

pub(super) fn update_music_volume(
    query_slider: Query<&Slider, (With<MusicVolumeSlider>, Changed<Slider>)>,
    mut update_music_volume: EventWriter<UpdateMusicVolume>
) {
    if let Ok(slider) = query_slider.get_single() {
        update_music_volume.send(UpdateMusicVolume(slider.value() / 100.));
    }
}

pub(super) fn update_sound_volume(
    query_slider: Query<&Slider, (With<SoundVolumeSlider>, Changed<Slider>)>,
    mut update_sound_volume: EventWriter<UpdateSoundVolume>
) {
    if let Ok(slider) = query_slider.get_single() {
        update_sound_volume.send(UpdateSoundVolume(slider.value() / 100.));
    }
}

pub(super) fn bind_slider_to_output<S: Component, O: Component>(
    query_slider: Query<&Slider, With<S>>,
    mut query_output: Query<&mut Text, With<O>>
) {
    let Ok(slider) = query_slider.get_single() else { return; };
    let Ok(mut text) = query_output.get_single_mut() else { return; };

    text.sections[0].value = format!("{:.0}", slider.value());
}

pub(super) fn animate_slider_border_color(
    mut query: Query<(&Interaction, &mut BackgroundColor), (With<Slider>, Changed<Interaction>)>,
) {
    for (interaction, mut background_color) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                *background_color = Color::YELLOW.into();
            }
            Interaction::None => {
                *background_color = Color::WHITE.into();
            },
            _ => {}
        }
    }
}