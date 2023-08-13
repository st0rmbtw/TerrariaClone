use std::time::Duration;

use autodefault::autodefault;
use bevy::{
    prelude::{
        Res, Commands, Vec3, Color, NodeBundle, default, TextBundle, Name, ImageBundle, Transform, 
        GlobalTransform, Query, With, ResMut, Camera, Visibility, 
        BuildChildren, Changed
    }, 
    ui::{
        Style, JustifyContent, AlignItems, PositionType, FocusPolicy, Val, AlignSelf, ZIndex, FlexDirection, UiRect, Interaction
    }, 
    text::{Text, TextStyle}, 
    sprite::{SpriteBundle, Sprite}, 
    window::{Window, PrimaryWindow}
};
use interpolation::EaseFunction;

use crate::{
    plugins::{
        assets::{FontAssets, CursorAssets, UiAssets}, 
        camera::components::MainCamera, 
        world::constants::TILE_SIZE, settings::CursorColor
    }, 
    animation::{Tween, lens::TransformScaleLens, Animator, RepeatStrategy, RepeatCount}, 
    common::{lens::BackgroundColorLens, helpers},
};

use crate::plugins::player::{PlayerVelocity, MAX_RUN_SPEED, MAX_FALL_SPEED};

use super::{MAX_TILE_GRID_OPACITY, MIN_TILE_GRID_OPACITY, CURSOR_SIZE, components::{Hoverable, CursorBackground, CursorForeground, CursorInfoMarker, CursorContainer, TileGrid}, resources::CursorPosition};

pub(super) fn setup(
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

            c.spawn(ImageBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::FlexStart,
                    width: Val::Px(CURSOR_SIZE),
                    height: Val::Px(CURSOR_SIZE),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                image: cursor_assets.cursor_background.clone_weak().into(),
                background_color: cursor_color.background_color.into(),
                ..default()
            })
            .insert(CursorBackground)
            .insert(Animator::new(animate_scale))
            .with_children(|c| {
                c.spawn(ImageBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        width: Val::Px(16.),
                        height: Val::Px(16.),
                        ..default()
                    },
                    focus_policy: FocusPolicy::Pass,
                    image: cursor_assets.cursor.clone_weak().into(),
                    background_color: cursor_color.foreground_color.into(),
                    ..default()
                })
                .insert(CursorForeground)
                .insert(Animator::new(animate_color));
            });

            // endregion

            c.spawn(TextBundle {
                style: Style {
                    align_self: AlignSelf::FlexEnd,
                    margin: UiRect::left(Val::Px(CURSOR_SIZE)),
                    ..default()
                },
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: fonts.andy_bold.clone_weak(),
                        font_size: 22.,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            })
            .insert(CursorInfoMarker);
        })
        .insert(CursorContainer)
        .insert(Name::new("Cursor Container"));
}

#[autodefault]
pub(super) fn spawn_tile_grid(
    mut commands: Commands, 
    ui_assets: Res<UiAssets>
) {
    commands.spawn((
        Name::new("TileGrid"),
        TileGrid,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1., 1., 1., MAX_TILE_GRID_OPACITY),
            },
            texture: ui_assets.radial.clone_weak(),
            transform: Transform::from_xyz(0., 0., 5.),
            visibility: Visibility::Hidden
        }
    ));
}

pub(super) fn update_cursor_position(
    mut cursor: ResMut<CursorPosition>,
    query_window: Query<&Window, With<PrimaryWindow>>,
    query_main_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query_cursor: Query<&mut Style, With<CursorContainer>>,
) {
    if let Ok((camera, camera_transform)) = query_main_camera.get_single() {
        let window = query_window.single();

        let Some(screen_pos) = window.cursor_position() else { return; };

        if let Ok(mut style) = query_cursor.get_single_mut() {
            style.left = Val::Px(screen_pos.x);
            style.top = Val::Px(screen_pos.y);
        }

        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
            cursor.position = screen_pos;
            cursor.world_position = world_pos;
        }
    }
}

pub(super) fn update_tile_grid_position(
    cursor: Res<CursorPosition>,
    mut query: Query<&mut Transform, With<TileGrid>>,
) {
    let mut transform = query.single_mut();
    
    let tile_coords = (cursor.world_position / TILE_SIZE).round();
    transform.translation.x = tile_coords.x * TILE_SIZE;
    transform.translation.y = tile_coords.y * TILE_SIZE;
}

pub(super) fn update_tile_grid_opacity(
    velocity: Res<PlayerVelocity>,
    mut query_tile_grid: Query<&mut Sprite, With<TileGrid>>,
) {
    use interpolation::Lerp;

    let mut sprite = query_tile_grid.single_mut();

    let opacity = if velocity.x.abs() > 0. {
        MIN_TILE_GRID_OPACITY.lerp(&MAX_TILE_GRID_OPACITY, &(1. - velocity.x.abs() / MAX_RUN_SPEED))
    } else if velocity.y.abs() > 0. {
        0f32.lerp(&MAX_TILE_GRID_OPACITY, &(1. - velocity.y.abs() / MAX_FALL_SPEED))
    } else {
        MAX_TILE_GRID_OPACITY
    };

    sprite.color = *sprite.color.set_a(opacity.clamp(0., MAX_TILE_GRID_OPACITY));
}

pub(super) fn update_cursor_info(
    query_hoverable: Query<(&Hoverable, &Interaction), Changed<Interaction>>,
    mut query_info: Query<(&mut Text, &mut Visibility), With<CursorInfoMarker>>,
) {
    let (mut text, mut visibility) = query_info.single_mut();

    query_hoverable.for_each(|(hoverable, interaction)| {
        helpers::set_visibility(
            &mut visibility, 
            !matches!(hoverable, Hoverable::None) && !matches!(interaction, Interaction::None)
        );

        if let (Hoverable::SimpleText(info), Interaction::Hovered) = (hoverable, interaction) {
            text.sections[0].value = info.clone();
        }
    });
}