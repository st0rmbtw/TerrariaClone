use autodefault::autodefault;
use bevy::{prelude::{Color, Commands, Entity, Name, TextBundle, Visibility, Component}, text::{TextStyle, TextSection, TextAlignment, Text}, ui::{Style, UiRect, Val}};

use crate::plugins::assets::FontAssets;

#[derive(Component)]
pub(crate) struct FpsText;

#[autodefault]
pub(super) fn spawn_fps_text(commands: &mut Commands, font_assets: &FontAssets) -> Entity {
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