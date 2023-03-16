use std::time::Duration;

use autodefault::autodefault;
use bevy::{
    prelude::{
        Res, Commands, Vec3, Color, NodeBundle, default, TextBundle, Name, ImageBundle, Transform, 
        Component, GlobalTransform, Query, With, ResMut, Camera, Visibility, 
        BuildChildren, DetectChanges
    }, 
    ui::{
        Style, JustifyContent, AlignItems, PositionType, FocusPolicy, Size, Val, AlignSelf, ZIndex, FlexDirection, UiRect
    }, 
    text::{Text, TextStyle}, 
    sprite::{SpriteBundle, Sprite}, 
    window::{Window, PrimaryWindow}
};
use interpolation::EaseFunction;

use crate::{
    plugins::{
        assets::{FontAssets, CursorAssets, UiAssets}, 
        camera::MainCamera, 
        ui::UiVisibility, 
        world::TILE_SIZE, settings::{ShowTileGrid, CursorColor}
    }, 
    animation::{Tween, lens::TransformScaleLens, Animator, RepeatStrategy, RepeatCount}, 
    lens::BackgroundColorLens, util,
};

use crate::plugins::player::{PlayerVelocity, MAX_RUN_SPEED, MAX_FALL_SPEED};

use super::{HoveredInfoMarker, CursorContainer, CursorForeground, CursorBackground, TileGrid, MAX_TILE_GRID_OPACITY, CursorPosition, HoveredInfo, MIN_TILE_GRID_OPACITY};

#[autodefault(except(TransformScaleLens, BackgroundColorLens))]
pub fn setup(
    mut commands: Commands, 
    cursor_assets: Res<CursorAssets>, 
    fonts: Res<FontAssets>,
    cursor_color: Res<CursorColor>
) {
    let animate_scale = Tween::new(
        EaseFunction::QuadraticInOut,
        RepeatStrategy::MirroredRepeat,
        Duration::from_millis(500),
        TransformScaleLens {
            start: Vec3::new(1., 1., 1.),
            end: Vec3::new(1.15, 1.15, 1.),
        },
    )
    .with_repeat_count(RepeatCount::Infinite);

    let animate_color = Tween::new(
        EaseFunction::QuadraticInOut,
        RepeatStrategy::MirroredRepeat,
        Duration::from_millis(500),
        BackgroundColorLens {
            start: cursor_color.foreground_color * 0.7,
            end: cursor_color.foreground_color,
        },
    ).with_repeat_count(RepeatCount::Infinite);

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            focus_policy: FocusPolicy::Pass,
            z_index: ZIndex::Global(i32::MAX),
            ..default()
        })
        .with_children(|c| {
            // region: Cursor

            const CURSOR_SIZE: f32 = 22.;

            c.spawn(ImageBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::FlexStart,
                    size: Size::new(Val::Px(CURSOR_SIZE), Val::Px(CURSOR_SIZE)),
                },
                focus_policy: FocusPolicy::Pass,
                image: cursor_assets.cursor_background.clone_weak().into(),
                background_color: cursor_color.background_color.into(),
            })
            .insert(CursorBackground)
            .insert(Animator::new(animate_scale))
            .with_children(|c| {
                c.spawn(ImageBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        size: Size::new(Val::Px(16.), Val::Px(16.)),
                    },
                    focus_policy: FocusPolicy::Pass,
                    image: cursor_assets.cursor.clone_weak().into(),
                    background_color: cursor_color.foreground_color.into(),
                })
                .insert(CursorForeground)
                .insert(Animator::new(animate_color));
            });

            // endregion

            c.spawn(TextBundle {
                style: Style {
                    align_self: AlignSelf::FlexEnd,
                    margin: UiRect::left(Val::Px(CURSOR_SIZE))
                },
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: fonts.andy_bold.clone_weak(),
                        font_size: 22.,
                        color: Color::WHITE,
                    },
                )
            })
            .insert(HoveredInfoMarker);
        })
        .insert(CursorContainer)
        .insert(Name::new("Cursor Container"));
}

#[autodefault]
pub fn spawn_tile_grid(
    mut commands: Commands, 
    ui_assets: Res<UiAssets>
) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1., 1., 1., MAX_TILE_GRID_OPACITY),
            },
            texture: ui_assets.radial.clone_weak(),
            transform: Transform::from_xyz(0., 0., 5.),
            visibility: Visibility::Hidden
        })
        .insert(TileGrid);
}

pub fn update_cursor_position(
    query_windows: Query<&Window, With<PrimaryWindow>>,
    mut cursor: ResMut<CursorPosition>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut cursor_query: Query<&mut Style, With<CursorContainer>>,
) {
    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        let window = query_windows.single();

        if let Some(screen_pos) = window.cursor_position() {
            if let Ok(mut style) = cursor_query.get_single_mut() {
                style.position.left = Val::Px(screen_pos.x);
                style.position.top = Val::Px(window.height() - screen_pos.y);
            }

            if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
                cursor.position = screen_pos;
                cursor.world_position = world_pos;
            }
        }
    }
}

pub fn set_visibility<C: Component>(
    ui_visibility: Res<UiVisibility>,
    mut query: Query<&mut Visibility, With<C>>,
) {
    if ui_visibility.is_changed() {
        for visibility in &mut query {
            util::set_visibility(visibility, ui_visibility.0);
        }
    }
}

pub fn update_hovered_info(
    hovered_info: Res<HoveredInfo>,
    mut query: Query<&mut Text, With<HoveredInfoMarker>>,
) {
    if hovered_info.is_changed() {
        let mut text = query.single_mut();
        text.sections[0].value = hovered_info.0.clone();
    }
}

pub fn update_tile_grid_position(
    cursor: Res<CursorPosition>,
    mut query: Query<&mut Transform, With<TileGrid>>,
) {
    let mut transform = query.single_mut();
    
    let tile_coords = (cursor.world_position / TILE_SIZE).round();
    transform.translation.x = tile_coords.x * TILE_SIZE;
    transform.translation.y = tile_coords.y * TILE_SIZE;
}

pub fn update_tile_grid_opacity(
    velocity: Res<PlayerVelocity>,
    mut tile_grid: Query<&mut Sprite, With<TileGrid>>,
) {
    use interpolation::Lerp;

    let mut sprite = tile_grid.single_mut();

    let opacity = if velocity.x.abs() > 0. {
        MIN_TILE_GRID_OPACITY.lerp(&MAX_TILE_GRID_OPACITY, &(1. - velocity.x.abs() / MAX_RUN_SPEED))
    } else if velocity.y.abs() > 0. {
        0f32.lerp(&MAX_TILE_GRID_OPACITY, &(1. - velocity.y.abs() / MAX_FALL_SPEED))
    } else {
        MAX_TILE_GRID_OPACITY
    };

    sprite.color = *sprite.color.set_a(opacity.clamp(0., MAX_TILE_GRID_OPACITY));
}

pub fn update_tile_grid_visibility(
    mut tile_grid: Query<&mut Visibility, With<TileGrid>>,
    show_tile_grid: Res<ShowTileGrid>
) {
    let visibility = tile_grid.single_mut();
    util::set_visibility(visibility, show_tile_grid.0);
}