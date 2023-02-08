use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::quick::{WorldInspectorPlugin};
use bevy_prototype_debug_lines::DebugLinesPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin);

        app.add_plugin(DebugLinesPlugin::default());
    }
}
