use bevy::{prelude::{EventWriter, Res, Resource, With, Changed, Query, Component, Color, DetectChanges, DetectChangesMut, Commands, Entity, GlobalTransform, ResMut}, text::Text, ui::{Interaction, BackgroundColor, Node}};

use crate::{plugins::{audio::{SoundType, UpdateMusicVolume, UpdateSoundVolume, AudioCommandsExt}, slider::Slider, config::ShowTileGrid, cursor::{components::Hoverable, position::CursorPosition}, camera::components::MainCamera}, common::{BoolValue, components::EntityRect}, language::{LocalizedText, keys::UIStringKey, args}};

use super::{components::{SoundVolumeSlider, MusicVolumeSlider, ToggleTileGridButton, PreviousInteraction, MouseOver}, MouseOverUi};

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

pub(super) fn update_world_mouse_over(
    mut commands: Commands,
    cursor_pos: Res<CursorPosition<MainCamera>>,
    query: Query<(Entity, &EntityRect), With<Hoverable>>
) {
    query.for_each(|(entity, entity_rect)| {
        if entity_rect.contains(cursor_pos.world) {
            commands.entity(entity).insert(MouseOver);
        } else {
            commands.entity(entity).remove::<MouseOver>();
        }
    });
}

pub(super) fn update_ui_mouse_over(
    mut commands: Commands,
    cursor_pos: Res<CursorPosition<MainCamera>>,
    query: Query<(Entity, &Node, &GlobalTransform), With<Hoverable>>
) {
    query.for_each(|(entity, node, global_transform)| {
        if node.logical_rect(global_transform).contains(cursor_pos.screen) {
            commands.entity(entity).insert(MouseOver);
        } else {
            commands.entity(entity).remove::<MouseOver>();
        }
    });
}

pub(super) fn update_mouse_over_ui(
    mut mouse_over_ui: ResMut<MouseOverUi>,
    query: Query<&Interaction, With<Node>>
) {
    mouse_over_ui.0 = false;

    for interaction in &query {
        if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
            mouse_over_ui.0 = true;
            return;
        }
    }
}