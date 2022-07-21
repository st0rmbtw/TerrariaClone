use bevy::prelude::{Plugin, App};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier2d::prelude::RapierDebugRenderPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(WorldInspectorPlugin::default())
            .add_plugin(RapierDebugRenderPlugin::default());
    }
}

// fn listen_input(
//     mut commands: Commands,
//     input: Res<Input<KeyCode>>
// ) {
//     if input.just_pressed(KeyCode::LControl) && input.just_released(KeyCode::D) {
//         // commands.pl
//     }
// }