use bevy::prelude::{App, Plugin,IntoSystemConfig, OnUpdate, ResMut};
use bevy_inspector_egui::{bevy_egui::{EguiPlugin, egui, EguiContexts}, egui::Align2, quick::WorldInspectorPlugin};

use crate::{state::GameState, DebugConfiguration};
use bevy_prototype_debug_lines::DebugLinesPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin);
        app.add_plugin(DebugLinesPlugin::default());
        app.add_plugin(WorldInspectorPlugin::new());
        app.add_system(debug_gui.in_set(OnUpdate(GameState::InGame)));
    }
}

fn debug_gui(mut contexts: EguiContexts, mut debug_config: ResMut<DebugConfiguration>) {
    let egui_context = contexts.ctx_mut();

    egui::Window::new("Debug Menu")
        .anchor(Align2::RIGHT_TOP, (-10., 10.))
        .resizable(false)
        .show(egui_context, |ui| {
            ui.checkbox(&mut debug_config.free_camera, "Free Camera");
            ui.checkbox(&mut debug_config.show_hitboxes, "Show Hitboxes");
        });
}
