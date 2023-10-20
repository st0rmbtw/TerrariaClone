use std::{marker::PhantomData, sync::Mutex};

use bevy::{prelude::{Component, Plugin, App, Vec2, ResMut, Query, With, Camera, GlobalTransform, PreUpdate, IntoSystemConfigs, in_state, not, Resource, Condition, IntoSystem, Res, Commands, Entity, ComputedVisibility, Transform, Update, SystemSet}, window::{Window, PrimaryWindow}, ecs::schedule::BoxedCondition, ui::{Style, Val}};

use crate::{common::{state::GameState, components::Bounds, rect::FRect}, plugins::entity::components::EntityRect};

use super::components::{CursorContainer, Hoverable, MouseOver};

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

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) enum CursorSystems {
    UpdateCursorPosition
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

        let mut systems_pre_update = (
            update_cursor_position::<M>.in_set(CursorSystems::UpdateCursorPosition),
            update_cursor_sprite_position::<M>,
        )
        .chain()
        .run_if(not(in_state(GameState::AssetLoading)));

        let mut systems_update = (
            update_world_mouse_over_bounds::<M>,
            update_world_mouse_over_rect::<M>
        )
        .run_if(not(in_state(GameState::AssetLoading)));

        if let Some(condition) = self.condition.try_lock().unwrap().take() {
            systems_pre_update.run_if_inner(condition);
        }

        if let Some(condition) = self.condition.try_lock().unwrap().take() {
            systems_update.run_if_inner(condition);
        }

        app.add_systems(PreUpdate, systems_pre_update);
        app.add_systems(Update, systems_update);
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

pub(crate) fn update_world_mouse_over_bounds<M: Component>(
    mut commands: Commands,
    cursor_pos: Res<CursorPosition<M>>,
    query_hoverable: Query<(Entity, &Transform, &Bounds, &ComputedVisibility), With<Hoverable>>,
    query_camera: Query<&Camera, With<M>>
) {
    let Ok(camera) = query_camera.get_single() else { return; };
    if !camera.is_active { return; }

    query_hoverable.for_each(|(entity, transform, bounds, visibility)| {
        let rect = FRect::new_center(transform.translation.x, transform.translation.y, bounds.width, bounds.height);

        if rect.contains(cursor_pos.world) && visibility.is_visible() {
            commands.entity(entity).insert(MouseOver);
        }
    });
}

pub(crate) fn update_world_mouse_over_rect<M: Component>(
    mut commands: Commands,
    cursor_pos: Res<CursorPosition<M>>,
    query_hoverable: Query<(Entity, &EntityRect, &ComputedVisibility), With<Hoverable>>,
    query_camera: Query<&Camera, With<M>>
) {
    let Ok(camera) = query_camera.get_single() else { return; };
    if !camera.is_active { return; }

    query_hoverable.for_each(|(entity, entity_rect, visibility)| {
        if entity_rect.contains(cursor_pos.world) && visibility.is_visible() {
            commands.entity(entity).insert(MouseOver);
        }
    });
}