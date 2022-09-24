use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::{WorldInspectorPlugin, RegisterInspectable};
use bevy_prototype_debug_lines::DebugLinesPlugin;

use crate::state::MovementState;

use super::player::FaceDirection;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::default())
            .add_plugin(DebugLinesPlugin::default())
            .register_inspectable::<FaceDirection>()
            .register_inspectable::<MovementState>();
    }
}
