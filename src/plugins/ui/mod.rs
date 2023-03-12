mod components;
mod resources;
mod systems;
mod events;

pub use components::*;
pub use resources::*;
pub use systems::*;
pub use events::*;

use bevy::prelude::{Plugin, App, IntoSystemAppConfig, OnEnter, IntoSystemConfigs, OnUpdate};
use crate::state::GameState;

pub const SPAWN_UI_CONTAINER_LABEL: &str = "spawn_ui_container";

pub struct PlayerUiPlugin;

impl Plugin for PlayerUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToggleExtraUiEvent>();
        app.init_resource::<ExtraUiVisibility>();
        app.init_resource::<UiVisibility>();
        app.add_system(spawn_ui_container.in_schedule(OnEnter(GameState::InGame)));
        app.add_systems(
            (
                toggle_extra_ui,
                toggle_ui,
                set_main_container_visibility,
            )
            .chain()
            .in_set(OnUpdate(GameState::InGame))
        );
    }
}