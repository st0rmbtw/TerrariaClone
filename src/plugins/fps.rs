use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::*, diagnostic::{FrameTimeDiagnosticsPlugin, Diagnostics}};
use iyes_loopless::{prelude::{AppLooplessStateExt, ConditionSet, IntoConditionalSystem}};

use crate::state::GameState;

use super::FontAssets;

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(update_fps_text)
                    .into()
            );
    }
} 

#[derive(Component)]
struct FpsText;

#[derive(Component, Deref, DerefMut)]
struct FpsTextTimer(Timer);

#[autodefault]
pub fn spawn_fps_text(
    commands: &mut Commands, 
    fonts: &FontAssets
) -> Entity {
    let text_style = TextStyle {
        font: fonts.andy_regular.clone(),
        font_size: 20.,
        color: Color::WHITE,
    };

    commands.spawn_bundle(TextBundle {
        style: Style {
            margin: UiRect { 
                left: Val::Px(5.), 
                bottom: Val::Px(5.)
            }
            
        },
        text: Text {
            sections: vec![
                TextSection {
                    value: "".to_string(),
                    style: text_style.to_owned()
                },
            ],
            alignment: TextAlignment::CENTER
        },
    })
    .insert(FpsText)
    .insert(FpsTextTimer(Timer::new(Duration::from_secs(1), true)))
    .insert(Name::new("FPS text"))
    .id()
}

fn update_fps_text(
    time: Res<Time>,
    diagnostics: Res<Diagnostics>,
    mut query: Query<(&mut Text, &mut FpsTextTimer), With<FpsText>>,
) {
    for (mut text, mut timer) in query.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                text.sections[0].value = format!("{:.0}", fps.value().unwrap_or(0.));
            }
        }
    }
}
