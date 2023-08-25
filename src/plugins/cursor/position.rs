use std::marker::PhantomData;

use bevy::{prelude::{Component, Plugin, App, Vec2, ResMut, Query, With, Camera, GlobalTransform, PreUpdate, IntoSystemConfigs, in_state, not, Resource}, window::{Window, PrimaryWindow}};

use crate::common::state::GameState;

#[derive(Resource)]
pub(crate) struct CursorPosition<CameraMarker: Component> {
    pub(crate) screen: Vec2,
    pub(crate) world: Vec2,
    marker: PhantomData<CameraMarker>
}

impl<M: Component> Default for CursorPosition<M> {
    fn default() -> Self {
        Self {
            screen: Default::default(),
            world: Default::default(),
            marker: PhantomData
        }
    }
}

pub(crate) struct CursorPositionPlugin<CameraMarker: Component> {
    marker: PhantomData<CameraMarker>,
}

impl<M: Component> Default for CursorPositionPlugin<M> {
    fn default() -> Self {
        Self { marker: PhantomData }
    }
}

impl<M: Component> Plugin for CursorPositionPlugin<M> {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPosition<M>>();

        app.add_systems(
            PreUpdate,
            update_cursor_position::<M>
                .run_if(not(in_state(GameState::AssetLoading)))
        );
    }

    fn is_unique(&self) -> bool {
        false
    }
}

fn update_cursor_position<CameraMarker: Component>(
    mut cursor: ResMut<CursorPosition<CameraMarker>>,
    query_window: Query<&Window, With<PrimaryWindow>>,
    query_main_camera: Query<(&Camera, &GlobalTransform), With<CameraMarker>>,
) {
    if let Ok((camera, camera_transform)) = query_main_camera.get_single() {
        let window = query_window.single();

        let Some(screen_pos) = window.cursor_position() else { return; };
        cursor.screen = screen_pos;

        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
            cursor.world = world_pos;
        }
    }
}