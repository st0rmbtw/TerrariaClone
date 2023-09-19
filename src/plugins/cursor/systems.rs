use std::time::Duration;

use autodefault::autodefault;
use bevy::{
    prelude::{
        Res, Commands, Vec3, Color, NodeBundle, default, TextBundle, Name, ImageBundle, Transform, Query, With, Visibility, 
        BuildChildren, Changed
    }, 
    ui::{
        Style, JustifyContent, AlignItems, PositionType, FocusPolicy, Val, AlignSelf, ZIndex, FlexDirection, UiRect, Interaction
    }, 
    text::{Text, TextStyle}, 
    sprite::{SpriteBundle, Sprite}
};
use interpolation::{EaseFunction, Lerp};

use crate::{
    plugins::{
        assets::{FontAssets, CursorAssets, UiAssets}, 
        camera::components::MainCamera, 
        world::constants::TILE_SIZE, config::CursorColor, DespawnOnGameExit, player::Player
    }, 
    animation::{Tween, lens::TransformScaleLens, Animator, RepeatStrategy, RepeatCount}, 
    common::{lens::BackgroundColorLens, components::Velocity}, language::LanguageContent,
};

use crate::plugins::player::{MAX_RUN_SPEED, MAX_FALL_SPEED};

use super::{MAX_TILE_GRID_OPACITY, MIN_TILE_GRID_OPACITY, CURSOR_SIZE, components::{Hoverable, CursorBackground, CursorForeground, CursorInfoMarker, CursorContainer, TileGrid}, position::CursorPosition};

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
        DespawnOnGameExit,
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
    cursor_pos: Res<CursorPosition<MainCamera>>,
    mut query_cursor: Query<&mut Style, With<CursorContainer>>,
) {
    if let Ok(mut style) = query_cursor.get_single_mut() {
        style.left = Val::Px(cursor_pos.screen.x);
        style.top = Val::Px(cursor_pos.screen.y);
    }
}

pub(super) fn update_tile_grid_position(
    cursor_pos: Res<CursorPosition<MainCamera>>,
    mut query: Query<&mut Transform, With<TileGrid>>,
) {
    let mut transform = query.single_mut();
    
    let tile_coords = (cursor_pos.world / TILE_SIZE).round();
    transform.translation.x = tile_coords.x * TILE_SIZE;
    transform.translation.y = tile_coords.y * TILE_SIZE;
}

pub(super) fn update_tile_grid_opacity(
    query_player: Query<&Velocity, With<Player>>,
    mut query_tile_grid: Query<&mut Sprite, With<TileGrid>>,
) {
    let Ok(velocity) = query_player.get_single() else { return; };
    
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
    language_content: Res<LanguageContent>,
    query_hoverable: Query<(&Hoverable, &Interaction), Changed<Interaction>>,
    mut query_info: Query<(&mut Text, &mut Visibility), With<CursorInfoMarker>>,
) {
    let (mut text, mut visibility) = query_info.single_mut();

    query_hoverable.for_each(|(hoverable, interaction)| {
        if let (Hoverable::SimpleText(info), Interaction::Hovered) = (hoverable, interaction) {
            text.sections[0].value = info.format(&language_content);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    });
}