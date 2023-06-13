use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Component, Query, Entity, With, Commands, DespawnRecursiveExt, Camera2dBundle, ChildBuilder, NodeBundle, BuildChildren, TextBundle, Button, Res, default, Changed, EventWriter, Color, ImageBundle, Transform, Quat, Vec3, Audio, NextState, ResMut, Visibility, Name, Camera2d}, text::{Text, TextStyle, TextSection}, ui::{Style, JustifyContent, AlignItems, UiRect, FocusPolicy, PositionType, Interaction, Size, Val, FlexDirection, AlignSelf, UiImage}, app::AppExit, core_pipeline::clear_color::ClearColorConfig};
use interpolation::EaseFunction;

use crate::{animation::{Tween, Animator, AnimatorState, TweeningDirection, RepeatStrategy, Tweenable, EaseMethod, RepeatCount}, plugins::{camera::MainCamera, assets::{FontAssets, UiAssets, SoundAssets}, settings::{Settings, FullScreen, ShowTileGrid, VSync, Resolution, CursorColor}, fps::FpsText}, common::{state::{GameState, SettingsMenuState, MenuState}, lens::{TextFontSizeLens, TransformLens}}, language::LanguageContent};
use super::{Menu, SinglePlayerButton, SettingsButton, ExitButton, MenuContainer, role::ButtonRole, settings::MENU_BUTTON_FONT_SIZE, TEXT_COLOR};

pub(super) fn despawn_with<C: Component>(query: Query<Entity, With<C>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[inline(always)]
fn text_tween(initial_font_size: f32) -> Tween<Text> {
    Tween::new(
        EaseMethod::Linear,
        RepeatStrategy::MirroredRepeat,
        Duration::from_millis(200),
        TextFontSizeLens {
            start: initial_font_size,
            end: initial_font_size * 1.2,
        },
    )
}

pub(super) fn setup_camera(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None
            },
            ..default()
        })
        .insert(MainCamera);
}

#[autodefault]
pub(super) fn menu_button(
    builder: &mut ChildBuilder,
    text_style: TextStyle,
    button_name: String,
    marker: impl Component,
) {
    builder
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
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
            .insert(marker)
            .insert(ButtonRole::MenuButton);
        });
}

#[autodefault]
pub(super) fn control_buttons_layout(
    builder: &mut ChildBuilder,
    spawn_builder: impl FnOnce(&mut ChildBuilder)
) {
    builder
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                margin: UiRect::vertical(Val::Px(40.)),
                gap: Size::new(Val::Px(0.), Val::Px(50.)),
            },
            focus_policy: FocusPolicy::Pass
        }).with_children(spawn_builder);
}

#[autodefault]
pub(super) fn control_button(
    builder: &mut ChildBuilder,
    text_style: TextStyle,
    name: String,
    marker: impl Component
) {
    builder
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            },
            focus_policy: FocusPolicy::Pass
        })
        .with_children(|b| {
            b.spawn(TextBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                },
                text: Text::from_section(name, TextStyle { font_size: MENU_BUTTON_FONT_SIZE, ..text_style }),
            })
            .insert(Button)
            .insert(Interaction::default())
            .insert(Animator::new(text_tween(MENU_BUTTON_FONT_SIZE)).with_state(AnimatorState::Paused))
            .insert(marker)
            .insert(ButtonRole::ControlButton);
        });
}

pub(super) fn menu(marker: impl Component, commands: &mut Commands, container: Entity, gap: f32, spawn_children: impl FnOnce(&mut ChildBuilder)) {
    let menu = commands.spawn((
        NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                gap: Size::new(Val::Px(0.), Val::Px(gap)),
                ..default()
            },
            ..default()
        },
        Menu
    ))
    .insert(marker)
    .with_children(spawn_children)
    .id();

    commands.entity(container)
        .add_child(menu);
}

pub(super) fn spawn_menu_container(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    font_assets: Res<FontAssets>
) {
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

    let fps_text_style = TextStyle {
        font: font_assets.andy_regular.clone_weak(),
        font_size: 24.,
        color: Color::WHITE,
    };

    commands.spawn((
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                margin: UiRect {
                    left: Val::Px(5.),
                    top: Val::Px(5.),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![TextSection {
                    value: "".to_string(),
                    style: fps_text_style,
                }],
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
        FpsText,
        Name::new("FPS Text")
    ));

    commands
        .spawn((
            NodeBundle {
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
            },
            MenuContainer
        ))
        .with_children(|children| {
            children.spawn((
                ImageBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                    image: UiImage {
                        texture: ui_assets.logo.clone_weak(),
                        ..default()
                    },
                    ..default()
                },
                Animator::new(logo_animation)
            ));
        });
}

pub(super) fn setup_main_menu(
    mut commands: Commands, 
    fonts: Res<FontAssets>,
    language_content: Res<LanguageContent>,
    query_container: Query<Entity, With<MenuContainer>>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: 56.,
        color: TEXT_COLOR,
    };

    let container = query_container.single();

    menu(Menu, &mut commands, container, 65., |builder| {
        menu_button(
            builder,
            text_style.clone(),
            language_content.ui.single_player.clone(),
            SinglePlayerButton,
        );
        menu_button(
            builder, 
            text_style.clone(), 
            language_content.ui.settings.clone(), 
            SettingsButton,
        );
        menu_button(
            builder, 
            text_style.clone(), 
            language_content.ui.exit.clone(), 
            ExitButton,
        );
    });
}

pub(super) fn update_buttons(
    mut query: Query<
        (&Interaction, &mut Text, &mut Animator<Text>, &ButtonRole),
        (With<Button>, Changed<Interaction>),
    >,
    audio: Res<Audio>,
    sounds: Res<SoundAssets>
) {
    for (interaction, mut text, mut animator, role) in query.iter_mut() {
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
            Interaction::Clicked => {
                let sound = match role {
                    ButtonRole::MenuButton => &sounds.menu_open,
                    ButtonRole::ControlButton => &sounds.menu_close,
                };
                audio.play(sound.clone_weak());
            }
        }
    }
}

pub(super) fn single_player_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::WorldLoading);
}

pub(super) fn settings_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu(MenuState::Settings(SettingsMenuState::Main)));
}

pub(super) fn exit_clicked(
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