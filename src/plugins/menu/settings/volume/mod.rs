use bevy::{prelude::{Plugin, App, Commands, OnEnter, Query, Entity, With, Res, Color, OnExit, default, BuildChildren, ChildBuilder, NodeBundle, TextBundle, ImageBundle, Component}, text::{TextStyle, Text, TextAlignment}, ui::{Style, Val, FlexDirection, AlignItems, JustifyContent, PositionType}};

use crate::{common::state::{GameState, MenuState, SettingsMenuState}, plugins::{menu::{systems::{menu, menu_text, despawn_with, control_buttons_layout, control_button}, components::MenuContainer, MENU_BUTTON_FONT_SIZE, BackButton, TEXT_COLOR}, assets::{FontAssets, UiAssets}, slider::{SliderBundle, SliderHandleBundle, Slider}}, language::LanguageContent};

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
    }
}

#[derive(Component)]
struct VolumeMenu;

fn setup_volume_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    ui_assets: Res<UiAssets>,
    language_content: Res<LanguageContent>,
    query_container: Query<Entity, With<MenuContainer>>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: Color::WHITE,
    };

    let container = query_container.single();

    menu(VolumeMenu, &mut commands, container, 50., |builder| {
        menu_text(builder, text_style.clone(), language_content.ui.volume.to_string());

        menu_slider(builder, &ui_assets, "Music".to_string(), TextStyle { font_size: 36., ..text_style.clone() });

        control_buttons_layout(builder, |control_button_builder| {
            control_button(
                control_button_builder,
                TextStyle { color: TEXT_COLOR, ..text_style },
                language_content.ui.back.clone(),
                BackButton
            );
        });
    });
}

fn menu_slider(
    builder: &mut ChildBuilder,
    ui_assets: &UiAssets,
    name: String,
    text_style: TextStyle
) {
    builder.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.),
            column_gap: Val::Px(10.),
            ..default()
        },
        ..default()
    }).with_children(|b| {
        b.spawn(ImageBundle {
            style: Style {
                width: Val::Px(200.),
                height: Val::Px(20.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            image: ui_assets.slider_background.clone_weak().into(),
            ..default()
        }).with_children(|b| {
            b
                .spawn(SliderBundle {
                    style: Style {
                        width: Val::Px(200.),
                        height: Val::Px(20.),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    image: ui_assets.slider_border.clone_weak().into(),
                    slider: Slider::new(0., 100.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(SliderHandleBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            width: Val::Px(15.),
                            height: Val::Px(30.),
                            ..default()
                        },
                        image: ui_assets.slider_handle.clone_weak().into(),
                        ..default()
                    });
            });
        });

        b.spawn(TextBundle {
            text: Text::from_section(name, text_style)
                .with_no_wrap()
                .with_alignment(TextAlignment::Center),
            ..default()
        });
    });
}