use bevy::{prelude::{Entity, Commands, NodeBundle, default, BuildChildren, Color, ChildBuilder}, text::TextStyle, ui::{Style, FlexDirection, JustifyContent, AlignItems, Val}};

use crate::{plugins::{ui::{menu::{builders::{menu, menu_button, slider_layout, menu_slider, slider_name_text, slider_value_text, spacer, menu_text_localized}, MENU_BUTTON_COLOR}, components::{MusicVolumeSlider, SoundVolumeSlider, MusicVolumeSliderOutput, SoundVolumeSliderOutput, ZoomSlider, ZoomSliderOutput, ToggleTileGridButton}}, assets::{FontAssets, UiAssets}, config::{MusicVolume, SoundVolume}, camera::resources::Zoom}, language::{keys::UIStringKey, LocalizedText, args}};

use super::{components::{MenuTabs, buttons::*, TabMenu, TabButton, TabMenuButton}, SelectedTab, TAB_BUTTON_TEXT_SIZE};

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
            UIStringKey::General,
            (TabButton, SelectedTab::General, GeneralButton)
        );
        menu_button(
            builder,
            text_style.clone(),
            UIStringKey::Interface,
            (TabButton, SelectedTab::Interface, InterfaceButton)
        );
        menu_button(
            builder,
            text_style.clone(),
            UIStringKey::Video,
            (TabButton, SelectedTab::Video, VideoButton)
        );
        menu_button(
            builder,
            text_style.clone(),
            UIStringKey::Cursor,
            (TabButton,SelectedTab::Cursor, CursorButton)
        );
        menu_button(
            builder,
            text_style.clone(),
            UIStringKey::CloseMenu,
            (TabButton, CloseMenuButton)
        );
        menu_button(
            builder,
            text_style,
            UIStringKey::SaveAndExit,
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
        menu_text_localized(builder, caption_text_style.clone(), UIStringKey::Volume);

        slider_layout(
            builder,
            0.,
            AlignItems::Center,
            |first_column| {
                row(first_column, 5., |builder| {
                    slider_name_text(builder, slider_text_style.clone(), UIStringKey::Music);
                    slider_value_text(builder, slider_text_style.clone(), music_volume.get(), 50., MusicVolumeSliderOutput);
                });

                row(first_column, 5., |builder| {
                    slider_name_text(builder, slider_text_style.clone(), UIStringKey::Sound);
                    slider_value_text(builder, slider_text_style.clone(), music_volume.get(), 50., SoundVolumeSliderOutput);
                });
            },
            AlignItems::Center,
            |second_column| {
                menu_slider(second_column, ui_assets, music_volume.get(), Color::WHITE, 0.8, Val::Px(slider_text_style.font_size), MusicVolumeSlider);
                menu_slider(second_column, ui_assets, sound_volume.get(), Color::WHITE, 0.8, Val::Px(slider_text_style.font_size), SoundVolumeSlider);
            }
        );

        spacer(builder, 15.);

        menu_text_localized(builder, caption_text_style.clone(), UIStringKey::Zoom);

        slider_layout(
            builder,
            0.,
            AlignItems::Center,
            |first_column| {
                row(first_column, 5., |builder| {
                    slider_name_text(builder, slider_text_style.clone(), UIStringKey::Zoom);
                    slider_value_text(builder, slider_text_style.clone(), zoom.get(), 55., ZoomSliderOutput);
                });
            },
            AlignItems::Center,
            |second_column| {
                menu_slider(second_column, ui_assets, zoom.get(), Color::WHITE, 0.8, Val::Px(slider_text_style.font_size), ZoomSlider);
            }
        );
    });
}

pub(super) fn interface_menu(
    commands: &mut Commands,
    container: Entity,
    font_assets: &FontAssets,
    show_tile_grid: bool
) {
    let text_style = TextStyle {
        font: font_assets.andy_bold.clone_weak(),
        font_size: 26.,
        color: MENU_BUTTON_COLOR,
    };

    let status = if show_tile_grid { UIStringKey::On } else { UIStringKey::Off };

    menu(TabMenu, commands, container, 5., |builder| {    
        menu_button(
            builder,
            text_style,
            LocalizedText::new(UIStringKey::TileGrid, "{} {}", args![status]),
            (TabMenuButton, ToggleTileGridButton)
        );
    })
}