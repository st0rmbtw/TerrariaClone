use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Entity, NodeBundle, Visibility, default, TextBundle, Color, Name, Button, BuildChildren, Res, Query, With, Changed, ResMut, Ref, DetectChanges}, ui::{Style, JustifyContent, AlignItems, AlignSelf, UiRect, Val, Interaction, PositionType, FlexDirection, Display, BackgroundColor}, text::{Text, TextAlignment, TextStyle}};
use interpolation::EaseFunction;

use crate::{language::LanguageContent, plugins::{assets::{FontAssets, UiAssets}, DespawnOnGameExit, config::{MusicVolume, SoundVolume, ShowTileGrid}, ui::{menu::MENU_BUTTON_COLOR, components::{ZoomSlider, ZoomSliderOutput}}, slider::Slider, camera::resources::Zoom}, animation::{Tween, RepeatStrategy, Animator, Tweenable, TweeningDirection}, common::lens::TextFontSizeLens};

use super::{components::{MenuContainer, SettingsButton, SettingsButtonContainer, TabMenuContainer, TabButton, TabMenu}, menus::{tabs_menu, general_menu, interface_menu}, SelectedTab, TAB_BUTTON_TEXT_SIZE};

pub(crate) fn spawn_ingame_settings_button(
    commands: &mut Commands,
    fonts: &FontAssets,
    language_content: &LanguageContent
) -> Entity {
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        RepeatStrategy::MirroredRepeat,
        Duration::from_millis(150),
        TextFontSizeLens {
            start: 32.,
            end: 38.,
        },
    );

    commands
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::End,
                padding: UiRect::all(Val::Px(10.)),
                width: Val::Px(100.),
                height: Val::Px(38.),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(SettingsButtonContainer)
        .with_children(|c| {
            c.spawn(TextBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_shrink: 0.,
                    ..default()
                },
                text: Text::from_section(
                    &language_content.ui.settings,
                    TextStyle {
                        font: fonts.andy_bold.clone_weak(),
                        font_size: 32.,
                        color: Color::WHITE,
                    },
                ).with_alignment(TextAlignment::Center),
                ..default()
            })
            .insert(Name::new("SettingsButton"))
            .insert(Interaction::default())
            .insert(SettingsButton)
            .insert(Button)
            .insert(Animator::new(tween));
        })
        .id()
}

#[autodefault]
pub(super) fn spawn_settings_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    language_content: Res<LanguageContent>
) {
    let text_style = TextStyle {
        font: font_assets.andy_bold.clone_weak(),
        font_size: 22.,
        color: Color::WHITE
    };

    let menu_container_color = BackgroundColor(Color::rgb_u8(54, 53, 131));

    let tabs_container = commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            padding: UiRect::vertical(Val::Px(10.))
        },
        background_color: menu_container_color
    }).id();

    let tab_content_container = commands.spawn((
        TabMenuContainer,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                padding: UiRect::vertical(Val::Px(10.))
            },
            background_color: menu_container_color
        }
    ))
    .id();

    tabs_menu(&mut commands, &font_assets, &language_content, tabs_container);

    commands.spawn((
        MenuContainer,
        DespawnOnGameExit,
        NodeBundle {
            style: Style {
                width: Val::Px(706.),
                height: Val::Px(516.),
                position_type: PositionType::Absolute,
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Column,
                padding: UiRect::top(Val::Px(22.)),
                display: Display::None
            },
            background_color: Color::rgb_u8(22, 10, 62).with_a(0.9).into(),
        }
    )).with_children(|builder| {
        builder.spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::Center
            },
            text: Text::from_section(&language_content.ui.settings_menu, text_style),
        });

        builder.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                padding: UiRect::all(Val::Px(28.)),
                column_gap: Val::Px(30.)
            }
        })
        .add_child(tabs_container)
        .add_child(tab_content_container);
    });
}

pub(super) fn spawn_general_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    ui_assets: Res<UiAssets>,
    language_content: Res<LanguageContent>,
    music_volume: Res<MusicVolume>,
    sound_volume: Res<SoundVolume>,
    zoom: Res<Zoom>,
    query_tab_menu: Query<(), With<TabMenu>>,
    query_container: Query<Entity, With<TabMenuContainer>>
) {
    if !query_tab_menu.is_empty() { return; };

    let Ok(container) = query_container.get_single() else { return; };
    general_menu(&mut commands, container, &font_assets, &ui_assets, &language_content, &music_volume, &sound_volume, &zoom);
}

pub(super) fn spawn_interface_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    language_content: Res<LanguageContent>,
    show_tile_grid: Res<ShowTileGrid>,
    query_tab_menu: Query<(), With<TabMenu>>,
    query_container: Query<Entity, With<TabMenuContainer>>
) {
    if !query_tab_menu.is_empty() { return; };

    let Ok(container) = query_container.get_single() else { return; };
    interface_menu(&mut commands, container, &font_assets, &language_content, show_tile_grid.0);
}

pub(super) fn update_tab_buttons(
    selected_tab: Res<SelectedTab>,
    mut query_tab_buttons: Query<(&mut Text, &Interaction, &SelectedTab), With<TabButton>>
) {
    for (mut text, interaction, tab) in &mut query_tab_buttons {
        let style = &mut text.sections[0].style;
        if *selected_tab == *tab {
            style.color = Color::YELLOW;
            style.font_size = TAB_BUTTON_TEXT_SIZE * 1.2;
        } else {
            match interaction {
                Interaction::Pressed | Interaction::Hovered => {
                    style.color = Color::WHITE;
                },
                Interaction::None => {
                    style.color = MENU_BUTTON_COLOR;
                },
            }
        }
    }
}

pub(super) fn animate_button_scale(
    selected_tab: Res<SelectedTab>,
    mut query: Query<(Ref<Interaction>, &mut Animator<Text>, Option<&SelectedTab>), With<TabButton>>,
) {
    for (interaction, mut animator, opt_tab) in query.iter_mut() {
        if !interaction.is_changed() && !selected_tab.is_changed() { continue; }
        if opt_tab.is_some_and(|tab| *tab == *selected_tab) { continue; }

        match *interaction {
            Interaction::Hovered | Interaction::Pressed => {
                animator.start();

                let tweenable = animator.tweenable_mut().as_any_mut().downcast_mut::<Tween<Text>>().unwrap();
                if tweenable.direction() != TweeningDirection::Forward {
                    tweenable.set_progress(0.);
                    tweenable.set_direction(TweeningDirection::Forward);
                }
            }
            Interaction::None => {
                let tweenable = animator.tweenable_mut().as_any_mut().downcast_mut::<Tween<Text>>().unwrap();
                if tweenable.direction() != TweeningDirection::Backward {
                    tweenable.set_progress(0.);
                    tweenable.set_direction(TweeningDirection::Backward);
                }
            }
        }
    }
}

pub(super) fn bind_zoom_slider_to_output(
    query_slider: Query<&Slider, (With<ZoomSlider>, Changed<Slider>)>,
    mut query_output: Query<&mut Text, With<ZoomSliderOutput>>
) {
    let Ok(slider) = query_slider.get_single() else { return; };
    let Ok(mut text) = query_output.get_single_mut() else { return; };

    text.sections[0].value = format!("{:.0}", (slider.value() + 1.) * 100.);
}

pub(super) fn update_zoom(
    mut zoom: ResMut<Zoom>,
    query_slider: Query<&Slider, (With<ZoomSlider>, Changed<Slider>)>,
) {
    if let Ok(slider) = query_slider.get_single() {
        zoom.set(slider.value());
    }
}