use bevy::{prelude::{App, Plugin, Commands, TextBundle, Res, Color, SystemSet, NodeBundle, default, BuildChildren, Camera2dBundle, ButtonBundle, Changed, Query, Component, Transform, Vec3, Entity, With}, text::TextStyle, ui::{Style, Size, Val, JustifyContent, AlignItems, FlexDirection, UiRect, Interaction}};

use crate::{state::GameState, TRANSPARENT, util::RectExtensions};

use super::{FontAssets, MainCamera};

// region: Plugin
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_system_set(
                SystemSet::on_enter(GameState::MainMenu)
                    .with_system(setup_main_menu)
            )
            .add_system_set(
                SystemSet::on_update(GameState::MainMenu)
                    .with_system(update_buttons)
            )
            .add_system_set(
                SystemSet::on_exit(GameState::MainMenu)
                    .with_system(despawn_camera)
            );
    }
}
// endregion

#[derive(Component)]
struct SinglePlayerButton;

#[derive(Component)]
struct SettingsButton;

#[derive(Component)]
struct ExitButton;

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(MainCamera);
}

fn despawn_camera(
    query: Query<Entity, With<MainCamera>>,
    mut commands: Commands
) {
    let entity = query.single();
    commands.entity(entity).despawn();
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
    mut query: Query<(&Interaction, &mut Transform), Changed<Interaction>>
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