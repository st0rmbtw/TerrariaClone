use bevy::{prelude::{Plugin, App, Commands, OnEnter, Query, Entity, With, Res, Color, OnExit, NodeBundle, BuildChildren, Component}, text::TextStyle, utils::default, ui::{Style, Val, FlexDirection, JustifyContent, AlignItems}};

use crate::{
    common::{state::{MenuState, SettingsMenuState}, systems::despawn_with},
    plugins::{
        ui::{menu::{components::{MenuContainer, MenuButton}, MENU_BUTTON_FONT_SIZE, BackButton, MENU_BUTTON_COLOR, builders::{menu, slider_layout, menu_slider, slider_value_text, control_buttons_layout, control_button, slider_name_text, menu_text_localized}}, components::{MusicVolumeSliderOutput, SoundVolumeSliderOutput, SoundVolumeSlider, MusicVolumeSlider}},
        assets::{FontAssets, UiAssets},
        config::{MusicVolume, SoundVolume},
    }, language::keys::UIStringKey,
};

pub(super) struct VolumeMenuPlugin;
impl Plugin for VolumeMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MenuState::Settings(SettingsMenuState::Volume)),
            setup_volume_menu
        );

        app.add_systems(
            OnExit(MenuState::Settings(SettingsMenuState::Volume)),
            despawn_with::<VolumeMenu>
        );
    }
}

#[derive(Component)]
struct VolumeMenu;

fn setup_volume_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    ui_assets: Res<UiAssets>,
    music_volume: Res<MusicVolume>,
    sound_volume: Res<SoundVolume>,
    query_container: Query<Entity, With<MenuContainer>>,
) {
    let title_text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: Color::WHITE,
    };

    let button_text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: MENU_BUTTON_COLOR,
    };

    let slider_text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: 32.,
        color: Color::rgb(0.9, 0.9, 0.9),
    };

    let container = query_container.single();

    menu(VolumeMenu, &mut commands, container, 5., |builder| {
        menu_text_localized(builder, title_text_style, UIStringKey::Volume);

        slider_layout(
            builder,
            5.,
            AlignItems::Start,
            |first_column| {
                first_column.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.),
                        ..default()
                    },
                    ..default()
                }).with_children(|b| {
                    menu_slider(b, &ui_assets, music_volume.get(), Color::WHITE, 1., Val::Auto, MusicVolumeSlider);
                    slider_name_text(b, slider_text_style.clone(), UIStringKey::Music);
                });
                
                first_column.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.),
                        ..default()
                    },
                    ..default()
                }).with_children(|b| {
                    menu_slider(b, &ui_assets, sound_volume.get(), Color::WHITE, 1., Val::Auto, SoundVolumeSlider);
                    slider_name_text(b, slider_text_style.clone(), UIStringKey::Sound);
                });
            }, 
            AlignItems::Start,
            |second_column| {
                slider_value_text(second_column, slider_text_style.clone(), music_volume.get(), 100., MusicVolumeSliderOutput);
                slider_value_text(second_column, slider_text_style.clone(), sound_volume.get(), 100., SoundVolumeSliderOutput);
            }
        );

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, button_text_style, UIStringKey::Back, (MenuButton, BackButton));
        });
    });
}

