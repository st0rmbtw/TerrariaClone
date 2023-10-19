use std::time::Duration;

use bevy::{
    prelude::{Plugin, Condition, Res, KeyCode, Query, With, Update, IntoSystemConfigs, App},
    text::Text,
    time::common_conditions::on_timer,
    input::common_conditions::input_just_pressed, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, Diagnostic},
};

use crate::common::{systems::toggle_visibility, conditions::is_visible};
use super::ui::{FpsText, resources::Ui};

pub(crate) struct FpsPlugin;
impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin);

        app.add_systems(
            Update,
            (
                toggle_visibility::<FpsText>.run_if(input_just_pressed(KeyCode::F10)),
                update_fps_text.run_if(is_visible::<Ui>.and_then(on_timer(Duration::from_secs(1)))),
            )
        );
    }
}

fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query_fps_text: Query<&mut Text, With<FpsText>>,
) {
    let Ok(mut text) = query_fps_text.get_single_mut() else { return; };

    let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS).and_then(Diagnostic::average) else { return; };

    text.sections[0].value = format!("{:.0}", fps);
}