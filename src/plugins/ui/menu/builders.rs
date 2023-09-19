use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Name, NodeBundle, TextBundle, ChildBuilder, Component, default, Color, ImageBundle, BuildChildren, Commands, Entity, Bundle}, ui::{Style, JustifyContent, AlignItems, FocusPolicy, PositionType, Interaction, Val, FlexDirection, UiRect}, text::{Text, TextStyle, TextSection, TextAlignment}};

use crate::{animation::{AnimatorState, Animator, Tween, EaseMethod, RepeatStrategy}, plugins::{slider::{SliderHandleBundle, SliderBundle, Slider}, assets::UiAssets, ui::components::PreviousInteraction}, common::lens::TextFontSizeLens, language::LocalizedText};

use super::{MENU_BUTTON_FONT_SIZE, components::Menu};

pub(crate) fn menu(marker: impl Component, commands: &mut Commands, container: Entity, gap: f32, spawn_children: impl FnOnce(&mut ChildBuilder)) {
    let menu = commands.spawn((
        Name::new("Menu"),
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(gap),
                ..default()
            },
            ..default()
        },
        Menu
    ))
    .insert(marker)
    .with_children(spawn_children)
    .id();

    commands.entity(container)
        .add_child(menu);
}

#[autodefault]
pub(crate) fn menu_button(
    builder: &mut ChildBuilder,
    text_style: TextStyle,
    text: impl Into<LocalizedText>,
    bundle: impl Bundle,
) {
    builder
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                height: Val::Px(text_style.font_size),
            },
            focus_policy: FocusPolicy::Pass
        })
        .with_children(|b| {
            b.spawn((
                Interaction::default(),
                PreviousInteraction::default(),
                Animator::new(text_tween(text_style.font_size)).with_state(AnimatorState::Paused),
                bundle,
                TextBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                    },
                    text: Text::from_section("", text_style.clone()).with_no_wrap(),
                },
                text.into()
            ));
        });
}

#[inline(always)]
#[autodefault]
pub(crate) fn menu_text(builder: &mut ChildBuilder, text_style: TextStyle, text: impl Into<String>) {
    builder.spawn((
        Name::new("MenuText"),
        TextBundle {
            text: Text::from_section(text, text_style.clone()).with_no_wrap(),
        },
    ));
}

#[inline(always)]
#[autodefault]
pub(crate) fn menu_text_localized(builder: &mut ChildBuilder, text_style: TextStyle, text: impl Into<LocalizedText>) {
    builder.spawn((
        Name::new("MenuText"),
        TextBundle {
            text: Text::from_section("", text_style.clone()).with_no_wrap(),
        },
        text.into()
    ));
}

#[autodefault]
#[inline(always)]
pub(crate) fn control_buttons_layout(
    builder: &mut ChildBuilder,
    spawn_builder: impl FnOnce(&mut ChildBuilder)
) {
    builder.spawn(NodeBundle {
        style: Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            margin: UiRect::vertical(Val::Px(40.)),
            row_gap: Val::Px(25.)
        },
        focus_policy: FocusPolicy::Pass
    }).with_children(spawn_builder);
}

#[inline(always)]
#[autodefault]
pub(crate) fn control_button(
    builder: &mut ChildBuilder,
    text_style: TextStyle,
    text: impl Into<LocalizedText>,
    bundle: impl Bundle
) {
    menu_button(builder, TextStyle { font_size: MENU_BUTTON_FONT_SIZE, ..text_style }, text, bundle);
}

pub(crate) fn slider_layout(
    builder: &mut ChildBuilder,
    gap: f32,
    first_column_align: AlignItems,
    first_column_builder: impl FnOnce(&mut ChildBuilder),
    second_column_align: AlignItems,
    second_column_builder: impl FnOnce(&mut ChildBuilder)
) {
    builder.spawn((
        Name::new("SliderLayout"),
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(gap),
                width: Val::Percent(100.),
                ..default()
            },
            ..default()
        }
    )).with_children(|b| {
        b.spawn((
            Name::new("FirstColumn"),
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: first_column_align,
                    justify_content: JustifyContent::Center,
                    height: Val::Percent(100.),
                    ..default()
                },
                ..default()
            }
        )).with_children(first_column_builder);

        b.spawn((
            Name::new("SecondColumn"),
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: second_column_align,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            }
        )).with_children(second_column_builder);
    });
}

#[autodefault]
pub(crate) fn menu_slider(
    builder: &mut ChildBuilder,
    ui_assets: &UiAssets,
    value: f32,
    background_color: Color,
    scale: f32,
    height: Val,
    slider_marker: impl Component,
) {
    builder.spawn((
        Name::new("SliderContainer"),
        NodeBundle {
            style: Style {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                height
            }
        }
    )).with_children(|b| {
        b.spawn((
            Name::new("SliderBackground"),
            ImageBundle {
                style: Style {
                    width: Val::Px(180. * scale),
                    height: Val::Px(16. * scale),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                },
                background_color: background_color.into(),
                image: ui_assets.slider_background.clone_weak().into(),
            }
        )).with_children(|b| {
            b.spawn((
                Name::new("SliderBorder"),
                SliderBundle {
                    style: Style {
                        width: Val::Px(180. * scale),
                        height: Val::Px(16. * scale),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                    },
                    image: ui_assets.slider_border.clone_weak().into(),
                    slider: Slider::new(0., 1.)
                        .with_step(0.01)
                        .with_value(value).unwrap(),
                }
            ))
            .insert(slider_marker)
            .with_children(|parent| {
                parent.spawn((
                    Name::new("SliderHandle"),
                    SliderHandleBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            width: Val::Px(12. * scale),
                            height: Val::Px(25. * scale),
                        },
                        image: ui_assets.slider_handle.clone_weak().into(),
                    }
                ));
            });
        });
    });
}

#[inline(always)]
pub(crate) fn slider_name_text(builder: &mut ChildBuilder, text_style: TextStyle, text: impl Into<LocalizedText>) {
    builder.spawn((
        Name::new("SliderNameText"),
        TextBundle {
            text: Text::from_sections([
                TextSection::new("", text_style.clone()),
                TextSection::new(":", text_style)
            ]).with_no_wrap(),
            ..default()
        },
        text.into()
    ));
}

#[inline(always)]
pub(crate) fn slider_value_text(builder: &mut ChildBuilder, text_style: TextStyle, value: f32, min_width: f32, output_marker: impl Component) {
    builder.spawn((
        Name::new("SliderValueText"),
        TextBundle {
            style: Style {
                min_width: Val::Px(min_width),
                ..default()
            },
            text: Text::from_sections([
                TextSection::new(value.to_string(), text_style.clone()),
                TextSection::new("%", text_style)
            ]).with_no_wrap().with_alignment(TextAlignment::Center),
            ..default()
        }
    ))
    .insert(output_marker);
}

#[inline(always)]
pub(crate) fn spacer(builder: &mut ChildBuilder, height: f32) {
    builder.spawn(NodeBundle {
        style: Style {
            height: Val::Px(height),
            ..default()
        },
        ..default()
    });
}

#[inline(always)]
fn text_tween(initial_font_size: f32) -> Tween<Text> {
    Tween::new(
        EaseMethod::Linear,
        RepeatStrategy::MirroredRepeat,
        Duration::from_millis(150),
        TextFontSizeLens {
            start: initial_font_size,
            end: initial_font_size * 1.2,
        },
    )
}