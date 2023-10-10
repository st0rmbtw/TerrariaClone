use std::time::Duration;

use bevy::{
    prelude::{
        Res, Commands, Vec3, Color, NodeBundle, default, TextBundle, Name, ImageBundle, Transform, Query, With, Visibility, 
        BuildChildren, Without
    }, 
    ui::{
        Style, JustifyContent, AlignItems, PositionType, FocusPolicy, Val, AlignSelf, ZIndex, FlexDirection, UiImage
    }, 
    text::{Text, TextStyle}, 
    sprite::{SpriteBundle, Sprite}, ecs::query::Has
};
use interpolation::{EaseFunction, Lerp};

use crate::{
    plugins::{
        assets::{FontAssets, CursorAssets, UiAssets, ItemAssets}, 
        camera::components::MainCamera, 
        world::constants::TILE_SIZE, config::{CursorColor, ShowTileGrid}, DespawnOnGameExit, player::Player, ui::{UiVisibility, components::MouseOver}, inventory::{Inventory, Slot}, entity::components::Velocity
    }, 
    animation::{Tween, lens::TransformScaleLens, Animator, RepeatStrategy, RepeatCount}, 
    common::{lens::BackgroundColorLens, helpers, BoolValue}, language::LanguageContent,
};

use crate::plugins::player::{MAX_WALK_SPEED, MAX_FALL_SPEED};

use super::{MAX_TILE_GRID_OPACITY, MIN_TILE_GRID_OPACITY, CURSOR_SIZE, components::{Hoverable, CursorBackground, CursorForeground, CursorInfoMarker, CursorContainer, TileGrid, CursorItemContainer, CursorItemStack, CursorItemImage}, position::CursorPosition};

pub(super) fn setup(
    mut commands: Commands, 
    cursor_assets: Res<CursorAssets>, 
    fonts: Res<FontAssets>,
    cursor_color: Res<CursorColor>
) {
    let animate_scale = Tween::new(
        EaseFunction::QuadraticInOut,
        RepeatStrategy::MirroredRepeat,
        Duration::from_millis(450),
        TransformScaleLens {
            start: Vec3::new(1., 1., 1.),
            end: Vec3::new(1.15, 1.15, 1.),
        },
    )
    .with_repeat_count(RepeatCount::Infinite);

    let animate_color = Tween::new(
        EaseFunction::QuadraticInOut,
        RepeatStrategy::MirroredRepeat,
        Duration::from_millis(450),
        BackgroundColorLens {
            start: cursor_color.foreground_color * 0.7,
            end: cursor_color.foreground_color,
        },
    ).with_repeat_count(RepeatCount::Infinite);

    commands
        .spawn((
            CursorContainer,
            Name::new("Cursor Container"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                z_index: ZIndex::Global(i32::MAX),
                ..default()
            }
        ))
        .with_children(|c| {
            // region: Cursor

            c.spawn((
                CursorBackground,
                Animator::new(animate_scale),
                ImageBundle {
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
                }
            ))
            .with_children(|c| {
                c.spawn((
                    CursorForeground,
                    Animator::new(animate_color),
                    ImageBundle {
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
                    }
                ));
            });

            // endregion

            c.spawn((
                CursorInfoMarker,
                TextBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(CURSOR_SIZE),
                        top: Val::Px(CURSOR_SIZE),
                        ..default()
                    },
                    text: Text::from_section(
                        String::new(),
                        TextStyle {
                            font: fonts.andy_bold.clone_weak(),
                            font_size: 22.,
                            color: Color::WHITE,
                        },
                    ).with_no_wrap(),
                    visibility: Visibility::Hidden,
                    ..default()
                }
            ));

            // Cursor item
            c.spawn((
                CursorItemContainer,
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(16.),
                        top: Val::Px(16.),
                        ..default()
                    },
                    ..default()
                },
            )).with_children(|parent| {
                // Item image
                parent.spawn((
                    CursorItemImage,
                    ImageBundle::default()
                ));

                // Item stack
                parent.spawn((
                    CursorItemStack,
                    TextBundle {
                        style: Style {
                            top: Val::Px(10.),
                            right: Val::Px(0.),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        text: Text::from_section(
                            String::new(),
                            TextStyle {
                                font: fonts.andy_bold.clone_weak(),
                                font_size: 20.,
                                color: Color::WHITE,
                            },
                        ),
                        ..default()
                    }
                ));
            });
        });
}

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
                color: Color::WHITE.with_a(MAX_TILE_GRID_OPACITY),
                ..default()
            },
            texture: ui_assets.radial.clone_weak(),
            transform: Transform::from_xyz(0., 0., 5.),
            visibility: Visibility::Hidden,
            ..default()
        }
    ));
}

pub(super) fn update_tile_grid_visibility(
    ui_visibility: Res<UiVisibility>,
    show_tile_grid: Res<ShowTileGrid>,
    mut query: Query<&mut Visibility, With<TileGrid>>,
) {
    let Ok(visibility) = query.get_single_mut() else { return; };
    helpers::set_visibility(visibility, ui_visibility.value() && show_tile_grid.value());
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
        MIN_TILE_GRID_OPACITY.lerp(&MAX_TILE_GRID_OPACITY, &(1. - velocity.x.abs() / MAX_WALK_SPEED))
    } else if velocity.y.abs() > 0. {
        0f32.lerp(&MAX_TILE_GRID_OPACITY, &(1. - velocity.y.abs() / MAX_FALL_SPEED))
    } else {
        MAX_TILE_GRID_OPACITY
    };

    sprite.color.set_a(opacity.clamp(0., MAX_TILE_GRID_OPACITY));
}

pub(super) fn update_cursor_info(
    inventory: Res<Inventory>,
    language_content: Res<LanguageContent>,
    mut query_hoverable: Query<(&mut Hoverable, Has<MouseOver>)>,
    mut query_info: Query<(&mut Text, &mut Visibility), With<CursorInfoMarker>>,
) {
    let (mut text, mut visibility) = query_info.single_mut();

    *visibility = Visibility::Hidden;

    if inventory.item_exists(Slot::MouseItem) { return; }
    
    for (mut hoverable, mouse_over) in &mut query_hoverable {
        if let (Hoverable::SimpleText(info), true) = (hoverable.as_mut(), mouse_over) {
            text.sections[0].value = info.text(&language_content);
            *visibility = Visibility::Inherited;
            return;
        }
    }
}

pub(super) fn update_cursor_item(
    inventory: Res<Inventory>,
    item_assets: Res<ItemAssets>,
    mut query_cursor_item_container: Query<&mut Visibility, With<CursorItemContainer>>,
    mut query_cursor_item_image: Query<&mut UiImage, With<CursorItemImage>>,
    mut query_cursor_item_stack: Query<(&mut Text, &mut Visibility), (With<CursorItemStack>, Without<CursorItemContainer>)>,
) {
    let mut visibility = query_cursor_item_container.single_mut();
    let mut image = query_cursor_item_image.single_mut();
    let (mut text, mut text_visibility) = query_cursor_item_stack.single_mut();
    
    *visibility = Visibility::Hidden;

    let Some(mouse_item) = inventory.get_item(Slot::MouseItem) else { return; };

    *visibility = Visibility::Inherited;
    image.texture = item_assets.get_by_item(mouse_item.item);

    if mouse_item.stack > 1 {
        text.sections[0].value = mouse_item.stack.to_string();
        *text_visibility = Visibility::Inherited;
    } else {
        *text_visibility = Visibility::Hidden;
    }
}