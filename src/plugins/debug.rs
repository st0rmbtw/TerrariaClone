use bevy::prelude::{Plugin, App};
use bevy_inspector_egui::{WorldInspectorPlugin, RegisterInspectable};
use bevy_rapier2d::prelude::RapierDebugRenderPlugin;

use super::{SpeedCoefficient, FaceDirection};
use crate::state::MovementState;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(WorldInspectorPlugin::default())
            .add_plugin(RapierDebugRenderPlugin::default())
            .register_inspectable::<FaceDirection>()
            .register_inspectable::<MovementState>()
            .register_inspectable::<SpeedCoefficient>();
    }
}