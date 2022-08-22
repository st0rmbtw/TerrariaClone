use std::time::Duration;

use bevy::{prelude::{App, Plugin, Commands, TextBundle, Res, Color, NodeBundle, default, BuildChildren, Camera2dBundle, ButtonBundle, Changed, Query, Component, Transform, Vec3, Entity, With, Button, DespawnRecursiveExt, EventWriter, Children, Vec2, ResMut, ChildBuilder}, text::{TextStyle, Text}, ui::{Style, Size, Val, JustifyContent, AlignItems, FlexDirection, UiRect, Interaction}, app::AppExit, window::Windows};
use crate::{animation::{EaseFunction, Animator, AnimatorState, Tween, TweeningType, TransformScaleLens, TweeningDirection}, parallax::{LayerData, ParallaxResource, ParallaxCameraComponent}};
use iyes_loopless::prelude::*;

use crate::{state::GameState, TRANSPARENT, util::RectExtensions};

use super::{FontAssets, MainCamera, BackgroundAssets};

// region: Plugin
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_camera)
            .add_enter_system(GameState::MainMenu, setup_background)
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
            .add_exit_system(GameState::MainMenu, despawn_with::<Menu>)
            .add_exit_system(GameState::MainMenu, despawn_background);
    }
}
// endregion

// region: Components
#[derive(Component)]
struct SinglePlayerButton;

#[derive(Component)]
struct SettingsButton;

#[derive(Component)]
struct ExitButton;

#[derive(Component)]
struct Menu;

// endregion

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

fn despawn_background(
    mut commands: Commands,
    mut parallax: ResMut<ParallaxResource>
) {
    parallax.despawn_layers(&mut commands);
}

#[inline(always)]
fn text_tween() -> Tween<Transform> {
    Tween::new(
        EaseFunction::QuadraticInOut, 
        TweeningType::Once, 
        Duration::from_millis(200), 
        TransformScaleLens {
            start: Vec3::ONE,
            end: Vec3::splat(1.3),
        }
    )
}


const TEXT_COLOR: Color = Color::rgb(134. / 255., 134. / 255., 140. / 255.);

fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(ParallaxCameraComponent)
        .insert(MainCamera);
}

fn setup_background(
    wnds: Res<Windows>,
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>
) {
    let window = wnds.get_primary().unwrap();

    // 600 is the background image height
    let height = window.height() - 600.;

    commands.insert_resource(ParallaxResource {
        layer_data: vec![
            LayerData {
                speed: 0.9,
                image: backgrounds.background_112.clone(),
                z: 0.0,
                transition_factor: 1.,
                scale: 2.,
                position: Vec2::NEG_Y * height + 400.,
                ..default()
            },
            LayerData {
                speed: 0.9,
                image: backgrounds.background_7.clone(),
                z: 0.1,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: 0.8,
                image: backgrounds.background_90.clone(),
                z: 1.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 200.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: 0.7,
                image: backgrounds.background_91.clone(),
                z: 2.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 300.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: 0.6,
                image: backgrounds.background_92.clone(),
                z: 3.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 400.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: 0.7,
                image: backgrounds.background_112.clone(),
                z: 4.0,
                transition_factor: 1.,
                scale: 1.2,
                position: Vec2::NEG_Y * height + 200.,
                ..default()
            },
        ],
        ..default()
    });
}

fn menu_button(children: &mut ChildBuilder, text_style: TextStyle, button_name: &str, marker: impl Component) {
    children.spawn_bundle(ButtonBundle {
        style: Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::vertical(7.),
            ..default()
        },
        color: TRANSPARENT.into(),
        ..default()
    })
    .insert(Animator::new(text_tween()).with_state(AnimatorState::Paused))
    .insert(marker)
    .with_children(|c| {
        c.spawn_bundle(TextBundle::from_section(
            button_name, 
            text_style.clone()
        ));
    });
}

fn setup_main_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>
) {
    let text_style = TextStyle { 
        font: fonts.andy_bold.clone(), 
        font_size: 46., 
        color: TEXT_COLOR
    };

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
            menu_button(children, text_style.clone(), "Single Player", SinglePlayerButton);
            menu_button(children, text_style.clone(), "Settings", SettingsButton);
            menu_button(children, text_style.clone(), "Exit", ExitButton);
        });
}

fn update_buttons(
    mut text_query: Query<&mut Text>,
    mut query: Query<(&Children, &Interaction, &mut Animator<Transform>), (With<Button>, Changed<Interaction>)>
) {
    for (children, interaction, mut animator) in query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match interaction {
            Interaction::Hovered => {
                text.sections[0].style.color = Color::YELLOW;

                animator.start();

                let tweenable = animator.tweenable_mut();
                tweenable.set_progress(1. - tweenable.progress());
                tweenable.set_direction(TweeningDirection::Forward);
            },
            Interaction::None => {
                text.sections[0].style.color = TEXT_COLOR;

                let tweenable = animator.tweenable_mut();
                tweenable.set_progress(1. - tweenable.progress());
                tweenable.set_direction(TweeningDirection::Backward);
            }
            _ => {}
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