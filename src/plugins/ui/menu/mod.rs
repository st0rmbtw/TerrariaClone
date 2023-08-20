mod settings;
mod celestial_body;
mod components;
mod systems;
mod builders;
mod events;

use std::time::Duration;

use components::*;
use interpolation::EaseFunction;
use systems::*;

use bevy::{prelude::{Plugin, App, IntoSystemConfigs, OnEnter, OnExit, Color, Update, KeyCode, PostUpdate, Button, EventWriter, Res, Query, Entity, With, Commands, Name, NodeBundle, BuildChildren, ImageBundle, default, Visibility, TextBundle, Transform, Quat, Vec3, Camera2dBundle, Camera2d, State, ResMut, NextState, EventReader, Component}, input::common_conditions::input_just_pressed, app::AppExit, text::{TextStyle, Text, TextSection}, ui::{Style, PositionType, AlignSelf, Val, UiRect, FlexDirection, UiImage}, core_pipeline::clear_color::ClearColorConfig};
use crate::{
    common::{state::{GameState, MenuState, SettingsMenuState}, conditions::on_click, systems::{animate_button_scale, play_sound_on_hover, send_event, despawn_with, set_state}, lens::TransformLens},
    parallax::{parallax_animation_system, ParallaxSet},
    language::LanguageContent,
    animation::{Animator, RepeatCount, Tween, RepeatStrategy}, 
    plugins::{slider::Slider, assets::{FontAssets, UiAssets}, camera::components::MainCamera, audio::{PlaySoundEvent, SoundType, PlayMusicEvent, MusicType, MusicAudio}, MenuSystemSet}
};
use self::{settings::SettingsMenuPlugin, celestial_body::CelestialBodyPlugin, builders::{menu, menu_button}, events::{Back, EnterMenu}};

use super::FpsText;

pub(super) const TEXT_COLOR: Color = Color::rgb(0.58, 0.58, 0.58);
pub(super) const MENU_BUTTON_FONT_SIZE: f32 = 42.;

#[derive(Component)]
pub(super) struct DespawnOnMenuExit;

#[derive(Component)]
pub(super) struct BackButton;

#[derive(Component)]
pub(super) struct ApplyButton;

pub(super) struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Back>();
        app.add_event::<EnterMenu>();

        app.add_plugins((CelestialBodyPlugin, SettingsMenuPlugin));

        app.add_systems(
            OnEnter(GameState::Menu),
            (
                setup_camera,
                spawn_menu_container,
                play_menu_music,
                set_state(MenuState::Main)
            )
        );

        app.add_systems(OnEnter(MenuState::Main), setup_main_menu);
        app.add_systems(OnExit(MenuState::Main), despawn_with::<Menu>);

        app.add_systems(
            OnExit(GameState::Menu),
            (
                despawn_with::<DespawnOnMenuExit>,
                despawn_with::<MusicAudio>
            )
        );

        app.add_systems(
            Update,
            (
                send_event(Back).run_if(on_click::<BackButton>),
                send_event(Back).run_if(input_just_pressed(KeyCode::Escape)),
            )
        );

        app.add_systems(
            PostUpdate,
            (
                handle_back_event,
                handle_enter_menu_event,
            )
            .in_set(MenuSystemSet::PostUpdate)
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
            .in_set(MenuSystemSet::Update)
        );

        app.add_systems(
            Update,
            (
                (
                    set_state(MenuState::None),
                    set_state(GameState::WorldLoading),
                )
                .chain()
                .run_if(on_click::<SinglePlayerButton>),

                send_event(EnterMenu(MenuState::Settings(SettingsMenuState::Main))).run_if(on_click::<SettingsButton>),
                send_event(AppExit).run_if(on_click::<ExitButton>),
            )
            .in_set(MenuSystemSet::PostUpdate)
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
                scale: Vec3::splat(0.9),
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
                    padding: UiRect::vertical(Val::Px(20.)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(80.),
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
                        align_self: AlignSelf::Center,
                        width: Val::Px(600.),
                        height: Val::Px(250.),
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
        font_size: 60.,
        color: TEXT_COLOR,
    };

    let container = query_container.single();

    menu(Menu, &mut commands, container, 70., |builder| {
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

fn play_menu_music(
    mut play_music: EventWriter<PlayMusicEvent>
) {
    play_music.send(PlayMusicEvent(MusicType::TitleScreen));
}

fn handle_back_event(
    state: Res<State<MenuState>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut play_sound: EventWriter<PlaySoundEvent>,
    mut back_events: EventReader<Back>
) {
    if back_events.iter().last().is_some() {
        next_state.set(state.back());
        play_sound.send(PlaySoundEvent(SoundType::MenuClose));
    }
}

fn handle_enter_menu_event(
    mut next_state: ResMut<NextState<MenuState>>,
    mut play_sound: EventWriter<PlaySoundEvent>,
    mut enter_events: EventReader<EnterMenu>
) {
    if let Some(event) = enter_events.iter().last() {
        next_state.set(event.0);
        play_sound.send(PlaySoundEvent(SoundType::MenuOpen));
    }
}