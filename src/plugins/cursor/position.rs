use std::{marker::PhantomData, sync::Mutex};

use bevy::{prelude::{Component, Plugin, App, Vec2, ResMut, Query, With, Camera, GlobalTransform, PreUpdate, IntoSystemConfigs, in_state, not, Resource, Condition, IntoSystem}, window::{Window, PrimaryWindow}, ecs::schedule::BoxedCondition};

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
    condition: Mutex<Option<BoxedCondition>>,
    marker: PhantomData<CameraMarker>,
}

impl<CM: Component> CursorPositionPlugin<CM> {
    pub(crate) fn run_if<M>(mut self, condition: impl Condition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl<M: Component> Default for CursorPositionPlugin<M> {
    fn default() -> Self {
        Self {
            condition: Mutex::default(),
            marker: PhantomData
        }
    }
}

impl<M: Component> Plugin for CursorPositionPlugin<M> {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPosition<M>>();

        let mut system = update_cursor_position::<M>
            .run_if(not(in_state(GameState::AssetLoading)));

        if let Some(condition) = self.condition.try_lock().unwrap().take() {
            system.run_if_inner(condition);
        }

        app.add_systems(PreUpdate, system);
    }
}

fn update_cursor_position<CameraMarker: Component>(
    mut cursor_pos: ResMut<CursorPosition<CameraMarker>>,
    query_window: Query<&Window, With<PrimaryWindow>>,
    query_camera: Query<(&Camera, &GlobalTransform), With<CameraMarker>>,
) {
    if let Ok((camera, camera_transform)) = query_camera.get_single() {
        let window = query_window.single();

        let Some(screen_pos) = window.cursor_position() else { return; };
        cursor_pos.screen = screen_pos;

        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
            cursor_pos.world = world_pos;
        }
    }
}