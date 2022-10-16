use bevy::{prelude::Plugin, diagnostic::FrameTimeDiagnosticsPlugin};

pub use components::*;
use iyes_loopless::prelude::ConditionSet;
pub use resources::*;
pub use systems::*;

use crate::state::GameState;

mod components;
mod resources;
mod systems;

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<FpsTextVisibility>()
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(toggle_fps_text_visibility)
                    .with_system(set_fps_text_visibility)
                    .with_system(update_fps_text)
                    .into(),
            );
    }
}