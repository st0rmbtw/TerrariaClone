use std::time::Duration;

use bevy::{prelude::{Visibility, Name, Color, TextBundle, Entity, Commands, NodeBundle, BuildChildren}, ui::{Interaction, Style, UiRect, Val, AlignItems, JustifyContent}, text::{TextAlignment, TextStyle, Text}, utils::default};
use interpolation::EaseFunction;

use crate::{plugins::assets::FontAssets, animation::{Animator, Tween, RepeatStrategy}, language::LanguageContent, common::lens::TextFontSizeLens};

use super::{SettingsButtonContainer, SettingsButton};

pub(crate) fn spawn_ingame_settings_button(
    commands: &mut Commands, 
    fonts: &FontAssets,
    language_content: &LanguageContent
) -> Entity {
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        RepeatStrategy::MirroredRepeat,
        Duration::from_millis(150),
        TextFontSizeLens {
            start: 32.,
            end: 38.,
        },
    );

    commands
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.)),
                width: Val::Px(100.),
                height: Val::Px(38.),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(SettingsButtonContainer)
        .with_children(|c| {
            c.spawn(TextBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_shrink: 0.,
                    ..default()
                },
                text: Text::from_section(
                    language_content.ui.settings.clone(),
                    TextStyle {
                        font: fonts.andy_bold.clone_weak(),
                        font_size: 32.,
                        color: Color::WHITE,
                    },
                ).with_alignment(TextAlignment::Center),
                ..default()
            })
            .insert(Name::new("Settings button"))
            .insert(Interaction::default())
            .insert(SettingsButton)
            .insert(Animator::new(tween));
        })
        .id()
}