pub(crate) mod resources;
pub(crate) mod systems;
pub(crate) mod components;
pub(crate) mod ingame;
pub(crate) mod menu;

use bevy::{prelude::{Plugin, App, KeyCode, Update, IntoSystemConfigs, OnExit, Commands, Res, NodeBundle, default, Name, BuildChildren, Visibility, Color, TextBundle, Condition, Button, not, Component, OnEnter, PostUpdate, Resource, resource_equals}, input::common_conditions::input_just_pressed, ui::{Style, Val, FlexDirection, JustifyContent, AlignItems, UiRect, PositionType}, text::{TextAlignment, Text, TextStyle, TextSection}};
use crate::common::{state::GameState, systems::{bind_visibility_to, despawn_with, toggle_resource, animate_button_scale, play_sound}, conditions::{on_click, is_visible}};

use self::{
    components::{MainUiContainer, MusicVolumeSliderOutput, SoundVolumeSliderOutput, MusicVolumeSlider, SoundVolumeSlider},
    ingame::{inventory::{systems::spawn_inventory_ui, InventoryUiPlugin, components::InventoryUi}, settings::{systems::spawn_ingame_settings_button, InGameSettingsUiPlugin}},
    menu::MenuPlugin, systems::{play_sound_on_hover, update_previous_interaction}, resources::{IsVisible, SettingsMenu, Ui, Cursor},
};

use crate::plugins::assets::{FontAssets, UiAssets};

use super::{InGameSystemSet, DespawnOnGameExit, slider::Slider, audio::SoundType, world_map_view::MapViewStatus};

#[derive(Component)]
pub(crate) struct FpsText;

#[derive(Resource, Default)]
pub(crate) struct MouseOverUi(pub(crate) bool);

pub(crate) struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InventoryUiPlugin, InGameSettingsUiPlugin, MenuPlugin));

        app.insert_resource(IsVisible::<Ui>::visible());
        app.init_resource::<MouseOverUi>();

        app.add_systems(OnEnter(GameState::InGame), setup);
        app.add_systems(OnExit(GameState::InGame), cleanup);

        app.add_systems(OnExit(GameState::WorldLoading), spawn_ui_container);
        app.add_systems(OnExit(GameState::WorldLoading), spawn_fps_text);
        app.add_systems(OnExit(GameState::InGame), despawn_with::<MainUiContainer>);

        app.add_systems(
            Update,
            (
                play_sound_on_hover::<Button>,
                play_sound(SoundType::MenuTick).run_if(on_click::<Button>),
                play_sound_on_hover::<Slider>,
            )
        );

        app.add_systems(
            Update,
            (
                (
                    toggle_resource::<IsVisible<InventoryUi>>,
                    systems::play_sound_on_toggle::<IsVisible<InventoryUi>>
                )
                .chain()
                .run_if(
                    not(is_visible::<SettingsMenu>).and_then(input_just_pressed(KeyCode::Escape))
                ),
                (
                    toggle_resource::<IsVisible<Ui>>,
                    toggle_resource::<IsVisible<Cursor>>,
                )
                .run_if(input_just_pressed(KeyCode::F11)),
                bind_visibility_to::<Ui, MainUiContainer>,
            )
            .in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            Update,
            (
                animate_button_scale::<Button>,
                systems::animate_slider_border_color,
                systems::bind_slider_to_output::<MusicVolumeSlider, MusicVolumeSliderOutput>,
                systems::bind_slider_to_output::<SoundVolumeSlider, SoundVolumeSliderOutput>,
                systems::update_music_volume,
                systems::update_sound_volume,
            )
        );

        app.add_systems(PostUpdate, update_previous_interaction);

        app.add_systems(
            Update,
            systems::update_mouse_over_ui
                .run_if(resource_equals(MapViewStatus::Closed)),
        );
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(IsVisible::<InventoryUi>::hidden());
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<IsVisible::<InventoryUi>>();
}

fn spawn_ui_container(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    ui_assets: Res<UiAssets>,
) {
    let main_id = commands
        .spawn((
            Name::new("Main UI Container"),
            MainUiContainer,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                ..default()
            }
        ))
        .id();

    // Left container
    let left_id = commands
        .spawn((
            Name::new("Left UI Container"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    margin: UiRect::left(Val::Px(20.)),
                    ..default()
                },
                ..default()
            }
        ))
        .id();

    // Right container
    let right_id = commands
        .spawn((
            Name::new("Right UI Container"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexEnd,
                    padding: UiRect {
                        right: Val::Px(20.),
                        ..UiRect::vertical(Val::Px(5.))
                    },
                    ..default()
                },
                ..default()
            }
        ))
        .id();

    let inventory = spawn_inventory_ui(&mut commands, &ui_assets, &font_assets);
    let settings_btn = spawn_ingame_settings_button(&mut commands, &font_assets);

    // TODO: Make a health bar in the feature, stub for now
    let health_bar = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(10.),
                height: Val::Px(2.),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Stub"))
        .id();

    commands
        .entity(left_id)
        .push_children(&[inventory]);

    commands
        .entity(right_id)
        .push_children(&[health_bar, settings_btn]);

    commands.entity(main_id).push_children(&[left_id, right_id]);
}

pub(super) fn spawn_fps_text(mut commands: Commands, font_assets: Res<FontAssets>) {
    let text_style = TextStyle {
        font: font_assets.andy_regular.clone_weak(),
        font_size: 20.,
        color: Color::WHITE,
    };

    commands.spawn((
        FpsText,
        Name::new("FPS Text"),
        DespawnOnGameExit,
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(5.),
                bottom: Val::Px(0.),
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection::from_style(text_style)
                ],
                alignment: TextAlignment::Left,
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        }
    ));
}