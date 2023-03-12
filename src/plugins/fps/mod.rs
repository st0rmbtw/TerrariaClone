use bevy::{prelude::{Plugin, IntoSystemConfigs, OnUpdate}, diagnostic::FrameTimeDiagnosticsPlugin};

pub use components::*;
pub use resources::*;
pub use systems::*;

use crate::state::GameState;

mod components;
mod resources;
mod systems;

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<FpsTextVisibility>();
        app.add_plugin(FrameTimeDiagnosticsPlugin);

        app.add_systems(
            (
                toggle_fps_text_visibility,
                set_fps_text_visibility,
                update_fps_text,
            )
            .chain()
            .in_set(OnUpdate(GameState::InGame))
        );
    }
}