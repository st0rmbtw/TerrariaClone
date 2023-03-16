use autodefault::autodefault;
use bevy::{prelude::{Res, Input, KeyCode, ResMut, Visibility, With, Query, Name, TextBundle, Color, Commands, Entity, DetectChanges}, time::Time, text::{Text, TextSection, TextAlignment, TextStyle}, ui::{Style, UiRect, Val}};

use crate::{plugins::assets::FontAssets, util};

use super::{FpsTextVisibility, FpsText};

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
        .insert(Name::new("FPS Text"))
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
    mut query_fps_text: Query<&mut Visibility, With<FpsText>>,
    fps_text_visibility: Res<FpsTextVisibility>,
) {
    if fps_text_visibility.is_changed() {
        let visibility = query_fps_text.single_mut();
        util::set_visibility(visibility, fps_text_visibility.0);
    }
}

pub fn update_fps_text(
    time: Res<Time>,
    fps_text_visibility: Res<FpsTextVisibility>,
    mut query_fps_text: Query<&mut Text, With<FpsText>>,
) {
    if fps_text_visibility.0 {
        let mut text = query_fps_text.single_mut();
        let fps = 1. / time.delta_seconds();
        text.sections[0].value = format!("{:.0}", fps);
    }
}