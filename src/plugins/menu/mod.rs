mod settings;
mod celestial_body;
mod components;
mod systems;
mod role;
mod builders;

use std::time::Duration;

use components::*;
use interpolation::EaseFunction;
use systems::*;

use bevy::{prelude::{Plugin, App, IntoSystemConfigs, OnEnter, OnExit, Color, Component, Startup, Update, Event, KeyCode, PostUpdate, Button, EventWriter, Res, Query, Entity, With, Commands, Name, NodeBundle, BuildChildren, ImageBundle, default, Visibility, TextBundle, Transform, Quat, Vec3, Camera2dBundle, Camera2d, State, ResMut, NextState, EventReader}, input::common_conditions::input_just_pressed, app::AppExit, text::{TextStyle, Text, TextSection}, ui::{Style, PositionType, AlignSelf, Val, UiRect, FlexDirection, UiImage}, core_pipeline::clear_color::ClearColorConfig};
use crate::{common::{state::{GameState, MenuState, SettingsMenuState}, conditions::{on_btn_clicked, in_menu_state}, systems::{animate_button_scale, play_sound_on_hover}, lens::TransformLens}, parallax::{parallax_animation_system, ParallaxSet}, language::LanguageContent, animation::{Animator, RepeatCount, Tween, RepeatStrategy}};
use self::{settings::SettingsMenuPlugin, celestial_body::CelestialBodyPlugin, builders::{menu, menu_button}};
use super::{slider::Slider, settings::{FullScreen, ShowTileGrid, VSync, Resolution, CursorColor, MusicVolume, SoundVolume, Settings}, assets::{FontAssets, UiAssets}, fps::FpsText, camera::components::MainCamera, audio::{PlaySoundEvent, SoundType, PlayMusicEvent, MusicType}};

pub(crate) const TEXT_COLOR: Color = Color::rgb(0.58, 0.58, 0.58);
pub(super) const MENU_BUTTON_FONT_SIZE: f32 = 42.;

#[derive(Component)]
pub(crate) struct DespawnOnMenuExit;

#[derive(Component)]
pub(super) struct BackButton;

#[derive(Component)]
pub(super) struct ApplyButton;

#[derive(Event)]
pub(super) struct BackEvent;

#[derive(Event)]
pub(super) struct EnterEvent(pub(super) GameState);

pub(crate) struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BackEvent>();
        app.add_event::<EnterEvent>();

        app.add_plugins((CelestialBodyPlugin, SettingsMenuPlugin));

        app.add_systems(Startup, setup_camera);
        app.add_systems(OnExit(GameState::AssetLoading), (spawn_menu_container, play_music));

        app.add_systems(OnEnter(GameState::Menu(MenuState::Main)), setup_main_menu);
        app.add_systems(OnExit(GameState::Menu(MenuState::Main)), despawn_with::<Menu>);

        app.add_systems(OnEnter(GameState::InGame), despawn_with::<DespawnOnMenuExit>);

        app.add_systems(
            Update,
            (
                send_back_event.run_if(on_btn_clicked::<BackButton>),
                send_back_event.run_if(input_just_pressed(KeyCode::Escape)),
            )
        );

        app.add_systems(
            PostUpdate,
            (
                handle_back_event,
                handle_enter_event    
            ).run_if(in_menu_state)
        );
        
        app.add_systems(
            Update,
            (
                parallax_animation_system(150.).in_set(ParallaxSet::FollowCamera),
                animate_button_scale::<Button>,
                animate_button_color,
                animate_slider_border_color,
                play_sound_on_hover::<Button>,
                play_sound_on_hover::<Slider>,
            )
            .run_if(in_menu_state)
        );

        app.add_systems(
            Update,
            (
                send_enter_event(GameState::WorldLoading)
                    .run_if(on_btn_clicked::<SinglePlayerButton>),
                send_enter_event(GameState::Menu(MenuState::Settings(SettingsMenuState::Main)))
                    .run_if(on_btn_clicked::<SettingsButton>),
                exit_clicked.run_if(on_btn_clicked::<ExitButton>),
            )
            .run_if(in_menu_state)
        );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("MenuCamera"),
        MainCamera,
        DespawnOnMenuExit,
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None
            },
            ..default()
        },
    ));
}

