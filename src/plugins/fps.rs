use std::time::Duration;

use autodefault::autodefault;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use iyes_loopless::prelude::ConditionSet;

use crate::state::GameState;

use super::FontAssets;

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<FpsTextVisibility>()
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(toggle_fps_text_visibility)
                    .with_system(set_fps_text_visibility)
                    .with_system(update_fps_text)
                    .into(),
            );
    }
}

#[derive(Component)]
struct FpsText;

#[derive(Component, Deref, DerefMut)]
struct FpsTextTimer(Timer);

#[derive(Clone, Copy, Default)]
struct FpsTextVisibility(bool);

#[autodefault]
pub fn spawn_fps_text(commands: &mut Commands, fonts: &FontAssets) -> Entity {
    let text_style = TextStyle {
        font: fonts.andy_regular.clone(),
        font_size: 20.,
        color: Color::WHITE,
    };

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                margin: UiRect {
                    left: Val::Px(5.),
                    bottom: Val::Px(5.),
                },
            },
            text: Text {
                sections: vec![TextSection {
                    value: "".to_string(),
                    style: text_style.to_owned(),
                }],
                alignment: TextAlignment::CENTER,
            },
            visibility: Visibility { is_visible: false },
        })
        .insert(FpsText)
        .insert(FpsTextTimer(Timer::new(Duration::from_secs(1), true)))
        .insert(Name::new("FPS text"))
        .id()
}

fn toggle_fps_text_visibility(
    input: Res<Input<KeyCode>>,
    mut fps_text_visibility: ResMut<FpsTextVisibility>,
) {
    if input.just_pressed(KeyCode::F10) {
        fps_text_visibility.0 = !fps_text_visibility.0;
    }
}

fn set_fps_text_visibility(
    mut query: Query<&mut Visibility, With<FpsText>>,
    fps_text_visibility: Res<FpsTextVisibility>,
) {
    if fps_text_visibility.is_changed() {
        for mut visibility in query.iter_mut() {
            visibility.is_visible = fps_text_visibility.0;
        }
    }
}

fn update_fps_text(
    time: Res<Time>,
    diagnostics: Res<Diagnostics>,
    fps_text_visibility: Res<FpsTextVisibility>,
    mut query: Query<(&mut Text, &mut FpsTextTimer), With<FpsText>>,
) {
    if fps_text_visibility.0 {
        for (mut text, mut timer) in query.iter_mut() {
            if timer.tick(time.delta()).just_finished() {
                if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                    text.sections[0].value = format!("{:.0}", fps.value().unwrap_or(0.));
                }
            }
        }
    }
}
