use autodefault::autodefault;
use bevy::{prelude::{Plugin, App, Commands, NodeBundle, BuildChildren, Entity, Name, ParallelSystemDescriptorCoercion, Res}, ui::{Style, Size, Val, FlexDirection, JustifyContent, AlignItems, UiRect}};
use iyes_loopless::prelude::AppLooplessStateExt;

use crate::{TRANSPARENT, state::GameState, util::RectExtensions};

use super::{spawn_fps_text, FontAssets, spawn_inventory_ui, UiAssets, spawn_ingame_settings_button};

pub const SPAWN_UI_CONTAINER_LABEL: &str = "spawn_ui_container";

// region: Plugin
pub struct PlayerUiPlugin;

impl Plugin for PlayerUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::InGame, spawn_ui_container);
    }
}
// endregion

#[autodefault(except(UiContainer))]
fn spawn_ui_container(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    ui_assets: Res<UiAssets>
) {
    let main_id = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size { width: Val::Percent(100.), height: Val::Percent(100.) },
            flex_direction: FlexDirection::Row
        },
        color: TRANSPARENT.into(),
    })
    .insert(Name::new("Main UI Container"))
    .id();

    // Left container
    let left_id = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size { width: Val::Percent(100.), height: Val::Percent(100.) },
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::FlexStart
        },
        color: TRANSPARENT.into(),
    })
    .insert(Name::new("Left UI Container"))
    .id();

    // Right container
    let right_id = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size { width: Val::Percent(100.), height: Val::Percent(100.) },
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::FlexEnd,
            padding: UiRect {
                right: Val::Px(20.),
                ..UiRect::vertical(5.)
            }
        },
        color: TRANSPARENT.into()
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
                size: Size { width: Val::Px(10.), height: Val::Px(2.) }  
            },
            color: TRANSPARENT.into()
        })
        .insert(Name::new("Stub"))
        .id();

    commands.entity(left_id)
        .push_children(&[inventory, fps_text]);

    commands.entity(right_id)
        .push_children(&[health_bar, settings_btn]);

    commands
        .entity(main_id)
        .push_children(&[left_id, right_id]);
}