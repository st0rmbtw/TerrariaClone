mod systems;

use bevy::{prelude::{Plugin, App, Deref, Update, IntoSystemConfigs, KeyCode, Handle, Image, apply_deferred, resource_equals, Resource, Component, resource_exists_and_equals, OnExit}, render::view::RenderLayers, input::common_conditions::input_just_pressed};

use crate::common::state::GameState;

use super::{InGameSystemSet, cursor::position::CursorPositionPlugin, entity::EntitySet};

const MOVE_SPEED: f32 = 1000.;

pub(crate) struct WorldMapViewPlugin;
impl Plugin for WorldMapViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            CursorPositionPlugin::<WorldMapViewCamera>::default()
                .run_if(resource_exists_and_equals(MapViewStatus::Opened))
        );

        app.init_resource::<MapViewStatus>();

        app.add_systems(
            OnExit(GameState::WorldLoading),
            (systems::init_world_map_texture, apply_deferred, systems::setup).chain()
        );

        app.add_systems(
            Update,
            (
                systems::toggle_world_map_view.run_if(input_just_pressed(KeyCode::M)),
                systems::update_world_map_texture,
            )
            .in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            Update,
            (
                (
                    systems::drag_map_view,
                    systems::move_map_view
                ),
                systems::update_min_scale,
                systems::update_map_view,
                systems::clamp_map_view_position,
                (
                    systems::update_spawn_icon_position,
                    systems::update_player_icon_position.after(EntitySet::UpdateEntityRect)
                )
            )
            .chain()
            .in_set(InGameSystemSet::Update)
            .run_if(resource_equals(MapViewStatus::Opened))
        );

        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            systems::teleport_player
                .after(EntitySet::UpdateEntityRect)
                .in_set(InGameSystemSet::Update)
                .run_if(resource_equals(MapViewStatus::Opened))
        );
    }
}

const WORLD_MAP_VIEW_RENDER_LAYER: RenderLayers = RenderLayers::layer(15);

#[derive(Component)]
struct WorldMapViewCamera;

#[derive(Component)]
struct WorldMapView;

#[derive(Component)]
struct SpawnPointIcon;

#[derive(Component)]
struct PlayerIcon;

#[derive(Resource, Deref)]
struct WorldMapTexture(Handle<Image>);

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum MapViewStatus {
    Opened,
    #[default]
    Closed
}

impl MapViewStatus {
    #[inline]
    pub(crate) fn is_opened(&self) -> bool {
        *self == MapViewStatus::Opened
    }

    fn set_opened(&mut self, opened: bool) {
        match opened {
            true => *self = MapViewStatus::Opened,
            false => *self = MapViewStatus::Closed,
        }
    }
}