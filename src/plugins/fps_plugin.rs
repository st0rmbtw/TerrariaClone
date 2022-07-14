use std::time::Duration;

use bevy::{prelude::*, diagnostic::{FrameTimeDiagnosticsPlugin, Diagnostics}};


pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_startup_system(setup_ui)
            .add_system(update_fps_text)
            .insert_resource(Timer::new(Duration::from_secs(1), true));
    }
} 

#[derive(Component)]
struct FpsText;

fn setup_ui(mut commands: Commands, assets: Res<AssetServer>) {
    let text_style = TextStyle {
        font: assets.load("fonts/andyb.ttf"),
        font_size: 26.,
        color: Color::GREEN,
    };

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                margin: Rect {
                    top: Val::Px(10.),
                    left: Val::Px(10.),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: text_style.to_owned()
                    },
                    TextSection {
                        value: "".to_string(),
                        style: text_style
                    },
                ],
                alignment: TextAlignment { 
                    vertical: VerticalAlign::Top, 
                    horizontal: HorizontalAlign::Left
                }
            },
            ..default()
        })
        .insert(FpsText);
}

fn update_fps_text(
    time: Res<Time>,
    mut timer: ResMut<Timer>,
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    if timer.tick(time.delta()).just_finished() {
        for mut text in query.iter_mut() {
            if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                text.sections[1].value = format!("{:.0}", fps.sum());
            }
        }
    }
}
