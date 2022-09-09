use std::time::Duration;

use autodefault::autodefault;
use bevy::{
    prelude::{
        App, BuildChildren, Button, Changed, Color, Commands, Component, Entity, EventReader, Name,
        NodeBundle, ParallelSystemDescriptorCoercion, Plugin, Query, TextBundle, Visibility, With,
    },
    text::{Text, TextAlignment, TextStyle},
    ui::{AlignItems, Interaction, JustifyContent, Size, Style, UiRect, Val},
};
use interpolation::EaseFunction;
use iyes_loopless::prelude::*;

use crate::{
    animation::{
        component_animator_system, AnimationSystem, Animator, Tween, TweeningDirection,
        TweeningType,
    },
    lens::TextFontSizeLens,
    state::GameState,
    TRANSPARENT,
};

use super::{FontAssets, ToggleExtraUiEvent};

// region: Plugin
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(update)
                .with_system(set_btn_visibility)
                .into(),
        )
        .add_system(component_animator_system::<Text>.label(AnimationSystem::AnimationUpdate));
    }
}
// endregion

#[derive(Component)]
struct SettingsButtonContainer;

#[derive(Component)]
struct SettingsButtonText;

#[autodefault(except(TextFontSizeLens))]
pub fn spawn_ingame_settings_button(commands: &mut Commands, fonts: &FontAssets) -> Entity {
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        TweeningType::Once,
        Duration::from_millis(150),
        TextFontSizeLens {
            start: 32.,
            end: 38.,
        },
    );

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.)),
                size: Size {
                    width: Val::Px(100.),
                    height: Val::Px(38.),
                },
            },
            visibility: Visibility { is_visible: false },
            color: TRANSPARENT.into(),
        })
        .insert(SettingsButtonContainer)
        .with_children(|c| {
            c.spawn_bundle(TextBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_shrink: 0.,
                },
                text: Text::from_section(
                    "Settings".to_string(),
                    TextStyle {
                        font: fonts.andy_bold.clone(),
                        font_size: 32.,
                        color: Color::WHITE,
                    },
                )
                .with_alignment(TextAlignment::CENTER),
            })
            .insert(Button)
            .insert(Name::new("Settings button"))
            .insert(Interaction::default())
            .insert(SettingsButtonText)
            .insert(Animator::new(tween));
        })
        .id()
}

fn set_btn_visibility(
    mut query: Query<&mut Visibility, With<SettingsButtonContainer>>,
    mut events: EventReader<ToggleExtraUiEvent>,
) {
    for event in events.iter() {
        for mut visibility in &mut query {
            visibility.is_visible = event.0;
        }
    }
}

fn update(
    mut query: Query<
        (&Interaction, &mut Animator<Text>),
        (With<Button>, With<Text>, Changed<Interaction>),
    >,
) {
    for (interaction, mut animator) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                animator.start();

                let tweenable = animator.tweenable_mut();
                tweenable.set_progress(1. - tweenable.progress());
                tweenable.set_direction(TweeningDirection::Forward);
            }
            Interaction::None => {
                let tweenable = animator.tweenable_mut();
                tweenable.set_progress(1. - tweenable.progress());
                tweenable.set_direction(TweeningDirection::Backward);
            }
            _ => {}
        }
    }
}
