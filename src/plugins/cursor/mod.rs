pub(crate) mod components;
pub(crate) mod position;
mod systems;

use bevy::{prelude::{Plugin, App, OnExit, OnEnter, IntoSystemConfigs, not, resource_equals, in_state, Update, Condition}, ui::BackgroundColor};
use crate::{common::{state::GameState, systems::bind_visibility_to, conditions::is_visible}, animation::{AnimationSystemSet, component_animator_system}};
use self::position::CursorPositionPlugin;

use super::{ui::resources::{Ui, Cursor}, config::ShowTileGrid, camera::components::{MainCamera, BackgroundCamera}, InGameSystemSet};

const CURSOR_SIZE: f32 = 22.;
const MAX_TILE_GRID_OPACITY: f32 = 0.8;
const MIN_TILE_GRID_OPACITY: f32 = 0.2;

#[cfg(feature = "debug")]
use crate::plugins::debug::DebugConfiguration;
#[cfg(feature = "debug")]
use bevy::prelude::Res;

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CursorPositionPlugin::<MainCamera>::default()
                .run_if(in_state(GameState::InGame).and_then(is_visible::<Ui>)),
            CursorPositionPlugin::<BackgroundCamera>::default()
                .run_if(in_state(GameState::Menu))
        ));

        app.add_systems(OnExit(GameState::AssetLoading), systems::setup);
        app.add_systems(OnEnter(GameState::InGame), systems::spawn_tile_grid);

        app.add_systems(
            Update,
            (
                (
                    systems::update_cursor_info,
                    systems::update_cursor_item,
                )
                .in_set(InGameSystemSet::Update),
            ).run_if(not(in_state(GameState::AssetLoading)))
        );

        app.add_systems(
            Update,
            component_animator_system::<BackgroundColor>
                .in_set(AnimationSystemSet::AnimationUpdate)
                .run_if(not(in_state(GameState::AssetLoading)))
                .run_if(is_visible::<Ui>)
        );

        let update_tile_grid_opacity = systems::update_tile_grid_opacity;

        #[cfg(feature = "debug")]
        let update_tile_grid_opacity = update_tile_grid_opacity.run_if(|config: Res<DebugConfiguration>| !config.free_camera);

        app.add_systems(
            Update,
            (
                (
                    update_tile_grid_opacity,
                    systems::update_tile_grid_position,
                )
                .run_if(is_visible::<Ui>.and_then(resource_equals(ShowTileGrid(true)))),

                systems::update_tile_grid_visibility,
                bind_visibility_to::<Cursor, components::CursorBackground>
            )
            .in_set(InGameSystemSet::Update)
        );

    }
}
