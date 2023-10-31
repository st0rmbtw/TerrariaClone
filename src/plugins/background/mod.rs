use crate::common::{state::GameState, systems::despawn_with};
use bevy::{
    prelude::{
        in_state, not, App, Component, IntoSystemConfigs, OnEnter, OnExit, Plugin, PostUpdate,
        Update,
    },
    render::view::RenderLayers,
    transform::TransformSystem,
};

use self::sun_and_moon::SunAndMoonPlugin;

use super::{camera::CameraSet, InGameSystemSet};

pub(crate) const BACKGROUND_RENDER_LAYER: RenderLayers = RenderLayers::layer(24);
pub(crate) const INGAME_BACKGROUND_RENDER_LAYER: RenderLayers = RenderLayers::layer(23);

pub(crate) mod sun_and_moon;
mod systems;

pub(crate) struct BackgroundPlugin;
impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SunAndMoonPlugin);

        app.add_systems(
            OnEnter(GameState::Menu),
            (
                systems::spawn_background_camera,
                systems::setup_main_menu_background,
            ),
        );

        app.add_systems(
            OnExit(GameState::WorldLoading),
            (
                despawn_with::<MenuParallaxContainer>,
                systems::spawn_ingame_background_camera,
                systems::spawn_sky_background,
                systems::spawn_ingame_background,
                systems::spawn_forest_background,
            ),
        );

        app.add_systems(
            Update,
            systems::update_sprites_color.run_if(not(in_state(GameState::AssetLoading))),
        );

        app.add_systems(
            PostUpdate,
            systems::follow_camera_system
                .in_set(InGameSystemSet::PostUpdate)
                .after(CameraSet::MoveCamera)
                .before(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(Component)]
pub(crate) struct MenuParallaxContainer;

#[derive(Component)]
pub(crate) struct BiomeParallaxContainer;

#[derive(Component)]
pub(crate) struct InGameParallaxContainer;
