mod components;
mod resources;
mod systems;

pub(crate) mod fps;
pub(crate) mod inventory;
pub(crate) mod menu;

use std::time::Duration;

use interpolation::EaseFunction;
pub(crate) use resources::*;

use bevy::{prelude::{Plugin, App, KeyCode, Update, IntoSystemConfigs, OnExit, Commands, Res, NodeBundle, default, Name, BuildChildren, Visibility, Component, Entity, Color, TextBundle}, input::common_conditions::input_just_pressed, ui::{Style, Val, FlexDirection, JustifyContent, AlignItems, UiRect, Interaction, AlignSelf}, text::{TextAlignment, Text, TextStyle}};
use crate::{common::{state::GameState, systems::{set_visibility, animate_button_scale, play_sound_on_hover}, lens::TextFontSizeLens}, InGameSystemSet, language::LanguageContent, animation::{Tween, RepeatStrategy, Animator}};

use self::{
    components::MainUiContainer,
    inventory::{systems::spawn_inventory_ui, InventoryUiPlugin},
    menu::MenuPlugin,
    fps::spawn_fps_text
};

use crate::plugins::assets::{FontAssets, UiAssets};

#[derive(Component)]
pub(super) struct SettingsButtonContainer;

#[derive(Component)]
pub(super) struct SettingsButton;

pub(crate) struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InventoryUiPlugin, MenuPlugin));

        app.init_resource::<ExtraUiVisibility>();
        app.init_resource::<UiVisibility>();

        app.add_systems(OnExit(GameState::WorldLoading), spawn_ui_container);
        app.add_systems(Update,
            (
                systems::toggle_extra_ui_visibility.run_if(input_just_pressed(KeyCode::Escape)),
                systems::toggle_ui_visibility.run_if(input_just_pressed(KeyCode::F11)),
                set_visibility::<MainUiContainer, UiVisibility>
            )
            .in_set(InGameSystemSet::Update)
        );
        app.add_systems(
            Update,
            (
                animate_button_scale::<SettingsButton>,
                play_sound_on_hover::<SettingsButton>,
                set_visibility::<SettingsButtonContainer, ExtraUiVisibility>,
            )
            .in_set(InGameSystemSet::Update)
        );
    }
}

pub(crate) fn spawn_ui_container(
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

    let fps_text = spawn_fps_text(&mut commands, &font_assets);
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
        .push_children(&[inventory, fps_text]);

    commands
        .entity(right_id)
        .push_children(&[health_bar, settings_btn]);

    commands.entity(main_id).push_children(&[left_id, right_id]);
}

pub(crate) fn spawn_ingame_settings_button(
    commands: &mut Commands, 
    fonts: &FontAssets,
    language_content: &LanguageContent
) -> Entity {
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        RepeatStrategy::MirroredRepeat,
        Duration::from_millis(150),
        TextFontSizeLens {
            start: 32.,
            end: 38.,
        },
    );

    commands
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::End,
                padding: UiRect::all(Val::Px(10.)),
                width: Val::Px(100.),
                height: Val::Px(38.),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(SettingsButtonContainer)
        .with_children(|c| {
            c.spawn(TextBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_shrink: 0.,
                    ..default()
                },
                text: Text::from_section(
                    language_content.ui.settings.clone(),
                    TextStyle {
                        font: fonts.andy_bold.clone_weak(),
                        font_size: 32.,
                        color: Color::WHITE,
                    },
                ).with_alignment(TextAlignment::Center),
                ..default()
            })
            .insert(Name::new("Settings button"))
            .insert(Interaction::default())
            .insert(SettingsButton)
            .insert(Animator::new(tween));
        })
        .id()
}