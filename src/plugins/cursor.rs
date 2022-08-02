use bevy::{prelude::{Plugin, App, Commands, Res, Camera, With, Query, Vec2, GlobalTransform, NodeBundle, Color, default, Component, Transform, ResMut, ImageBundle, BuildChildren, Without}, window::Windows, render::camera::RenderTarget, ui::{Style, Size, Val, UiRect, PositionType, JustifyContent, AlignContent, AlignSelf}};

use crate::TRANSPARENT;

use super::{MainCamera, CursorAssets};

// region: Plugin

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_system(update_cursor_position);
    }
}

// endregion

// region: Components

#[derive(Component)]
struct Cursor;

// endregion

fn setup(
    mut commands: Commands,
    cursor_assets: Res<CursorAssets>
) {
    commands.spawn_bundle(NodeBundle {
        style: Style {
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        color: TRANSPARENT.into(),
        ..default()
    }).with_children(|c| {
        c.spawn_bundle(ImageBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_self: AlignSelf::Center,
                size: Size::new(Val::Px(22.), Val::Px(22.)),
                ..default()
            },
            image: cursor_assets.cursor_background.clone().into(),
            color: Color::rgb(0.7, 0.7, 0.7).into(),
            ..default()
        }).with_children(|c| {
            c.spawn_bundle(ImageBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_self: AlignSelf::Center,
                    size: Size::new(Val::Px(16.), Val::Px(16.)),
                    ..default()
                },
                image: cursor_assets.cursor.clone().into(),
                color: Color::PINK.into(),
                ..default()
            });
        });
    }).insert(Cursor);
}

fn update_cursor_position(
    mut wnds: ResMut<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), (With<MainCamera>, Without<Cursor>)>,
    mut query: Query<(&mut Style, &mut Transform, &mut GlobalTransform), With<Cursor>>
) {
    let (mut style, mut transform, mut global_transform) = query.single_mut();

    let (camera, camera_transform) = q_camera.single();
    
    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get_mut(id)
    } else {
        wnds.get_primary_mut()
    }.unwrap();

    wnd.set_cursor_visibility(false);

    if let Some(screen_pos) = wnd.cursor_position() {
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let mut world_pos = ndc_to_world.project_point3(ndc.extend(0.));
        world_pos.z = 999.9;

        style.position = UiRect {
            left: Val::Px(screen_pos.x),
            bottom: Val::Px(screen_pos.y - 15.),
            ..default()
        };

        transform.translation.z = 1.;
        global_transform.affine().translation.z = 1.;
    }
}