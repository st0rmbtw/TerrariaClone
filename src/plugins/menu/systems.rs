use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Component, Query, Entity, With, Commands, DespawnRecursiveExt, Camera2dBundle, ChildBuilder, NodeBundle, BuildChildren, TextBundle, Button, Res, default, Changed, EventWriter, Color, ImageBundle, Transform, Quat, Vec3}, text::{Text, TextStyle}, ui::{Style, JustifyContent, AlignItems, UiRect, FocusPolicy, PositionType, Interaction, Size, Val, FlexDirection, UiImage, AlignSelf}, app::AppExit};
use interpolation::EaseFunction;
use iyes_loopless::state::NextState;

use crate::{animation::{Tween, Animator, AnimatorState, TweeningDirection, RepeatStrategy, Tweenable, RepeatCount}, lens::{TextFontSizeLens, TransformLens}, parallax::ParallaxCameraComponent, plugins::{camera::MainCamera, assets::{FontAssets, UiAssets}}, TEXT_COLOR, state::GameState, language::LanguageContent};

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
        RepeatStrategy::MirroredRepeat,
        Duration::from_millis(200),
        TextFontSizeLens {
            start: 48.,
            end: 52.,
        },
    )
}

pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(ParallaxCameraComponent)
        .insert(MainCamera);
}

#[autodefault]
pub fn menu_button(
    children: &mut ChildBuilder,
    text_style: TextStyle,
    button_name: String,
    marker: impl Component,
) {
    children
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::vertical(Val::Px(25.)),
            },
            focus_policy: FocusPolicy::Pass
        })
        .with_children(|c| {
            c.spawn(TextBundle {
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

pub fn setup_main_menu(
    mut commands: Commands, 
    fonts: Res<FontAssets>,
    language_content: Res<LanguageContent>,
    ui_assets: Res<UiAssets>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone(),
        font_size: 48.,
        color: TEXT_COLOR,
    };

    let logo_animation = Tween::new(
        EaseFunction::SineInOut,
        RepeatStrategy::MirroredRepeat,
        Duration::from_secs(10),
        TransformLens {
            start: Transform {
                scale: Vec3::splat(1.),
                rotation: Quat::from_rotation_z(-5f32.to_radians()),
                ..default()
            },
            end: Transform {
                scale: Vec3::splat(1.1),
                rotation: Quat::from_rotation_z(5f32.to_radians()),
                ..default()
            }
        }
    ).with_repeat_count(RepeatCount::Infinite);

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                padding: UiRect::all(Val::Px(50.)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .insert(Menu)
        .with_children(|children| {
            children.spawn((
                ImageBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                    image: UiImage(ui_assets.logo.clone()),
                    ..default()
                },
                Animator::new(logo_animation)
            ));

            children.spawn(NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                    },
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            }).with_children(|children| {    
                menu_button(
                    children,
                    text_style.clone(),
                    language_content.ui.single_player.clone(),
                    SinglePlayerButton,
                );
                menu_button(
                    children, 
                    text_style.clone(), 
                    language_content.ui.settings.clone(), 
                    SettingsButton
                );
                menu_button(
                    children, 
                    text_style.clone(), 
                    language_content.ui.exit.clone(), 
                    ExitButton
                );
            });
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

                let tweenable = animator.tweenable_mut().as_any_mut().downcast_mut::<Tween<Text>>().unwrap();
                tweenable.set_progress(1. - tweenable.progress());
                tweenable.set_direction(TweeningDirection::Forward);
            }
            Interaction::None => {
                text.sections[0].style.color = TEXT_COLOR;

                let tweenable = animator.tweenable_mut().as_any_mut().downcast_mut::<Tween<Text>>().unwrap();
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