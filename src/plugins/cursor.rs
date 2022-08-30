use std::time::Duration;

use autodefault::autodefault;
use bevy::{
    prelude::{Plugin, App, Commands, Res, Camera, With, Query, Vec2, GlobalTransform, NodeBundle, Color, default, Component, ResMut, ImageBundle, BuildChildren, Without, TextBundle, Deref, DerefMut, Vec3, Name, Transform}, 
    window::Windows,
    render::camera::RenderTarget, 
    ui::{Style, Size, Val, UiRect, PositionType, JustifyContent, AlignSelf, UiColor, AlignItems}, 
    text::{Text, TextStyle}, sprite::SpriteBundle
};
use interpolation::EaseFunction;
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet, IntoConditionalSystem};

use crate::{TRANSPARENT, lens::UiColorLens, state::GameState, animation::{component_animator_system, AnimationSystem, Tween, TweeningType, TransformScaleLens, Animator}};

use super::{MainCamera, CursorAssets, FontAssets, UiAssets};

// region: Plugin

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(HoveredInfo::default())
            .insert_resource(CursorPosition::default())
            .add_enter_system(GameState::MainMenu, setup)
            .add_enter_system(GameState::InGame, spawn_tile_grid)
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(GameState::AssetLoading)
                    .with_system(update_cursor_position)
                    .with_system(update_hovered_info_position)
                    .with_system(update_hovered_info)
                    .into()
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(update_tile_grid_position)
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
struct CursorPosition {
    position: Vec2,
    world_position: Vec2
}

#[derive(Default, Deref, DerefMut)]
pub struct HoveredInfo(pub String);

#[derive(Component)]
struct HoveredInfoMarker;

#[derive(Component)]
struct TileGrid;
// endregion

#[autodefault(except(TransformScaleLens, UiColorLens))]
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
                size: Size::new(Val::Px(22.), Val::Px(22.))
            },
            image: cursor_assets.cursor_background.clone().into(),
            color: Color::rgb(0.7, 0.7, 0.7).into()
        })
        .insert(CursorBackground)
        .with_children(|c| {
            c.spawn_bundle(ImageBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Center,
                    size: Size::new(Val::Px(16.), Val::Px(16.))
                },
                image: cursor_assets.cursor.clone().into(),
                color: Color::PINK.into()
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
            position_type: PositionType::Absolute
        },
        text: Text::from_section(
            "", 
            TextStyle {
                font: fonts.andy_regular.clone(),
                font_size: 24.,
                color: Color::WHITE.into(),
            }
        )
    }).insert(HoveredInfoMarker);
}

#[autodefault]
fn spawn_tile_grid(
    mut commands: Commands,
    ui_assets: Res<UiAssets>
) {
    commands.spawn_bundle(SpriteBundle {
        texture: ui_assets.radial.clone().into(),
        transform: Transform::from_xyz(0., 0., 1.)
    })
    .insert(TileGrid);
}

fn update_cursor_position(
    wnds: Res<Windows>,
    mut cursor: ResMut<CursorPosition>,
    cemera_query: Query<(&Camera, &GlobalTransform), (With<MainCamera>, Without<CursorContainer>)>,
    mut cursor_query: Query<&mut Style, With<CursorContainer>>
) {
    let mut style = cursor_query.single_mut();

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
                
                let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
                
                // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
                let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
                
                // matrix for undoing the projection and camera transform
                let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
                
                // use it to convert ndc to world-space coordinates
                let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
                
                // reduce it to a 2D value
                let world_pos: Vec2 = world_pos.truncate();
                
                cursor.position = screen_pos;
                cursor.world_position = world_pos;
            }
        }
    }
}


fn update_hovered_info_position(
    cursor: Res<CursorPosition>,
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

fn update_tile_grid_position(
    cursor: Res<CursorPosition>,
    mut query: Query<&mut Transform, With<TileGrid>>
) {
    if cursor.is_changed() {
        let mut transform = query.single_mut();

        let x = cursor.world_position.x + 5.;
        let y = cursor.world_position.y;

        transform.translation.x = x - x % 16.;
        transform.translation.y = y - y % 16.;
    }
}