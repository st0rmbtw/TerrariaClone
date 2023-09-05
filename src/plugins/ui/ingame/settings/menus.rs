use bevy::{prelude::{Entity, Commands, NodeBundle, default, BuildChildren, Color, ChildBuilder}, text::TextStyle, ui::{Style, FlexDirection, JustifyContent, AlignItems, Val}};

use crate::{plugins::{ui::{menu::{builders::{menu, menu_button, menu_text, slider_layout, menu_slider, slider_name_text, slider_value_text, spacer}, MENU_BUTTON_COLOR}, components::{MusicVolumeSlider, SoundVolumeSlider, MusicVolumeSliderOutput, SoundVolumeSliderOutput, ZoomSlider, ZoomSliderOutput}}, assets::{FontAssets, UiAssets}, config::{MusicVolume, SoundVolume}, camera::resources::Zoom}, language::LanguageContent};

use super::{components::{MenuTabs, buttons::*, TabMenu, TabButton}, SelectedTab, TAB_BUTTON_TEXT_SIZE};

#[inline(always)]
fn row(commands: &mut ChildBuilder, gap: f32, builder: impl FnOnce(&mut ChildBuilder)) {
    commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(gap),
            ..default()
        },
        ..default()
    }).with_children(builder);
}

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
    zoom: &Zoom
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
                row(first_column, 5., |builder| {
                    slider_name_text(builder, slider_text_style.clone(), language_content.ui.music.clone());
                    slider_value_text(builder, slider_text_style.clone(), music_volume.get(), 50., MusicVolumeSliderOutput);
                });

                row(first_column, 5., |builder| {
                    slider_name_text(builder, slider_text_style.clone(), language_content.ui.sound.clone());
                    slider_value_text(builder, slider_text_style.clone(), music_volume.get(), 50., SoundVolumeSliderOutput);
                });
            }, 
            |second_column| {
                menu_slider(second_column, &ui_assets, music_volume.get(), Color::WHITE, 0.8, Val::Px(slider_text_style.font_size), MusicVolumeSlider);
                menu_slider(second_column, &ui_assets, sound_volume.get(), Color::WHITE, 0.8, Val::Px(slider_text_style.font_size), SoundVolumeSlider);
            }
        );

        spacer(builder, 15.);

        menu_text(builder, caption_text_style.clone(), "Zoom");

        slider_layout(
            builder,
            0.,
            |first_column| {
                row(first_column, 5., |builder| {
                    slider_name_text(builder, slider_text_style.clone(), "Zoom");
                    slider_value_text(builder, slider_text_style.clone(), zoom.get(), 55., ZoomSliderOutput);
                });
            }, 
            |second_column| {
                menu_slider(second_column, &ui_assets, zoom.get(), Color::WHITE, 0.8, Val::Px(slider_text_style.font_size), ZoomSlider);
            }
        );
    });
}