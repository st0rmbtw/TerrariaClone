use bevy::{prelude::{Plugin, App, Commands, OnEnter, Query, Entity, With, Res, Color, OnExit, Update, IntoSystemConfigs, in_state, Component, Changed, EventWriter}, text::TextStyle};

use crate::{common::state::{GameState, MenuState, SettingsMenuState}, plugins::{menu::{systems::{menu, menu_text, despawn_with, control_buttons_layout, control_button, slider_value_text, menu_slider, slider_layout, bind_slider_to_output}, components::MenuContainer, MENU_BUTTON_FONT_SIZE, BackButton, TEXT_COLOR}, assets::{FontAssets, UiAssets}, settings::{MusicVolume, SoundVolume}, slider::Slider, audio::{UpdateMusicVolume, UpdateSoundVolume}}, language::LanguageContent};

pub(super) struct VolumeMenuPlugin;
impl Plugin for VolumeMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Menu(MenuState::Settings(SettingsMenuState::Volume))),
            setup_volume_menu
        );
        app.add_systems(
            OnExit(GameState::Menu(MenuState::Settings(SettingsMenuState::Volume))),
            despawn_with::<VolumeMenu>
        );

        app.add_systems(
            Update,
            (
                bind_slider_to_output::<MusicVolumeSlider, MusicVolumeSliderOutput>,
                bind_slider_to_output::<SoundVolumeSlider, SoundVolumeSliderOutput>,
                update_music_volume,
                update_sound_volume,
            )
            .run_if(in_state(GameState::Menu(MenuState::Settings(SettingsMenuState::Volume))))
        );
    }
}

#[derive(Component)]
struct VolumeMenu;

#[derive(Component)]
struct MusicVolumeSlider;

#[derive(Component)]
struct SoundVolumeSlider;

#[derive(Component)]
struct MusicVolumeSliderOutput;

#[derive(Component)]
struct SoundVolumeSliderOutput;

fn setup_volume_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    ui_assets: Res<UiAssets>,
    language_content: Res<LanguageContent>,
    query_container: Query<Entity, With<MenuContainer>>,
    music_volume: Res<MusicVolume>,
    sound_volume: Res<SoundVolume>,
) {
    let title_text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: Color::WHITE,
    };

    let button_text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: TEXT_COLOR,
    };

    let slider_text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: 36.,
        color: Color::rgb(0.9, 0.9, 0.9),
    };

    let container = query_container.single();

    menu(VolumeMenu, &mut commands, container, 50., |builder| {
        menu_text(builder, title_text_style, language_content.ui.volume.clone());

        slider_layout(
            builder, 
            |slider_builder| {
                menu_slider(slider_builder, &ui_assets, slider_text_style.clone(), "Music", music_volume.slider_value(), Color::WHITE, MusicVolumeSlider);
                menu_slider(slider_builder, &ui_assets, slider_text_style.clone(), "Sound", sound_volume.slider_value(), Color::WHITE, SoundVolumeSlider);
            }, 
            |output_builder| {
                slider_value_text(output_builder, slider_text_style.clone(), music_volume.slider_value(), MusicVolumeSliderOutput);
                slider_value_text(output_builder, slider_text_style.clone(), music_volume.slider_value(), SoundVolumeSliderOutput);
            }
        );

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, button_text_style, language_content.ui.back.clone(), BackButton);
        });
    });
}

fn update_music_volume(
    query_slider: Query<&Slider, (With<MusicVolumeSlider>, Changed<Slider>)>,
    mut update_music_volume: EventWriter<UpdateMusicVolume>
) {
    if let Ok(slider) = query_slider.get_single() {
        update_music_volume.send(UpdateMusicVolume(slider.value() / 100.));
    }
}

fn update_sound_volume(
    query_slider: Query<&Slider, (With<SoundVolumeSlider>, Changed<Slider>)>,
    mut update_sound_volume: EventWriter<UpdateSoundVolume>
) {
    if let Ok(slider) = query_slider.get_single() {
        update_sound_volume.send(UpdateSoundVolume(slider.value() / 100.));
    }
}