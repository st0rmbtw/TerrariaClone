use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Res, Input, KeyCode, ResMut, Visibility, With, Query, Name, TextBundle, Color, Commands, Entity, DetectChanges}, time::{Time, Timer, TimerMode}, diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin}, text::{Text, TextSection, TextAlignment, TextStyle}, ui::{Style, UiRect, Val}};

use crate::plugins::assets::FontAssets;

use super::{FpsTextVisibility, FpsText, FpsTextTimer};

#[autodefault]
pub fn spawn_fps_text(commands: &mut Commands, fonts: &FontAssets) -> Entity {
    let text_style = TextStyle {
        font: fonts.andy_regular.clone(),
        font_size: 20.,
        color: Color::WHITE,
    };

    commands
        .spawn(TextBundle {
            style: Style {
                margin: UiRect {
                    left: Val::Px(5.),
                    bottom: Val::Px(5.),
                },
            },
            text: Text {
                sections: vec![TextSection {
                    value: "".to_string(),
                    style: text_style,
                }],
                alignment: TextAlignment::Center,
            },
            visibility: Visibility::Hidden,
        })
        .insert(FpsText)
        .insert(FpsTextTimer(Timer::new(Duration::from_secs(1), TimerMode::Repeating)))
        .insert(Name::new("FPS text"))
        .id()
}

pub fn toggle_fps_text_visibility(
    input: Res<Input<KeyCode>>,
    mut fps_text_visibility: ResMut<FpsTextVisibility>,
) {
    if input.just_pressed(KeyCode::F10) {
        fps_text_visibility.0 = !fps_text_visibility.0;
    }
}

pub fn set_fps_text_visibility(
    mut query: Query<&mut Visibility, With<FpsText>>,
    fps_text_visibility: Res<FpsTextVisibility>,
) {
    if fps_text_visibility.is_changed() {
        for mut visibility in query.iter_mut() {
            if fps_text_visibility.0 {
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Visible;
            }
        }
    }
}

pub fn update_fps_text(
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