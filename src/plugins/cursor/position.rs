use std::{marker::PhantomData, sync::Mutex};

use bevy::{prelude::{Component, Plugin, App, Vec2, ResMut, Query, With, Camera, GlobalTransform, PreUpdate, IntoSystemConfigs, in_state, not, Resource, Condition, IntoSystem, Res}, window::{Window, PrimaryWindow}, ecs::schedule::BoxedCondition, ui::{Style, Val}};

use crate::common::state::GameState;

use super::components::CursorContainer;

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

        let mut systems = (
            update_cursor_position::<M>,
            update_cursor_sprite_position::<M>
        )
        .chain()
        .run_if(not(in_state(GameState::AssetLoading)));

        if let Some(condition) = self.condition.try_lock().unwrap().take() {
            systems.run_if_inner(condition);
        }

        app.add_systems(PreUpdate, systems);
    }
}

fn update_cursor_position<CameraMarker: Component>(
    mut cursor_pos: ResMut<CursorPosition<CameraMarker>>,
    query_window: Query<&Window, With<PrimaryWindow>>,
    query_camera: Query<(&Camera, &GlobalTransform), With<CameraMarker>>,
) {
    let Ok((camera, camera_transform)) = query_camera.get_single() else { return; };
    let window = query_window.single();

    let Some(screen_pos) = window.cursor_position() else { return; };
    cursor_pos.screen = screen_pos;

    let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) else { return; };
    cursor_pos.world = world_pos;
}

fn update_cursor_sprite_position<CameraMarker: Component>(
    cursor_pos: Res<CursorPosition<CameraMarker>>,
    mut query_cursor: Query<&mut Style, With<CursorContainer>>,
) {
    let Ok(mut style) = query_cursor.get_single_mut() else { return; };
    style.left = Val::Px(cursor_pos.screen.x);
    style.top = Val::Px(cursor_pos.screen.y);
}