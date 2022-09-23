use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::{WorldInspectorPlugin, RegisterInspectable};
use bevy_prototype_debug_lines::DebugLinesPlugin;

use super::FaceDirection;
use crate::state::MovementState;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::default())
            .add_plugin(DebugLinesPlugin::default())
            .register_inspectable::<FaceDirection>()
            .register_inspectable::<MovementState>();
    }
}
