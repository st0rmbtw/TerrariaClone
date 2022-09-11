use std::time::Duration;

use crate::{
    animation::{
        Animator, AnimatorState, EaseFunction, Tween, TweeningDirection,
        TweeningType,
    },
    parallax::{move_background_system, ParallaxCameraComponent},
    util::on_btn_clicked, lens::TextFontSizeLens, TEXT_COLOR,
};
use autodefault::autodefault;
use bevy::{
    app::AppExit,
    prelude::{
        default, App, BuildChildren, Button, Camera2dBundle, Changed, ChildBuilder,
        Color, Commands, Component, DespawnRecursiveExt, Entity, EventWriter, NodeBundle,
        Plugin, Query, Res, TextBundle, With,
    },
    text::{Text, TextStyle},
    ui::{AlignItems, FlexDirection, Interaction, JustifyContent, Size, Style, UiRect, Val, FocusPolicy, PositionType},
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
pub fn text_tween() -> Tween<Text> {
    Tween::new(
        EaseFunction::QuadraticInOut,
        TweeningType::Once,
        Duration::from_millis(200),
        TextFontSizeLens {
            start: 48.,
            end: 52.,
        },
    )
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(ParallaxCameraComponent)
        .insert(MainCamera);
}

#[autodefault]
fn menu_button(
    children: &mut ChildBuilder,
    text_style: TextStyle,
    button_name: &str,
    marker: impl Component,
) {
    children
        .spawn_bundle(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::vertical(25.),
            },
            color: TRANSPARENT.into(),
            focus_policy: FocusPolicy::Pass
        })
        .with_children(|c| {
            c.spawn_bundle(TextBundle {
                style: Style {
                    position_type: PositionType::Absolute
                },
                text: Text::from_section(button_name, text_style.clone()),
            })
            .insert(Button)
            .insert(Interaction::default())
            .insert(Animator::new(text_tween()).with_state(AnimatorState::Paused))
            .insert(marker);
        });
}

fn setup_main_menu(mut commands: Commands, fonts: Res<FontAssets>) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone(),
        font_size: 48.,
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
    mut query: Query<
        (&Interaction, &mut Text, &mut Animator<Text>),
        (With<Button>, Changed<Interaction>),
    >,
) {
    for (interaction, mut text, mut animator) in query.iter_mut() {

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
