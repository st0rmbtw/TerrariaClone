use bevy::{prelude::{Plugin, App, Commands, Res, Camera, With, Query, Vec2, GlobalTransform, NodeBundle, Color, default, Component, Transform, ResMut, ImageBundle, BuildChildren, Without, TextBundle, Deref, DerefMut}, window::Windows, render::camera::RenderTarget, ui::{Style, Size, Val, UiRect, PositionType, JustifyContent, AlignContent, AlignSelf}, text::{Text, TextStyle}};

use crate::TRANSPARENT;

use super::{MainCamera, CursorAssets, FontAssets};

// region: Plugin

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(HoveredInfo::default())
            .insert_resource(Cursor::default())
            .add_startup_system(setup)
            .add_system(update_cursor_position)
            .add_system(update_hovered_info)
            .add_system(update_hovered_info_position);
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
        // region: Cursor

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
        })
        .insert(CursorBackground)
        .with_children(|c| {
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
            })
            .insert(CursorForeground);
        });

        // endregion
    }).insert(CursorContainer);

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
    mut wnds: ResMut<Windows>,
    mut cursor: ResMut<Cursor>,
    q_camera: Query<(&Camera, &GlobalTransform), (With<MainCamera>, Without<CursorContainer>)>,
    mut query: Query<(&mut Style, &mut Transform, &mut GlobalTransform), With<CursorContainer>>
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
        style.position = UiRect {
            left: Val::Px(screen_pos.x - 2.),
            bottom: Val::Px(screen_pos.y - 20.),
            ..default()
        };

        cursor.position = screen_pos;

        transform.translation.z = 1.;
        global_transform.affine().translation.z = 1.;
    }
}


fn update_hovered_info_position(
    cursor: Res<Cursor>,
    mut query: Query<&mut Style, With<HoveredInfoMarker>>
) {
    let mut style = query.single_mut();

    style.position = UiRect {
        left: Val::Px(cursor.position.x + 20.),
        bottom: Val::Px(cursor.position.y - 45.),
        ..default()
    }
}

// TODO
fn cursor_animation(
    mut cursor_foreground: Query<&mut Style, With<CursorForeground>>,
    mut cursor_background: Query<&mut Style, With<CursorBackground>>,
) {
    let mut cursor_foreground = cursor_foreground.single_mut();
    let mut cursor_background = cursor_background.single_mut();   
}

fn update_hovered_info(
    hovered_info: Res<HoveredInfo>,
    mut query: Query<&mut Text, With<HoveredInfoMarker>>
) {
    for mut text in &mut query {
        text.sections[0].value = hovered_info.0.clone();
    }
}