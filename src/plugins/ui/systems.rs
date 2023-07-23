use bevy::{prelude::{Commands, Res, NodeBundle, Name, BuildChildren, EventWriter, ResMut, Visibility, With, DetectChanges, Query, AudioBundle, PlaybackSettings}, ui::{Style, FlexDirection, Val, JustifyContent, AlignItems, UiRect}, utils::default};

use crate::{plugins::{assets::{FontAssets, UiAssets, SoundAssets}, fps::spawn_fps_text, inventory::spawn_inventory_ui, settings::spawn_ingame_settings_button}, language::LanguageContent, common::helpers};

use super::{MainUiContainer, ToggleExtraUiEvent, ExtraUiVisibility, UiVisibility};

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
    mut commands: Commands,
    mut events: EventWriter<ToggleExtraUiEvent>,
    mut visibility: ResMut<ExtraUiVisibility>,
    sounds: Res<SoundAssets>,
) {
    visibility.0 = !visibility.0;
    events.send(ToggleExtraUiEvent(visibility.0));

    let sound = if visibility.0 { &sounds.menu_open } else { &sounds.menu_close };

    commands.spawn(AudioBundle {
        source: sound.clone_weak(),
        settings: PlaybackSettings::DESPAWN
    });
}

pub(super) fn toggle_ui_visibility(mut ui_visibility: ResMut<UiVisibility>) {
    ui_visibility.0 = !ui_visibility.0;
}

pub(super) fn set_main_container_visibility(
    ui_visibility: Res<UiVisibility>,
    mut query: Query<&mut Visibility, With<MainUiContainer>>,
) {
    if ui_visibility.is_changed() {
        for mut visibility in &mut query {
            helpers::set_visibility(&mut visibility, ui_visibility.0);
        }
    }
}