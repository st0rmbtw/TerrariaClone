use std::time::Duration;

use bevy::{
    prelude::{Plugin, resource_exists_and_equals, Condition, Res, KeyCode, Query, With, Update, IntoSystemConfigs},
    text::Text,
    time::common_conditions::on_timer,
    input::common_conditions::input_just_pressed, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, Diagnostic},
};

use crate::common::helpers::toggle_visibility;
use super::ui::{UiVisibility, FpsText};

pub(crate) struct FpsPlugin;
impl Plugin for FpsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin);

        app.add_systems(
            Update,
            (
                toggle_visibility::<FpsText>.run_if(input_just_pressed(KeyCode::F10)),
                update_fps_text.run_if(
                    resource_exists_and_equals(UiVisibility::VISIBLE).and_then(on_timer(Duration::from_secs(1)))
                ),
            )
        );
    }
}

fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query_fps_text: Query<&mut Text, With<FpsText>>,
) {
    let Ok(mut text) = query_fps_text.get_single_mut() else { return; };

    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS).and_then(Diagnostic::average) {
        text.sections[0].value = format!("{:.0}", fps);
    }
}