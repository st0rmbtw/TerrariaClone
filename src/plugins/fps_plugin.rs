use std::time::Duration;

use bevy::{prelude::*, diagnostic::{FrameTimeDiagnosticsPlugin, Diagnostics}};

use super::SPAWN_PLAYER_UI_LABEL;

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_startup_system(spawn_fps_text.after(SPAWN_PLAYER_UI_LABEL))
            .add_system(update_fps_text);
    }
} 

#[derive(Component)]
struct FpsText;

#[derive(Component, Deref, DerefMut)]
struct FpsTextTimer(Timer);

fn spawn_fps_text(mut commands: Commands, assets: Res<AssetServer>) {
    let text_style = TextStyle {
        font: assets.load("fonts/andyb.ttf"),
        font_size: 22.,
        color: Color::GREEN,
    };

    commands.spawn_bundle(NodeBundle {
        style: Style {
            align_items: AlignItems::FlexEnd,
            align_self: AlignSelf::FlexStart,
            align_content: AlignContent::FlexEnd,
            justify_content: JustifyContent::FlexEnd,
            size: Size { 
                width: Val::Percent(100.), 
                height: Val::Auto 
            },
            margin: Rect {
                bottom: Val::Px(10.),
                right: Val::Px(10.),
                ..default()
            },
            ..default()
        },
        color: Color::rgba(0., 0., 0., 0.).into(),
        global_transform: GlobalTransform::from_xyz(0., 0., 1.),
        ..default()
    }).with_children(|children| {
        children.spawn_bundle(TextBundle {
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
                    horizontal: HorizontalAlign::Center,
                    vertical: VerticalAlign::Center
                }
            },
            ..default()
        })
        .insert(FpsText)
        .insert(FpsTextTimer(Timer::new(Duration::from_secs(1), true)));
    }); 
}

fn update_fps_text(
    time: Res<Time>,
    diagnostics: Res<Diagnostics>,
    mut query: Query<(&mut Text, &mut FpsTextTimer), With<FpsText>>,
) {
    for (mut text, mut timer) in query.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                text.sections[1].value = format!("{:.0}", fps.sum());
            }
        }
    }
}
