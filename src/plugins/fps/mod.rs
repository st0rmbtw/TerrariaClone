use std::time::Duration;

use autodefault::autodefault;
use bevy::{
    prelude::{Plugin, IntoSystemConfig, resource_exists_and_equals, Condition, Component, Commands, Entity, Color, TextBundle, Res, KeyCode, Query, Visibility, With, Name},
    text::{TextStyle, Text, TextSection, TextAlignment},
    ui::{Style, UiRect, Val},
    time::{common_conditions::on_timer, Time},
    input::common_conditions::input_just_pressed,
};

use crate::common::helpers::toggle_visibility;
use super::{assets::FontAssets, ui::UiVisibility};

pub(crate) struct FpsPlugin;
impl Plugin for FpsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            (
                toggle_visibility::<FpsText>.run_if(input_just_pressed(KeyCode::F10)),
                update_fps_text.run_if(
                    resource_exists_and_equals(UiVisibility(true)).and_then(on_timer(Duration::from_secs(1)))
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
        .insert(Name::new("FPS Text"))
        .id()
}

fn update_fps_text(
    time: Res<Time>,
    mut query_fps_text: Query<&mut Text, With<FpsText>>,
) {
    let mut text = query_fps_text.single_mut();

    let fps = 1. / time.delta_seconds();
    text.sections[0].value = format!("{:.0}", fps);
}