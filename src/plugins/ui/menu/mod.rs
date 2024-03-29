mod settings;
mod components;
pub(super) mod builders;
mod events;

use std::time::Duration;

use components::*;
use interpolation::EaseFunction;

use bevy::{prelude::{Plugin, App, IntoSystemConfigs, OnEnter, OnExit, Color, Update, KeyCode, PostUpdate, Res, Query, Entity, With, Commands, Name, NodeBundle, BuildChildren, ImageBundle, default, Visibility, TextBundle, Transform, Quat, Vec3, Camera2dBundle, Camera2d, State, ResMut, NextState, EventReader, Component, FixedUpdate}, input::common_conditions::input_just_pressed, app::AppExit, text::{TextStyle, Text, TextSection}, ui::{Style, PositionType, AlignSelf, Val, UiRect, FlexDirection, UiImage}, core_pipeline::clear_color::ClearColorConfig, time::common_conditions::on_fixed_timer, ecs::query::Has};
use crate::{
    common::{state::{GameState, MenuState, SettingsMenuState}, conditions::on_click, systems::{send_event, despawn_with, set_state, animate_button_scale, animate_button_color}, lens::TransformLens},
    parallax::{parallax_animation_system, ParallaxSet},
    animation::{Animator, RepeatCount, Tween, RepeatStrategy}, 
    plugins::{assets::{FontAssets, UiAssets}, camera::components::MainCamera, audio::{SoundType, AudioCommandsExt}, MenuSystemSet, world::time::GameTime, background::sun_and_moon::SunAndMoon}, language::keys::UIStringKey
};
use self::{settings::SettingsMenuPlugin, builders::{menu, menu_button}, events::{Back, EnterMenu}};

use super::{FpsText, systems::play_sound_on_hover};

pub(crate) const MENU_BUTTON_COLOR: Color = Color::rgb(0.58, 0.58, 0.58);
pub(super) const MENU_BUTTON_FONT_SIZE: f32 = 42.;

#[derive(Component)]
pub(super) struct DespawnOnMenuExit;

#[derive(Component)]
pub(super) struct BackButton;

#[derive(Component)]
pub(super) struct ApplyButton;

#[derive(Component)]
struct Dragging;

pub(super) struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Back>();
        app.add_event::<EnterMenu>();

        app.add_plugins(SettingsMenuPlugin);

        app.add_systems(
            OnEnter(GameState::Menu),
            (
                setup_camera,
                spawn_menu_container,
                set_state(MenuState::Main)
            )
        );

        app.add_systems(OnEnter(MenuState::Main), setup_main_menu);
        app.add_systems(OnExit(MenuState::Main), despawn_with::<Menu>);

        app.add_systems(OnExit(GameState::Menu), despawn_with::<DespawnOnMenuExit>);

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
                play_sound_on_hover::<MenuButton>,
                animate_button_scale::<MenuButton>,
                animate_button_color::<MenuButton>(MENU_BUTTON_COLOR, Color::YELLOW),
                parallax_animation_system(150.).in_set(ParallaxSet::FollowCamera),
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
                .run_if(on_click::<SinglePlayerButton>),

                send_event(EnterMenu(MenuState::Settings(SettingsMenuState::Main))).run_if(on_click::<SettingsButton>),
                send_event(AppExit).run_if(on_click::<ExitButton>),
            )
            .in_set(MenuSystemSet::PostUpdate)
        );

        app.add_systems(
            FixedUpdate,
            update_game_time
                .in_set(MenuSystemSet::FixedUpdate)
                .run_if(on_fixed_timer(Duration::from_millis(1)))
        );
    }
}

fn update_game_time(
    mut game_time: ResMut<GameTime>,
    query_celestial_body: Query<Has<Dragging>, With<SunAndMoon>>
) {
    if game_time.paused { return; }
    if query_celestial_body.get_single().is_ok_and(|dragging| dragging) { return; }

    game_time.tick();
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
    query_container: Query<Entity, With<MenuContainer>>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: 60.,
        color: MENU_BUTTON_COLOR,
    };

    let container = query_container.single();

    menu(Menu, &mut commands, container, 10., |builder| {
        menu_button(
            builder,
            text_style.clone(),
            UIStringKey::SinglePlayer,
            (MenuButton, SinglePlayerButton),
        );
        menu_button(
            builder, 
            text_style.clone(),
            UIStringKey::Settings,
            (MenuButton, SettingsButton),
        );
        menu_button(
            builder, 
            text_style,
            UIStringKey::Exit,
            (MenuButton, ExitButton),
        );
    });
}

fn handle_back_event(
    mut commands: Commands,
    state: Res<State<MenuState>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut back_events: EventReader<Back>
) {
    if !back_events.is_empty() {
        back_events.clear();
        if *state.get() != state.back() {
            next_state.set(state.back());
            commands.play_sound(SoundType::MenuClose);
        }
    }
}

fn handle_enter_menu_event(
    mut commands: Commands,
    mut next_state: ResMut<NextState<MenuState>>,
    mut enter_events: EventReader<EnterMenu>
) {
    if let Some(event) = enter_events.iter().last() {
        next_state.set(event.0);
        commands.play_sound(SoundType::MenuOpen);
    }
}