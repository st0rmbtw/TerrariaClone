use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Plugin, App, Commands, Query, Changed, Entity, Component, Color, With, Button, Name, TextBundle, ParallelSystemDescriptorCoercion, NodeBundle, BuildChildren}, text::{Text, TextStyle, TextAlignment}, ui::{Interaction, Style, JustifyContent, AlignItems, UiRect, Val, Size}};
use interpolation::EaseFunction;
use iyes_loopless::prelude::*;

use crate::{animation::{Animator, TweeningDirection, Tween, TweeningType, component_animator_system, AnimationSystem}, state::GameState, lens::TextFontSizeLens, TRANSPARENT};

use super::FontAssets;

// region: Plugin
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(update)
                    .into()
            )
            .add_system(component_animator_system::<Text>.label(AnimationSystem::AnimationUpdate));
    }
}
// endregion

#[derive(Component)]
struct SettingsButton;

#[autodefault(except(TextFontSizeLens))]
pub fn spawn_ingame_settings_button(
    commands: &mut Commands,
    fonts: &FontAssets
) -> Entity {
    let tween = Tween::new(
        EaseFunction::QuadraticInOut, 
        TweeningType::Once, 
        Duration::from_millis(150),
        TextFontSizeLens {
            start: 32.,
            end: 38.
        }
    );

    commands.spawn_bundle(NodeBundle {
        style: Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.)),
            size: Size { 
                width: Val::Px(100.),
                height: Val::Px(38.),
            }
        },
        color: TRANSPARENT.into()
    }).with_children(|c| {
        c.spawn_bundle(TextBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_shrink: 0.
            },
            text: Text::from_section(
                "Settings".to_string(), 
                TextStyle { 
                    font: fonts.andy_bold.clone(), 
                    font_size: 32., 
                    color: Color::WHITE
                }
            ).with_alignment(TextAlignment::CENTER)
        })
        .insert(Button)
        .insert(Name::new("Settings button"))
        .insert(Interaction::default())
        .insert(SettingsButton)
        .insert(Animator::new(tween));
    })
    .id()
}

fn update(
    mut query: Query<(&Interaction, &mut Animator<Text>), (With<Button>, With<Text>, Changed<Interaction>)>
) {
    for (interaction, mut animator) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                animator.start();

                let tweenable = animator.tweenable_mut();
                tweenable.set_progress(1. - tweenable.progress());
                tweenable.set_direction(TweeningDirection::Forward);
            },
            Interaction::None => {
                let tweenable = animator.tweenable_mut();
                tweenable.set_progress(1. - tweenable.progress());
                tweenable.set_direction(TweeningDirection::Backward);
            }
            _ => {}
        }
    }
}