fn spawn_menu_container(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    font_assets: Res<FontAssets>
) {
    let logo_animation = Tween::new(
        EaseFunction::SineInOut,
        RepeatStrategy::MirroredRepeat,
        Duration::from_secs(10),
        TransformLens {
            start: Transform {
                scale: Vec3::splat(1.),
                rotation: Quat::from_rotation_z(-5f32.to_radians()),
                ..default()
            },
            end: Transform {
                scale: Vec3::splat(1.1),
                rotation: Quat::from_rotation_z(5f32.to_radians()),
                ..default()
            }
        }
    ).with_repeat_count(RepeatCount::Infinite);

    let fps_text_style = TextStyle {
        font: font_assets.andy_regular.clone_weak(),
        font_size: 24.,
        color: Color::WHITE,
    };

    commands.spawn((
        FpsText,
        DespawnOnMenuExit,
        Name::new("FPS Text"),
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                margin: UiRect {
                    left: Val::Px(5.),
                    top: Val::Px(5.),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection::from_style(fps_text_style)
                ],
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
    ));

    commands
        .spawn((
            MenuContainer,
            DespawnOnMenuExit,
            Name::new("MenuContainer"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    padding: UiRect::all(Val::Px(50.)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            }
        ))
        .with_children(|children| {
            children.spawn((
                Name::new("LogoImage"),
                Animator::new(logo_animation),
                ImageBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                    image: UiImage {
                        texture: ui_assets.logo.clone_weak(),
                        ..default()
                    },
                    ..default()
                },
            ));
        });
}

fn setup_main_menu(
    mut commands: Commands, 
    fonts: Res<FontAssets>,
    language_content: Res<LanguageContent>,
    query_container: Query<Entity, With<MenuContainer>>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: 56.,
        color: TEXT_COLOR,
    };

    let container = query_container.single();

    menu(Menu, &mut commands, container, 65., |builder| {
        menu_button(
            builder,
            text_style.clone(),
            language_content.ui.single_player.clone(),
            SinglePlayerButton,
        );
        menu_button(
            builder, 
            text_style.clone(), 
            language_content.ui.settings.clone(), 
            SettingsButton,
        );
        menu_button(
            builder, 
            text_style, 
            language_content.ui.exit.clone(), 
            ExitButton,
        );
    });
}

fn play_music(
    mut play_music: EventWriter<PlayMusicEvent>
) {
    play_music.send(PlayMusicEvent(MusicType::TitleScreen));
}

fn handle_back_event(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut play_sound: EventWriter<PlaySoundEvent>,
    mut back_events: EventReader<BackEvent>
) {
    if back_events.iter().last().is_some() {
        next_state.set(state.back());
        play_sound.send(PlaySoundEvent(SoundType::MenuClose));
    }
}

fn handle_enter_event(
    mut next_state: ResMut<NextState<GameState>>,
    mut play_sound: EventWriter<PlaySoundEvent>,
    mut enter_events: EventReader<EnterEvent>
) {
    if let Some(event) = enter_events.iter().last() {
        next_state.set(event.0);
        play_sound.send(PlaySoundEvent(SoundType::MenuOpen));
    }
}

fn exit_clicked(
    mut ev: EventWriter<AppExit>,
    fullscreen: Res<FullScreen>,
    show_tile_grid: Res<ShowTileGrid>,
    vsync: Res<VSync>,
    resolution: Res<Resolution>,
    cursor_color: Res<CursorColor>,
    music_volume: Res<MusicVolume>,
    sound_volume: Res<SoundVolume>
) {
    ev.send(AppExit);

    crate::plugins::settings::save_settings(Settings {
        full_screen: fullscreen.0,
        show_tile_grid: show_tile_grid.0,
        vsync: vsync.0,
        resolution: *resolution,
        cursor_color: *cursor_color,
        sound_volume: sound_volume.slider_value(),
        music_volume: music_volume.slider_value()
    });
}