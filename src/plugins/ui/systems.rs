use bevy::{prelude::{Commands, Res, NodeBundle, Name, BuildChildren, EventWriter, ResMut}, ui::{Style, FlexDirection, Val, JustifyContent, AlignItems, UiRect}, utils::default};

use crate::{plugins::{assets::{FontAssets, UiAssets}, fps::spawn_fps_text, inventory::spawn_inventory_ui, config::spawn_ingame_settings_button, audio::{PlaySoundEvent, SoundType}}, language::LanguageContent, common::IsVisible};

use super::{components::MainUiContainer, resources::{ExtraUiVisibility, UiVisibility}};

pub(crate) fn spawn_ui_container(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    ui_assets: Res<UiAssets>,
    language_content: Res<LanguageContent>
) {
    let main_id = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .insert(MainUiContainer)
        .insert(Name::new("Main UI Container"))
        .id();

    // Left container
    let left_id = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Left UI Container"))
        .id();

    // Right container
    let right_id = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                padding: UiRect {
                    right: Val::Px(20.),
                    ..UiRect::vertical(Val::Px(5.))
                },
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Right UI Container"))
        .id();

    let fps_text = spawn_fps_text(&mut commands, &font_assets);
    let inventory = spawn_inventory_ui(&mut commands, &ui_assets, &font_assets, &language_content);
    let settings_btn = spawn_ingame_settings_button(&mut commands, &font_assets, &language_content);

    // TODO: Make a health bar in feature, stub now
    let health_bar = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(10.),
                height: Val::Px(2.),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Stub"))
        .id();

    commands
        .entity(left_id)
        .push_children(&[inventory, fps_text]);

    commands
        .entity(right_id)
        .push_children(&[health_bar, settings_btn]);

    commands.entity(main_id).push_children(&[left_id, right_id]);
}

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