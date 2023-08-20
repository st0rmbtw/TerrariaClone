mod components;
mod resources;
mod systems;

pub(crate) mod inventory;
pub(crate) mod menu;

use std::time::Duration;

use interpolation::EaseFunction;
pub(crate) use resources::*;

use bevy::{prelude::{Plugin, App, KeyCode, Update, IntoSystemConfigs, OnExit, Commands, Res, NodeBundle, default, Name, BuildChildren, Visibility, Component, Entity, Color, TextBundle, Button}, input::common_conditions::input_just_pressed, ui::{Style, Val, FlexDirection, JustifyContent, AlignItems, UiRect, Interaction, AlignSelf, PositionType}, text::{TextAlignment, Text, TextStyle, TextSection}};
use crate::{common::{state::GameState, systems::{set_visibility, animate_button_scale, play_sound_on_hover, despawn_with, set_state}, lens::TextFontSizeLens, conditions::on_click}, language::LanguageContent, animation::{Tween, RepeatStrategy, Animator}};

use self::{
    components::MainUiContainer,
    inventory::{systems::spawn_inventory_ui, InventoryUiPlugin},
    menu::MenuPlugin,
};

use crate::plugins::assets::{FontAssets, UiAssets};

use super::{InGameSystemSet, DespawnOnGameExit};

#[derive(Component)]
pub(super) struct ExitButtonContainer;

#[derive(Component)]
pub(super) struct ExitButton;

#[derive(Component)]
pub(crate) struct FpsText;

pub(crate) struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InventoryUiPlugin, MenuPlugin));

        app.init_resource::<ExtraUiVisibility>();
        app.init_resource::<UiVisibility>();

        app.add_systems(OnExit(GameState::WorldLoading), spawn_ui_container);
        app.add_systems(OnExit(GameState::WorldLoading), spawn_fps_text);
        app.add_systems(OnExit(GameState::InGame), despawn_with::<MainUiContainer>);

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
            set_state(GameState::Menu)
                .in_set(InGameSystemSet::Update)
                .run_if(on_click::<ExitButton>)
        );

        app.add_systems(
            Update,
            (
                animate_button_scale::<ExitButton>,
                play_sound_on_hover::<ExitButton>,
                set_visibility::<ExitButtonContainer, ExtraUiVisibility>,
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
    let settings_btn = spawn_exit_button(&mut commands, &font_assets, &language_content);

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

fn spawn_exit_button(
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
        .insert(ExitButtonContainer)
        .with_children(|c| {
            c.spawn(TextBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_shrink: 0.,
                    ..default()
                },
                text: Text::from_section(
                    language_content.ui.exit.clone(),
                    TextStyle {
                        font: fonts.andy_bold.clone_weak(),
                        font_size: 32.,
                        color: Color::WHITE,
                    },
                ).with_alignment(TextAlignment::Center),
                ..default()
            })
            .insert(Name::new("ExitButton"))
            .insert(Interaction::default())
            .insert(ExitButton)
            .insert(Button)
            .insert(Animator::new(tween));
        })
        .id()
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