use autodefault::autodefault;
use bevy::{prelude::{Color, Commands, Name, TextBundle, Visibility, Component, Plugin, App, OnEnter, Res}, text::{TextStyle, TextSection, TextAlignment, Text}, ui::{Style, Val, PositionType}};

use crate::{plugins::{assets::FontAssets, DespawnOnGameExit}, common::state::GameState};

pub(super) struct FpsUiPlugin;

impl Plugin for FpsUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_fps_text);
    }
}

#[derive(Component)]
pub(crate) struct FpsText;

#[autodefault]
pub(super) fn spawn_fps_text(mut commands: Commands, font_assets: Res<FontAssets>) {
    let text_style = TextStyle {
        font: font_assets.andy_regular.clone_weak(),
        font_size: 20.,
        color: Color::WHITE,
    };

    commands.spawn((
        FpsText,
        Name::new("FPS Text"),
        DespawnOnGameExit,
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(5.),
                bottom: Val::Px(0.),
            },
            text: Text {
                sections: vec![
                    TextSection::from_style(text_style)
                ],
                alignment: TextAlignment::Left,
            },
            visibility: Visibility::Hidden,
        }
    ));
}