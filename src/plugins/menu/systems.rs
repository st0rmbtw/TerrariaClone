use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Component, Query, Entity, With, Commands, DespawnRecursiveExt, Camera2dBundle, ChildBuilder, NodeBundle, BuildChildren, TextBundle, Button, Res, default, Changed, EventWriter, Color}, text::{Text, TextStyle}, ui::{Style, JustifyContent, AlignItems, UiRect, FocusPolicy, PositionType, Interaction, Size, Val, FlexDirection}, app::AppExit};
use interpolation::EaseFunction;
use iyes_loopless::state::NextState;

use crate::{animation::{Tween, TweeningType, Animator, AnimatorState, TweeningDirection}, lens::TextFontSizeLens, parallax::ParallaxCameraComponent, plugins::{camera::MainCamera, assets::FontAssets}, TRANSPARENT, util::RectExtensions, TEXT_COLOR, state::GameState};

use super::{Menu, SinglePlayerButton, SettingsButton, ExitButton};

pub fn despawn_with<C: Component>(query: Query<Entity, With<C>>, mut commands: Commands) {
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

pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(ParallaxCameraComponent)
        .insert(MainCamera);
}

#[autodefault]
pub fn menu_button(
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

pub fn setup_main_menu(mut commands: Commands, fonts: Res<FontAssets>) {
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

pub fn update_buttons(
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

pub fn single_player_btn(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::WorldLoading));
}

pub fn exit_btn(mut ev: EventWriter<AppExit>) {
    ev.send(AppExit);
}