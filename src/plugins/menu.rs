use std::time::Duration;

use crate::{
    animation::{
        Animator, AnimatorState, EaseFunction, TransformScaleLens, Tween, TweeningDirection,
        TweeningType,
    },
    parallax::{move_background_system, ParallaxCameraComponent},
    util::on_btn_clicked,
};
use bevy::{
    app::AppExit,
    prelude::{
        default, App, BuildChildren, Button, ButtonBundle, Camera2dBundle, Changed, ChildBuilder,
        Children, Color, Commands, Component, DespawnRecursiveExt, Entity, EventWriter, NodeBundle,
        Plugin, Query, Res, TextBundle, Transform, Vec3, With,
    },
    text::{Text, TextStyle},
    ui::{AlignItems, FlexDirection, Interaction, JustifyContent, Size, Style, UiRect, Val},
};
use iyes_loopless::prelude::*;

use crate::{state::GameState, util::RectExtensions, TRANSPARENT};

use super::{FontAssets, MainCamera};

// region: Plugin
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_camera)
            .add_enter_system(GameState::MainMenu, setup_main_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(move_background_system())
                    .with_system(update_buttons)
                    .with_system(single_player_btn.run_if(on_btn_clicked::<SinglePlayerButton>))
                    .with_system(exit_btn.run_if(on_btn_clicked::<ExitButton>))
                    .into(),
            )
            .add_exit_system(GameState::MainMenu, despawn_with::<MainCamera>)
            .add_exit_system(GameState::MainMenu, despawn_with::<Menu>);
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

fn despawn_with<C: Component>(query: Query<Entity, With<C>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[inline(always)]
pub fn text_tween() -> Tween<Transform> {
    Tween::new(
        EaseFunction::QuadraticInOut,
        TweeningType::Once,
        Duration::from_millis(200),
        TransformScaleLens {
            start: Vec3::ONE,
            end: Vec3::splat(1.3),
        },
    )
}

const TEXT_COLOR: Color = Color::rgb(134. / 255., 134. / 255., 140. / 255.);

fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(ParallaxCameraComponent)
        .insert(MainCamera);
}

fn menu_button(
    children: &mut ChildBuilder,
    text_style: TextStyle,
    button_name: &str,
    marker: impl Component,
) {
    children
        .spawn_bundle(ButtonBundle {
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
            c.spawn_bundle(TextBundle::from_section(button_name, text_style.clone()));
        });
}

fn setup_main_menu(mut commands: Commands, fonts: Res<FontAssets>) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone(),
        font_size: 46.,
        color: TEXT_COLOR,
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
            menu_button(
                children,
                text_style.clone(),
                "Single Player",
                SinglePlayerButton,
            );
            menu_button(children, text_style.clone(), "Settings", SettingsButton);
            menu_button(children, text_style.clone(), "Exit", ExitButton);
        });
}

fn update_buttons(
    mut text_query: Query<&mut Text>,
    mut query: Query<
        (&Children, &Interaction, &mut Animator<Transform>),
        (With<Button>, Changed<Interaction>),
    >,
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
            }
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

fn single_player_btn(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::WorldLoading));
}

fn exit_btn(mut ev: EventWriter<AppExit>) {
    ev.send(AppExit);
}
