use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, NodeBundle, Name, Input, BuildChildren, EventWriter, ResMut, KeyCode, Query, Visibility, With}, ui::{Style, Size, FlexDirection, Val, JustifyContent, AlignItems, UiRect}};

use crate::{plugins::{assets::{FontAssets, UiAssets}, fps::spawn_fps_text, inventory::spawn_inventory_ui, settings::spawn_ingame_settings_button}, TRANSPARENT, util::RectExtensions};

use super::{MainUiContainer, ToggleExtraUiEvent, ExtraUiVisibility, UiVisibility};

#[autodefault(except(UiContainer))]
pub fn spawn_ui_container(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    ui_assets: Res<UiAssets>,
) {
    let main_id = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                flex_direction: FlexDirection::Row,
            },
            color: TRANSPARENT.into(),
        })
        .insert(MainUiContainer)
        .insert(Name::new("Main UI Container"))
        .id();

    // Left container
    let left_id = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
            },
            color: TRANSPARENT.into(),
        })
        .insert(Name::new("Left UI Container"))
        .id();

    // Right container
    let right_id = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                padding: UiRect {
                    right: Val::Px(20.),
                    ..UiRect::vertical(5.)
                },
            },
            color: TRANSPARENT.into(),
        })
        .insert(Name::new("Right UI Container"))
        .id();

    let fps_text = spawn_fps_text(&mut commands, &font_assets);
    let inventory = spawn_inventory_ui(&mut commands, &ui_assets, &font_assets);
    let settings_btn = spawn_ingame_settings_button(&mut commands, &font_assets);

    // TODO: Make a health bar in feature, stub now
    let health_bar = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Px(10.),
                    height: Val::Px(2.),
                },
            },
            color: TRANSPARENT.into(),
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

pub fn toggle_extra_ui(
    input: Res<Input<KeyCode>>,
    mut events: EventWriter<ToggleExtraUiEvent>,
    mut extra_ui_visibility: ResMut<ExtraUiVisibility>,
) {
    if input.just_pressed(KeyCode::Escape) {
        let visibility = !extra_ui_visibility.0;
        extra_ui_visibility.0 = visibility;
        events.send(ToggleExtraUiEvent(visibility));
    }
}

pub fn toggle_ui(input: Res<Input<KeyCode>>, mut ui_visibility: ResMut<UiVisibility>) {
    if input.just_pressed(KeyCode::F11) {
        ui_visibility.0 = !ui_visibility.0;
    }
}

pub fn set_main_container_visibility(
    ui_visibility: Res<UiVisibility>,
    mut query: Query<&mut Visibility, With<MainUiContainer>>,
) {
    if ui_visibility.is_changed() {
        for mut visibility in &mut query {
            visibility.is_visible = ui_visibility.0;
        }
    }
}