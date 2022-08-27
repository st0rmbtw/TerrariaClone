use std::time::Duration;

use bevy::{prelude::{Plugin, App, Commands, Res, Camera, With, Query, Vec2, GlobalTransform, NodeBundle, Color, default, Component, Transform, ResMut, ImageBundle, BuildChildren, Without, TextBundle, Deref, DerefMut, Vec3, Name}, window::Windows, render::camera::RenderTarget, ui::{Style, Size, Val, UiRect, PositionType, JustifyContent, AlignSelf, UiColor, AlignItems}, text::{Text, TextStyle}};
use interpolation::EaseFunction;
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet, IntoConditionalSystem};

use crate::{TRANSPARENT, lens::UiColorLens, state::GameState, animation::{component_animator_system, AnimationSystem, Tween, TweeningType, TransformScaleLens, Animator}};

use super::{MainCamera, CursorAssets, FontAssets};

// region: Plugin

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(HoveredInfo::default())
            .insert_resource(Cursor::default())
            .add_enter_system(GameState::MainMenu, setup)
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(GameState::AssetLoading)
                    .with_system(update_cursor_position)
                    .with_system(update_hovered_info_position)
                    .with_system(update_hovered_info)
                    .into()
            )
            .add_system(
                component_animator_system::<UiColor>
                    .run_not_in_state(GameState::AssetLoading)
                    .label(AnimationSystem::AnimationUpdate)
            );
    }
}

// endregion

// region: Components

#[derive(Component)]
struct CursorContainer;

#[derive(Component)]
struct CursorBackground;

#[derive(Component)]
struct CursorForeground;

#[derive(Default)]
struct Cursor {
    position: Vec2
}

#[derive(Default, Deref, DerefMut)]
pub struct HoveredInfo(pub String);

#[derive(Component)]
struct HoveredInfoMarker;

// endregion

fn setup(
    mut commands: Commands,
    cursor_assets: Res<CursorAssets>,
    fonts: Res<FontAssets>
) {
    let animate_scale = Tween::new(
        EaseFunction::QuadraticInOut,
        TweeningType::PingPong,
        Duration::from_millis(500),
        TransformScaleLens {
            start: Vec3::new(1., 1., 1.),
            end: Vec3::new(1.15, 1.15, 1.),
        },
    );

    let animate_color = Tween::new(
        EaseFunction::QuadraticInOut,
        TweeningType::PingPong,
        Duration::from_millis(500),
        UiColorLens {
            start: Color::PINK * 0.7,
            end: Color::PINK,
        }
    );

    commands.spawn_bundle(NodeBundle {
        style: Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        color: TRANSPARENT.into(),
        ..default()
    })
    .with_children(|c| {
        // region: Cursor

        c.spawn_bundle(ImageBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                size: Size::new(Val::Px(22.), Val::Px(22.)),
                ..default()
            },
            image: cursor_assets.cursor_background.clone().into(),
            color: Color::rgb(0.7, 0.7, 0.7).into(),
            ..default()
        })
        .insert(CursorBackground)
        .with_children(|c| {
            c.spawn_bundle(ImageBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Center,
                    size: Size::new(Val::Px(16.), Val::Px(16.)),
                    ..default()
                },
                image: cursor_assets.cursor.clone().into(),
                color: Color::PINK.into(),
                ..default()
            })
            .insert(CursorForeground)
            .insert(Animator::new(animate_color));
        });

        // endregion
    })
    .insert(CursorContainer)
    .insert(Name::new("Cursor Container"))
    .insert(Animator::new(animate_scale));

    commands.spawn_bundle(TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            ..default()
        },
        text: Text::from_section(
            "", 
            TextStyle {
                font: fonts.andy_regular.clone(),
                font_size: 24.,
                color: Color::WHITE.into(),
            }
        ),
        ..default()
    }).insert(HoveredInfoMarker);
}

fn update_cursor_position(
    wnds: Res<Windows>,
    mut cursor: ResMut<Cursor>,
    cemera_query: Query<(&Camera, &GlobalTransform), (With<MainCamera>, Without<CursorContainer>)>,
    mut cursor_query: Query<(&mut Style, &mut Transform, &mut GlobalTransform), With<CursorContainer>>
) {
    let (mut style, mut transform, mut global_transform) = cursor_query.single_mut();

    if let Ok((camera, camera_transform)) = cemera_query.get_single() {
        let wnd = if let RenderTarget::Window(id) = camera.target {
            wnds.get(id)
        } else {
            wnds.get_primary()
        };

        if let Some(wnd) = wnd {
            if let Some(screen_pos) = wnd.cursor_position() {
                style.position = UiRect {
                    left: Val::Px(screen_pos.x - 2.),
                    bottom: Val::Px(screen_pos.y - 20.),
                    ..default()
                };

                cursor.position = screen_pos;
            }
        }
    } 
}


fn update_hovered_info_position(
    cursor: Res<Cursor>,
    mut query: Query<&mut Style, With<HoveredInfoMarker>>
) {
    if cursor.is_changed() {
        let mut style = query.single_mut();

        style.position = UiRect {
            left: Val::Px(cursor.position.x + 20.),
            bottom: Val::Px(cursor.position.y - 45.),
            ..default()
        }
    }
}

fn update_hovered_info(
    hovered_info: Res<HoveredInfo>,
    mut query: Query<&mut Text, With<HoveredInfoMarker>>
) {
    if hovered_info.is_changed() {
        let mut text = query.single_mut();
        
        text.sections[0].value = hovered_info.0.clone();
    }
}