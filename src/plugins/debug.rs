use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};

use super::FaceDirection;
use crate::state::MovementState;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::default())
            .register_inspectable::<FaceDirection>()
            .register_inspectable::<MovementState>();
    }
}
