use bevy::{prelude::{Plugin, App, OnEnter, Update, IntoSystemConfigs}, sprite::TextureAtlasSprite};
use bevy_ecs_tilemap::{tiles::TilePos, helpers::square_grid::neighbors::Neighbors};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::common::{components::EntityRect, state::GameState};

use self::resources::HoverBlockData;

use super::{inventory::{UseItemAnimationIndex, UseItemAnimationData}, player::FaceDirection, InGameSystemSet};

mod components;
mod gui;
mod resources;
mod systems;

pub(crate) use resources::DebugConfiguration;

pub(crate) struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldInspectorPlugin::new());

        app.insert_resource(HoverBlockData {
            pos: TilePos::default(),
            block_type: None,
            neighbors: Neighbors {
                east: None,
                north_east: None,
                north: None,
                north_west: None,
                west: None,
                south_west: None,
                south: None,
                south_east: None,
            },
        });

        app.insert_resource(resources::DebugConfiguration::default());
        app.insert_resource(resources::MouseParticleSettings::default());
        app.insert_resource(resources::MouseLightSettings::default());

        app.register_type::<TextureAtlasSprite>();
        app.register_type::<UseItemAnimationIndex>();
        app.register_type::<UseItemAnimationData>();
        app.register_type::<FaceDirection>();
        app.register_type::<EntityRect>();

        app.add_systems(
            OnEnter(GameState::InGame),
            (
                systems::spawn_free_camera_legend,
                systems::spawn_mouse_light
            )
        );
        
        app.add_systems(
            Update,
            (
                gui::debug_gui,
                gui::block_gui,
                gui::particle_gui,
                gui::mouse_light_gui,
                systems::set_free_camera_legend_visibility,
                systems::block_hover,
                systems::spawn_particles,
                systems::update_mouse_light
            )
            .in_set(InGameSystemSet::Update)
        );
    }
}