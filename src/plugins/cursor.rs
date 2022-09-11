use std::time::Duration;

use super::{
    CursorAssets, FontAssets, MainCamera, Player, SpeedCoefficient, UiAssets, UiVisibility, TILE_SIZE,
};
use crate::{
    animation::{
        component_animator_system, AnimationSystem, Animator, TransformScaleLens, Tween,
        TweeningType,
    },
    lens::UiColorLens,
    state::{GameState, MovementState},
    TRANSPARENT, util::get_tile_coords,
};
use autodefault::autodefault;
use bevy::{
    prelude::{
        default, App, BuildChildren, Camera, Color, Commands, Component, CoreStage, Deref,
        DerefMut, GlobalTransform, ImageBundle, Name, NodeBundle, Plugin, Query, Res, ResMut,
        TextBundle, Transform, Vec2, Vec3, Visibility, With, Without,
    },
    render::camera::RenderTarget,
    sprite::{Sprite, SpriteBundle},
    text::{Text, TextStyle},
    time::Time,
    ui::{
        AlignItems, AlignSelf, FocusPolicy, JustifyContent, PositionType, Size, Style, UiColor,
        UiRect, Val,
    },
    window::Windows,
};
use interpolation::EaseFunction;
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet, IntoConditionalSystem};

const MAX_TILE_GRID_OPACITY: f32 = 0.8;
const MIN_TILE_GRID_OPACITY: f32 = 0.2;

// region: Plugin

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HoveredInfo::default())
            .insert_resource(CursorPosition::default())
            .add_enter_system(GameState::MainMenu, setup)
            .add_enter_system(GameState::InGame, spawn_tile_grid)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(set_visibility::<TileGrid>)
                    .with_system(set_visibility::<CursorBackground>)
                    .with_system(update_tile_grid_opacity)
                    .into(),
            )
            .add_system_set_to_stage(
                CoreStage::Last,
                ConditionSet::new()
                    .run_not_in_state(GameState::AssetLoading)
                    .with_system(set_ui_component_z::<HoveredInfoMarker>)
                    .with_system(set_ui_component_z::<CursorBackground>)
                    .with_system(set_cursor_foreground_z)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(GameState::AssetLoading)
                    .run_if_resource_equals(UiVisibility(true))
                    .with_system(update_cursor_position)
                    .with_system(update_hovered_info_position)
                    .with_system(update_hovered_info)
                    .into(),
            )
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .run_if_resource_equals(UiVisibility(true))
                    .with_system(update_tile_grid_position)
                    .into(),
            )
            .add_system(
                component_animator_system::<UiColor>
                    .run_not_in_state(GameState::AssetLoading)
                    .label(AnimationSystem::AnimationUpdate),
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
pub struct CursorPosition {
    pub position: Vec2,
    pub world_position: Vec2,
}

#[derive(Default, Deref, DerefMut)]
pub struct HoveredInfo(pub String);

#[derive(Component)]
struct HoveredInfoMarker;

#[derive(Component)]
struct TileGrid;
// endregion

#[autodefault(except(TransformScaleLens, UiColorLens))]
fn setup(mut commands: Commands, cursor_assets: Res<CursorAssets>, fonts: Res<FontAssets>) {
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
        },
    );

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            focus_policy: FocusPolicy::Pass,
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
                },
                focus_policy: FocusPolicy::Pass,
                image: cursor_assets.cursor_background.clone().into(),
                color: Color::rgb(0.7, 0.7, 0.7).into(),
            })
            .insert(CursorBackground)
            .with_children(|c| {
                c.spawn_bundle(ImageBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        size: Size::new(Val::Px(16.), Val::Px(16.)),
                    },
                    focus_policy: FocusPolicy::Pass,
                    image: cursor_assets.cursor.clone().into(),
                    color: Color::PINK.into(),
                })
                .insert(CursorForeground)
                .insert(Animator::new(animate_color));
            });

            // endregion
        })
        .insert(CursorContainer)
        .insert(Name::new("Cursor Container"))
        .insert(Animator::new(animate_scale));

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
            },
            text: Text::from_section(
                "",
                TextStyle {
                    font: fonts.andy_bold.clone(),
                    font_size: 22.,
                    color: Color::WHITE.into(),
                },
            ),
        })
        .insert(HoveredInfoMarker);
}

