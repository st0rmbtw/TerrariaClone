use bevy::{prelude::{EventWriter, Res, Resource, With, Changed, Query, Component, Color}, text::Text, ui::{Interaction, BackgroundColor}};

use crate::{plugins::{audio::{PlaySoundEvent, SoundType, UpdateMusicVolume, UpdateSoundVolume}, slider::Slider, config::ShowTileGrid}, common::BoolValue, language::LanguageContent};

use super::components::{SoundVolumeSlider, MusicVolumeSlider, ToggleTileGridButton};

pub(super) fn play_sound_on_hover<B: Component>(
    mut query: Query<&Interaction, (With<B>, Changed<Interaction>)>,
    mut play_sound: EventWriter<PlaySoundEvent>
) {
    for interaction in query.iter_mut() {
        if let Interaction::Hovered = interaction {
            play_sound.send(PlaySoundEvent(SoundType::MenuTick));
        }
    }
}

pub(super) fn play_sound_on_toggle<R: BoolValue + Resource>(
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
        update_music_volume.send(UpdateMusicVolume(slider.value()));
    }
}

pub(super) fn update_sound_volume(
    query_slider: Query<&Slider, (With<SoundVolumeSlider>, Changed<Slider>)>,
    mut update_sound_volume: EventWriter<UpdateSoundVolume>
) {
    if let Ok(slider) = query_slider.get_single() {
        update_sound_volume.send(UpdateSoundVolume(slider.value()));
    }
}

pub(super) fn bind_slider_to_output<S: Component, O: Component>(
    query_slider: Query<&Slider, With<S>>,
    mut query_output: Query<&mut Text, With<O>>
) {
    let Ok(slider) = query_slider.get_single() else { return; };
    let Ok(mut text) = query_output.get_single_mut() else { return; };

    text.sections[0].value = format!("{:.0}", slider.value() * 100.);
}

pub(super) fn animate_slider_border_color(
    mut query: Query<(&Interaction, &mut BackgroundColor), (With<Slider>, Changed<Interaction>)>,
) {
    for (interaction, mut background_color) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                background_color.0 = Color::YELLOW;
            }
            Interaction::None => {
                background_color.0 = Color::WHITE;
            },
            _ => {}
        }
    }
}

pub(super) fn update_toggle_tile_grid_button_text(
    mut query: Query<&mut Text, With<ToggleTileGridButton>>,
    show_tile_grid: Res<ShowTileGrid>,
    language_content: Res<LanguageContent>
) {
    if let Ok(mut text) = query.get_single_mut() {
        let status = if show_tile_grid.0 { language_content.ui.on.clone() } else { language_content.ui.off.clone() } ;

        text.sections[0].value = format!("{} {}", language_content.ui.tile_grid, status);
    }
}