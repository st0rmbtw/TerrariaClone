mod systems;

use bevy::{prelude::{Plugin, App, OnEnter, Deref, Update, IntoSystemConfigs, KeyCode, Handle, Image, apply_deferred, resource_equals, Resource, Component, resource_exists_and_equals}, render::{view::RenderLayers, render_resource::{ShaderRef, AsBindGroup}}, input::common_conditions::input_just_pressed, sprite::{Material2d, Material2dPlugin}, reflect::{TypeUuid, TypePath}};

use crate::common::state::GameState;

use super::{InGameSystemSet, ui::systems::update_world_mouse_over_bounds, cursor::position::CursorPositionPlugin};

pub(crate) struct WorldMapViewPlugin;
impl Plugin for WorldMapViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            CursorPositionPlugin::<WorldMapViewCamera>::default()
                .run_if(resource_exists_and_equals(MapViewStatus::Opened))
        );

        app.add_plugins(Material2dPlugin::<WorldMapViewMaterial>::default());

        app.init_resource::<MapViewStatus>();

        app.add_systems(
            OnEnter(GameState::InGame),
            (systems::init_world_map_texture, apply_deferred, systems::setup).chain()
        );

        app.add_systems(
            Update,
            systems::update_world_map_texture.in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            Update,
            (
                systems::update_map_view,
                systems::clamp_map_view_position,
                update_world_mouse_over_bounds::<WorldMapViewCamera>
            )
            .chain()
            .in_set(InGameSystemSet::Update)
            .run_if(resource_equals(MapViewStatus::Opened))
        );

        app.add_systems(
            Update,
            systems::toggle_world_map_view
                .in_set(InGameSystemSet::Update)
                .run_if(input_just_pressed(KeyCode::M))
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

#[derive(Resource, Deref)]
struct WorldMapTexture(Handle<Image>);

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum MapViewStatus {
    Opened,
    #[default]
    Closed
}

impl MapViewStatus {
    fn is_opened(&self) -> bool {
        *self == MapViewStatus::Opened
    }

    fn set_opened(&mut self, opened: bool) {
        match opened {
            true => *self = MapViewStatus::Opened,
            false => *self = MapViewStatus::Closed,
        }
    }
}

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "1b120582-0216-4a54-95d8-924071b88311"]
struct WorldMapViewMaterial {
    #[texture(0)]
    #[sampler(1)]
    tile_map: Handle<Image>
}

impl Material2d for WorldMapViewMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/map_view.wgsl".into()
    }
}