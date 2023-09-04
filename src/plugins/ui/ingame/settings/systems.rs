use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Entity, NodeBundle, Visibility, default, TextBundle, Color, Name, Button, BuildChildren, Res}, ui::{Style, JustifyContent, AlignItems, AlignSelf, UiRect, Val, Interaction, PositionType, FlexDirection, Display}, text::{Text, TextAlignment, TextStyle}};
use interpolation::EaseFunction;

use crate::{language::LanguageContent, plugins::{assets::FontAssets, ui::menu::{builders::{menu, menu_button}, TEXT_COLOR}, DespawnOnGameExit}, animation::{Tween, RepeatStrategy, Animator}, common::lens::TextFontSizeLens};

use super::components::{InGameSettingsMenuContainer, InGameSettingsButton, InGameSettingsButtonContainer, InGameSettingsMenuTabs, GeneralButton, InterfaceButton, VideoButton, CursorButton, CloseMenuButton, SaveAndExitButton};

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
                align_self: AlignSelf::End,
                padding: UiRect::all(Val::Px(10.)),
                width: Val::Px(100.),
                height: Val::Px(38.),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(InGameSettingsButtonContainer)
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
            .insert(Name::new("SettingsButton"))
            .insert(Interaction::default())
            .insert(InGameSettingsButton)
            .insert(Button)
            .insert(Animator::new(tween));
        })
        .id()
}

#[autodefault]
pub(super) fn spawn_settings_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    language_content: Res<LanguageContent>
) {
    let text_style = TextStyle {
        font: font_assets.andy_bold.clone_weak(),
        font_size: 22.,
        color: Color::WHITE
    };

    let tabs_container = commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            padding: UiRect::vertical(Val::Px(10.))
        },
        background_color: Color::rgb_u8(54, 53, 131).into()
    }).id();

    menu(InGameSettingsMenuTabs, &mut commands, tabs_container, 5., |builder| {
        let text_style = TextStyle {
            font: font_assets.andy_bold.clone_weak(),
            font_size: 36.,
            color: TEXT_COLOR
        };

        menu_button(builder, text_style.clone(), language_content.ui.general.clone(), GeneralButton);
        menu_button(builder, text_style.clone(), language_content.ui.interface.clone(), InterfaceButton);
        menu_button(builder, text_style.clone(), language_content.ui.video.clone(), VideoButton);
        menu_button(builder, text_style.clone(), language_content.ui.cursor.clone(), CursorButton);
        menu_button(builder, text_style.clone(), language_content.ui.close_menu.clone(), CloseMenuButton);
        menu_button(builder, text_style, language_content.ui.save_and_exit.clone(), SaveAndExitButton);
    });

    commands.spawn((
        InGameSettingsMenuContainer,
        DespawnOnGameExit,
        NodeBundle {
            style: Style {
                width: Val::Px(706.),
                height: Val::Px(516.),
                position_type: PositionType::Absolute,
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Column,
                padding: UiRect::top(Val::Px(22.)),
                display: Display::None
            },
            background_color: Color::rgb_u8(22, 10, 62).with_a(0.9).into(),
        }
    )).with_children(|builder| {
        builder.spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::Center
            },
            text: Text::from_section(language_content.ui.settings_menu.clone(), text_style),
        });

        builder.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                padding: UiRect::all(Val::Px(28.)),
                column_gap: Val::Px(30.)
            }
        })
        .add_child(tabs_container)
        .with_children(|builder| {
            builder.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.)
                },
                background_color: Color::rgb_u8(54, 53, 131).into()
            });
        });
    });
}
