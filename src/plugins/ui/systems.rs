use bevy::{prelude::{EventWriter, Res, Resource, With, Changed, Query, Component, Color, DetectChanges, DetectChangesMut, Commands, ResMut}, text::Text, ui::{Interaction, BackgroundColor, Node}};

use crate::{plugins::{audio::{SoundType, UpdateMusicVolume, UpdateSoundVolume, AudioCommandsExt}, slider::Slider, config::ShowTileGrid}, common::BoolValue, language::{LocalizedText, keys::UIStringKey, args}};

use super::{components::{SoundVolumeSlider, MusicVolumeSlider, ToggleTileGridButton, PreviousInteraction}, MouseOverUi};

pub(super) fn play_sound_on_hover<B: Component>(
    mut commands: Commands,
    mut query: Query<(&PreviousInteraction, &Interaction), (With<B>, Changed<Interaction>)>,
) {
    for (previous_interaction, interaction) in &mut query {
        if **previous_interaction != Interaction::Pressed && *interaction == Interaction::Hovered {
            commands.play_sound(SoundType::MenuTick);
        }
    }
}

pub(super) fn play_sound_on_toggle<R: BoolValue + Resource>(
    mut commands: Commands,
    res: Res<R>,
) {
    let sound = match res.value() {
        true => SoundType::MenuOpen,
        false => SoundType::MenuClose,
    };

    commands.play_sound(sound);
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
    query_slider: Query<&Slider, (With<S>, Changed<Slider>)>,
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
    mut query: Query<&mut LocalizedText, With<ToggleTileGridButton>>,
    show_tile_grid: Res<ShowTileGrid>,
) {
    if let Ok(mut localized_text) = query.get_single_mut() {
        if show_tile_grid.is_changed() {
            let status = if show_tile_grid.0 { UIStringKey::On } else { UIStringKey::Off };
            *localized_text = LocalizedText::new(UIStringKey::TileGrid, "{} {}", args![status]);
        }
    }
}

pub(super) fn update_previous_interaction(
    mut query: Query<(&mut PreviousInteraction, &Interaction), Changed<Interaction>>
) {
    for (mut previous_interaction, interaction) in &mut query {
        previous_interaction.set_if_neq(PreviousInteraction(*interaction));
    }
}

pub(super) fn update_mouse_over_ui(
    mut mouse_over_ui: ResMut<MouseOverUi>,
    query: Query<&Interaction, With<Node>>
) {
    mouse_over_ui.0 = query.iter().any(|&interaction| interaction != Interaction::None);
}