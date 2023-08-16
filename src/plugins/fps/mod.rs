use std::time::Duration;

use autodefault::autodefault;
use bevy::{
    prelude::{Plugin, resource_exists_and_equals, Condition, Commands, Entity, Color, TextBundle, Res, KeyCode, Query, Visibility, With, Name, Update, IntoSystemConfigs, Component},
    text::{TextStyle, Text, TextSection, TextAlignment},
    ui::{Style, UiRect, Val},
    time::common_conditions::on_timer,
    input::common_conditions::input_just_pressed, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, Diagnostic},
};

use crate::common::helpers::toggle_visibility;
use super::{assets::FontAssets, ui::UiVisibility};

pub(crate) struct FpsPlugin;
impl Plugin for FpsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());

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

#[derive(Component)]
pub(crate) struct FpsText;

#[autodefault]
pub(crate) fn spawn_fps_text(commands: &mut Commands, font_assets: &FontAssets) -> Entity {
    let text_style = TextStyle {
        font: font_assets.andy_regular.clone_weak(),
        font_size: 20.,
        color: Color::WHITE,
    };

    commands.spawn((
        FpsText,
        Name::new("FPS Text"),
        TextBundle {
            style: Style {
                margin: UiRect {
                    left: Val::Px(5.),
                    bottom: Val::Px(5.),
                },
            },
            text: Text {
                sections: vec![
                    TextSection::from_style(text_style)
                ],
                alignment: TextAlignment::Center,
            },
            visibility: Visibility::Hidden,
        }
    ))
    .id()
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