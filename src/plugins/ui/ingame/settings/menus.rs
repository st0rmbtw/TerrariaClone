use bevy::{prelude::{Entity, Commands, NodeBundle, default, BuildChildren, Color}, text::TextStyle, ui::{Style, FlexDirection, JustifyContent, AlignItems, Val}};

use crate::{plugins::{ui::{menu::{builders::{menu, menu_button, menu_text, slider_layout, menu_slider, slider_name_text, slider_value_text}, MENU_BUTTON_COLOR}, components::{MusicVolumeSlider, SoundVolumeSlider, MusicVolumeSliderOutput, SoundVolumeSliderOutput}}, assets::{FontAssets, UiAssets}, config::{MusicVolume, SoundVolume}}, language::LanguageContent};

use super::{components::{MenuTabs, buttons::*, TabMenu, TabButton}, SelectedTab, TAB_BUTTON_TEXT_SIZE};


#[inline(always)]
pub(super) fn tabs_menu(
    commands: &mut Commands,
    font_assets: &FontAssets,
    language_content: &LanguageContent,
    container: Entity
) {
    menu(MenuTabs, commands, container, 5., |builder| {
        let text_style = TextStyle {
            font: font_assets.andy_bold.clone_weak(),
            font_size: TAB_BUTTON_TEXT_SIZE,
            color: MENU_BUTTON_COLOR
        };

        menu_button(
            builder,
            text_style.clone(),
            language_content.ui.general.clone(),
            (TabButton, SelectedTab::General, GeneralButton)
        );
        menu_button(
            builder,
            text_style.clone(),
            language_content.ui.interface.clone(),
            (TabButton, SelectedTab::Interface, InterfaceButton)
        );
        menu_button(
            builder,
            text_style.clone(),
            language_content.ui.video.clone(),
            (TabButton, SelectedTab::Video, VideoButton)
        );
        menu_button(
            builder,
            text_style.clone(),
            language_content.ui.cursor.clone(),
            (TabButton,SelectedTab::Cursor, CursorButton)
        );
        menu_button(
            builder,
            text_style.clone(),
            language_content.ui.close_menu.clone(),
            (TabButton, CloseMenuButton)
        );
        menu_button(
            builder,
            text_style, language_content.ui.save_and_exit.clone(),
            (TabButton, SaveAndExitButton)
        );
    });
}

#[inline(always)]
pub(super) fn general_menu(
    commands: &mut Commands,
    container: Entity,
    font_assets: &FontAssets,
    ui_assets: &UiAssets,
    language_content: &LanguageContent,
    music_volume: &MusicVolume,
    sound_volume: &SoundVolume,
) {
    let slider_text_style = TextStyle {
        font: font_assets.andy_bold.clone_weak(),
        font_size: 24.,
        color: MENU_BUTTON_COLOR,
    };

    let caption_text_style = TextStyle {
        font: font_assets.andy_bold.clone_weak(),
        font_size: 24.,
        color: Color::rgb(0.9, 0.9, 0.9)
    };

    menu(TabMenu, commands, container, 5., |builder| {
        menu_text(builder, caption_text_style.clone(), "Volume");

        slider_layout(
            builder,
            0.,
            |first_column| {
                first_column.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(5.),
                        ..default()
                    },
                    ..default()
                }).with_children(|b| {
                    slider_name_text(b, slider_text_style.clone(), language_content.ui.music.clone());
                    slider_value_text(b, slider_text_style.clone(), music_volume.slider_value(), 50., MusicVolumeSliderOutput);
                });

                first_column.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(5.),
                        ..default()
                    },
                    ..default()
                }).with_children(|b| {
                    slider_name_text(b, slider_text_style.clone(), language_content.ui.sound.clone());
                    slider_value_text(b, slider_text_style.clone(), music_volume.slider_value(), 50., SoundVolumeSliderOutput);
                });
            }, 
            |second_column| {
                menu_slider(second_column, &ui_assets, music_volume.slider_value(), Color::WHITE, 0.8, Val::Px(slider_text_style.font_size), MusicVolumeSlider);
                menu_slider(second_column, &ui_assets, sound_volume.slider_value(), Color::WHITE, 0.8, Val::Px(slider_text_style.font_size), SoundVolumeSlider);
            }
        );
    });
}