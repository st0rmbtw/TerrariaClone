mod components;
mod resources;
mod systems;

pub(crate) mod ingame;
pub(crate) mod menu;

pub(crate) use resources::*;

use bevy::{prelude::{Plugin, App, KeyCode, Update, IntoSystemConfigs, OnExit, Commands, Res, NodeBundle, default, Name, BuildChildren, Visibility, Color, TextBundle, Condition, Button, resource_exists_and_equals, not, Component}, input::common_conditions::input_just_pressed, ui::{Style, Val, FlexDirection, JustifyContent, AlignItems, UiRect, PositionType}, text::{TextAlignment, Text, TextStyle, TextSection}};
use crate::{common::{state::GameState, systems::{set_visibility, despawn_with, toggle_resource, animate_button_scale, play_sound_on_hover}}, language::LanguageContent};

use self::{
    components::MainUiContainer,
    ingame::{inventory::{systems::spawn_inventory_ui, InventoryUiPlugin}, settings::{systems::spawn_ingame_settings_button, InGameSettingsUiPlugin}},
    menu::MenuPlugin,
};

use crate::plugins::assets::{FontAssets, UiAssets};

use super::{InGameSystemSet, DespawnOnGameExit, slider::Slider};

#[derive(Component)]
pub(crate) struct FpsText;

pub(crate) struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InventoryUiPlugin, InGameSettingsUiPlugin, MenuPlugin));

        app.init_resource::<InventoryUiVisibility>();
        app.init_resource::<UiVisibility>();

        app.add_systems(OnExit(GameState::WorldLoading), spawn_ui_container);
        app.add_systems(OnExit(GameState::WorldLoading), spawn_fps_text);
        app.add_systems(OnExit(GameState::InGame), despawn_with::<MainUiContainer>);

        app.add_systems(
            Update,
            (
                animate_button_scale::<Button>,
                play_sound_on_hover::<Button>,
                play_sound_on_hover::<Slider>,
            )
        );

        app.add_systems(
            Update,
            (
                (
                    toggle_resource::<InventoryUiVisibility>,
                    systems::play_sound_on_toggle::<InventoryUiVisibility>
                )
                .chain()
                .run_if(
                    not(resource_exists_and_equals(SettingsMenuVisibility(true))).and_then(input_just_pressed(KeyCode::Escape))
                ),

                toggle_resource::<UiVisibility>.run_if(input_just_pressed(KeyCode::F11)),
                set_visibility::<MainUiContainer, UiVisibility>
            )
            .in_set(InGameSystemSet::Update)
        );
    }
}

fn spawn_ui_container(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    ui_assets: Res<UiAssets>,
    language_content: Res<LanguageContent>
) {
    let main_id = commands
        .spawn((
            Name::new("Main UI Container"),
            MainUiContainer,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                ..default()
            }
        ))
        .id();

    // Left container
    let left_id = commands
        .spawn((
            Name::new("Left UI Container"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    margin: UiRect::left(Val::Px(20.)),
                    ..default()
                },
                ..default()
            }
        ))
        .id();

    // Right container
    let right_id = commands
        .spawn((
            Name::new("Right UI Container"),
            NodeBundle {
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
            }
        ))
        .id();

    let inventory = spawn_inventory_ui(&mut commands, &ui_assets, &font_assets, &language_content);
    let settings_btn = spawn_ingame_settings_button(&mut commands, &font_assets, &language_content);

    // TODO: Make a health bar in the feature, stub for now
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
        .push_children(&[inventory]);

    commands
        .entity(right_id)
        .push_children(&[health_bar, settings_btn]);

    commands.entity(main_id).push_children(&[left_id, right_id]);
}

pub(super) fn spawn_fps_text(mut commands: Commands, font_assets: Res<FontAssets>) {
    let text_style = TextStyle {
        font: font_assets.andy_regular.clone_weak(),
        font_size: 20.,
        color: Color::WHITE,
    };

    commands.spawn((
        FpsText,
        Name::new("FPS Text"),
        DespawnOnGameExit,
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(5.),
                bottom: Val::Px(0.),
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection::from_style(text_style)
                ],
                alignment: TextAlignment::Left,
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        }
    ));
}