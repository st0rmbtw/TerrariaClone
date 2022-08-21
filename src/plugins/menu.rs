use bevy::{prelude::{App, Plugin, Commands, TextBundle, Res, Color, NodeBundle, default, BuildChildren, Camera2dBundle, ButtonBundle, Changed, Query, Component, Transform, Vec3, Entity, With, Button, DespawnRecursiveExt, EventWriter}, text::TextStyle, ui::{Style, Size, Val, JustifyContent, AlignItems, FlexDirection, UiRect, Interaction}, app::AppExit};
use iyes_loopless::prelude::*;

use crate::{state::GameState, TRANSPARENT, util::RectExtensions};

use super::{FontAssets, MainCamera};

// region: Plugin
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_enter_system(GameState::MainMenu, setup_main_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(update_buttons)
                    .with_system(single_player_btn.run_if(on_btn_clicked::<SinglePlayerButton>))
                    .with_system(exit_btn.run_if(on_btn_clicked::<ExitButton>))
                    .into()
            )
            .add_exit_system(GameState::MainMenu, despawn_with::<MainCamera>)
            .add_exit_system(GameState::MainMenu, despawn_with::<Menu>);
    }
}
// endregion

#[derive(Component)]
struct SinglePlayerButton;

#[derive(Component)]
struct SettingsButton;

#[derive(Component)]
struct ExitButton;

#[derive(Component)]
struct Menu;

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(MainCamera);
}

fn on_btn_clicked<B: Component>(
    query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<B>)>,
) -> bool {
    for interaction in query.iter() {
        if *interaction == Interaction::Clicked {
            return true;
        }
    }

    false
}

fn despawn_with<C: Component>(
    query: Query<Entity, With<C>>,
    mut commands: Commands
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_main_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size { 
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            color: TRANSPARENT.into(),
            ..default()
        })
        .insert(Menu)
        .with_children(|children| {
            children.spawn_bundle(ButtonBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::vertical(10.),
                    ..default()
                },
                color: TRANSPARENT.into(),
                ..default()
            })
            .insert(SinglePlayerButton)
            .with_children(|c| {
                c.spawn_bundle(TextBundle::from_section(
                    "Single Player", 
                    TextStyle { 
                        font: fonts.andy_bold.clone(), 
                        font_size: 46., 
                        color: Color::rgb(150. / 255., 145. / 255., 151. / 255.) 
                    }
                ));
            });

            children.spawn_bundle(ButtonBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::vertical(10.),
                    ..default()
                },
                color: TRANSPARENT.into(),
                ..default()
            })
            .insert(SettingsButton)
            .with_children(|c| {
                c.spawn_bundle(TextBundle::from_section(
                    "Settings", 
                    TextStyle { 
                        font: fonts.andy_bold.clone(), 
                        font_size: 46., 
                        color: Color::rgb(150. / 255., 145. / 255., 151. / 255.) 
                    }
                ));
            });

            children.spawn_bundle(ButtonBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::vertical(10.),
                    ..default()
                },
                color: TRANSPARENT.into(),
                ..default()
            })
            .insert(ExitButton)
            .with_children(|c| {
                c.spawn_bundle(TextBundle::from_section(
                    "Exit", 
                    TextStyle { 
                        font: fonts.andy_bold.clone(), 
                        font_size: 46., 
                        color: Color::rgb(150. / 255., 145. / 255., 151. / 255.) 
                    }
                ));
            });
        });
}

fn update_buttons(
    mut query: Query<(&Interaction, &mut Transform), (With<Button>, Changed<Interaction>)>
) {
    for (interaction, mut transform) in query.iter_mut() {
        match interaction {
            Interaction::Clicked => {},
            Interaction::Hovered => {
                transform.scale = Vec3::splat(1.2);
            },
            Interaction::None => {
                transform.scale = Vec3::ONE;
            }
        }
    }
}

fn single_player_btn(
    mut commands: Commands,
) {
    commands.insert_resource(NextState(GameState::WorldLoading));
}

fn exit_btn(
    mut ev: EventWriter<AppExit>
) {
    ev.send(AppExit);
}