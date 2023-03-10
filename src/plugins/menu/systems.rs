use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Component, Query, Entity, With, Commands, DespawnRecursiveExt, Camera2dBundle, ChildBuilder, NodeBundle, BuildChildren, TextBundle, Button, Res, default, Changed, EventWriter, Color, ImageBundle, Transform, Quat, Vec3, Audio}, text::{Text, TextStyle}, ui::{Style, JustifyContent, AlignItems, UiRect, FocusPolicy, PositionType, Interaction, Size, Val, FlexDirection}, app::AppExit};
use interpolation::EaseFunction;
use iyes_loopless::state::NextState;

use crate::{animation::{Tween, Animator, AnimatorState, TweeningDirection, RepeatStrategy, Tweenable, EaseMethod, RepeatCount}, lens::{TextFontSizeLens, TransformLens}, parallax::ParallaxCameraComponent, plugins::{camera::MainCamera, assets::{FontAssets, UiAssets, SoundAssets}, settings::{Settings, FullScreen, ShowTileGrid, VSync, Resolution, CursorColor}}, TEXT_COLOR, state::GameState, language::LanguageContent};

use super::{Menu, SinglePlayerButton, SettingsButton, ExitButton};

pub fn despawn_with<C: Component>(query: Query<Entity, With<C>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[inline(always)]
pub fn text_tween(initial_font_size: f32) -> Tween<Text> {
    Tween::new(
        EaseMethod::Linear,
        RepeatStrategy::MirroredRepeat,
        Duration::from_millis(200),
        TextFontSizeLens {
            start: initial_font_size,
            end: initial_font_size * 1.1,
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
    builder: &mut ChildBuilder,
    text_style: TextStyle,
    button_name: String,
    marker: impl Component,
    margin: Option<f32>
) {
    let margin = margin.unwrap_or(25.);

    builder
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::vertical(Val::Px(margin)),
            },
            focus_policy: FocusPolicy::Pass
        })
        .with_children(|b| {
            b.spawn(TextBundle {
                style: Style {
                    position_type: PositionType::Absolute
                },
                text: Text::from_section(button_name, text_style.clone()),
            })
            .insert(Button)
            .insert(Interaction::default())
            .insert(Animator::new(text_tween(text_style.font_size)).with_state(AnimatorState::Paused))
            .insert(marker);
        });
}

#[autodefault]
pub fn control_buttons_layout(
    builder: &mut ChildBuilder,
    spawn_builder: impl FnOnce(&mut ChildBuilder)
) {
    builder
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                margin: UiRect::top(Val::Px(10.)),
            },
            focus_policy: FocusPolicy::Pass
        }).with_children(spawn_builder);
}

#[autodefault]
pub fn control_button(
    builder: &mut ChildBuilder,
    text_style: TextStyle,
    name: String,
    marker: impl Component
) {
    const FONT_SIZE: f32 = 52.;

    builder
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::vertical(Val::Px(40.)),
            },
            focus_policy: FocusPolicy::Pass
        })
        .with_children(|b| {
            b.spawn(TextBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                },
                text: Text::from_section(name, TextStyle { font_size: FONT_SIZE, ..text_style }),
            })
            .insert(Button)
            .insert(Interaction::default())
            .insert(Animator::new(text_tween(FONT_SIZE)).with_state(AnimatorState::Paused))
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
        font: fonts.andy_bold.clone_weak(),
        font_size: 54.,
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
        .with_children(|builder| {
            menu_button(
                builder,
                text_style.clone(),
                language_content.ui.single_player.clone(),
                SinglePlayerButton,
                Some(30.)
            );
            menu_button(
                builder, 
                text_style.clone(), 
                language_content.ui.settings.clone(), 
                SettingsButton,
                Some(30.)
            );
            menu_button(
                builder, 
                text_style.clone(), 
                language_content.ui.exit.clone(), 
                ExitButton,
                Some(30.)
            );
        });
}

pub fn update_buttons(
    mut query: Query<
        (&Interaction, &mut Text, &mut Animator<Text>),
        (With<Button>, Changed<Interaction>),
    >,
    audio: Res<Audio>,
    sounds: Res<SoundAssets>
) {
    for (interaction, mut text, mut animator) in query.iter_mut() {

        match interaction {
            Interaction::Hovered => {
                audio.play(sounds.menu_tick.clone_weak());

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

pub fn single_player_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::WorldLoading));
}

pub fn settings_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::Settings));
}

pub fn exit_clicked(
    mut ev: EventWriter<AppExit>,
    fullscreen: Res<FullScreen>,
    show_tile_grid: Res<ShowTileGrid>,
    vsync: Res<VSync>,
    resolution: Res<Resolution>,
    cursor_color: Res<CursorColor>,
) {
    ev.send(AppExit);

    crate::plugins::settings::save_settings(Settings {
        full_screen: fullscreen.0,
        show_tile_grid: show_tile_grid.0,
        vsync: vsync.0,
        resolution: *resolution,
        cursor_color: *cursor_color
    });
}