#[autodefault]
fn spawn_tile_grid(mut commands: Commands, ui_assets: Res<UiAssets>) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1., 1., 1., MAX_TILE_GRID_OPACITY),
            },
            texture: ui_assets.radial.clone().into(),
            transform: Transform::from_xyz(0., 0., 1.),
        })
        .insert(TileGrid);
}

fn set_ui_component_z<C: Component>(
    mut query: Query<
        (&mut Transform, &mut GlobalTransform),
        With<C>,
    >,
) {
    let (mut transform, mut global_transform) = query.single_mut();

    transform.translation.z = 10.;
    global_transform.translation_mut().z = 10.;
}
 
fn set_cursor_foreground_z(
    mut cursor_foreground_query: Query<
        (&mut Transform, &mut GlobalTransform),
        (With<CursorForeground>, Without<CursorBackground>),
    >,
) {
    let (mut cursor_foreground_transform, mut cursor_foreground_global_transform) =
        cursor_foreground_query.single_mut();

    cursor_foreground_transform.translation.z = 10.1;
    cursor_foreground_global_transform.translation_mut().z = 10.1;
}

fn update_cursor_position(
    wnds: Res<Windows>,
    mut cursor: ResMut<CursorPosition>,
    cemera_query: Query<(&Camera, &GlobalTransform), (With<MainCamera>, Without<CursorContainer>)>,
    mut cursor_query: Query<&mut Style, With<CursorContainer>>,
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
                let ndc_to_world =
                    camera_transform.compute_matrix() * camera.projection_matrix().inverse();

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

fn set_visibility<C: Component>(
    ui_visibility: Res<UiVisibility>,
    mut query: Query<&mut Visibility, With<C>>,
) {
    if ui_visibility.is_changed() {
        for mut visibility in &mut query {
            visibility.is_visible = ui_visibility.0;
        }
    }
}

fn update_hovered_info_position(
    cursor: Res<CursorPosition>,
    mut query: Query<&mut Style, With<HoveredInfoMarker>>,
) {
    let mut style = query.single_mut();

    style.position = UiRect {
        left: Val::Px(cursor.position.x + 20.),
        bottom: Val::Px(cursor.position.y - 45.),
        ..default()
    }
}

fn update_hovered_info(
    hovered_info: Res<HoveredInfo>,
    mut query: Query<&mut Text, With<HoveredInfoMarker>>,
) {
    if hovered_info.is_changed() {
        let mut text = query.single_mut();

        text.sections[0].value = hovered_info.0.clone();
    }
}

fn update_tile_grid_position(
    cursor: Res<CursorPosition>,
    mut query: Query<&mut Transform, With<TileGrid>>,
) {
    let mut transform = query.single_mut();
    
    let tile_coords = get_tile_coords(cursor.world_position);

    transform.translation.x = tile_coords.x * TILE_SIZE;
    transform.translation.y = tile_coords.y * TILE_SIZE;
}

fn update_tile_grid_opacity(
    time: Res<Time>,
    player: Query<(&SpeedCoefficient, &MovementState), With<Player>>,
    mut tile_grid: Query<&mut Sprite, With<TileGrid>>,
) {
    if let Ok((SpeedCoefficient(speed_coefficient), movement_state)) = player.get_single() {
        let mut sprite = tile_grid.single_mut();

        let opacity = match movement_state {
            MovementState::WALKING => {
                let mut a = sprite.color.a();

                if a > MIN_TILE_GRID_OPACITY {
                    a = a - speed_coefficient * time.delta_seconds() * 0.7;
                } else if a < MIN_TILE_GRID_OPACITY {
                    a = a + speed_coefficient * time.delta_seconds() * 0.7;
                }

                a.clamp(0., MAX_TILE_GRID_OPACITY)
            }
            MovementState::FLYING | MovementState::FALLING => {
                let mut a = sprite.color.a();

                if a > 0. {
                    a = (a - time.delta_seconds() * 0.7).clamp(0., MAX_TILE_GRID_OPACITY);
                }

                a
            }
            _ => {
                let mut a = sprite.color.a();

                if a < MAX_TILE_GRID_OPACITY {
                    a = (a + time.delta_seconds() * 0.7).clamp(0., MAX_TILE_GRID_OPACITY);
                }

                a
            }
        };

        sprite.color = *sprite.color.set_a(opacity);
    }
}
