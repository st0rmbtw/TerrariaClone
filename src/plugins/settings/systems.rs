use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Query, Visibility, With, EventReader, Name, Color, TextBundle, Entity, Commands, NodeBundle, BuildChildren, Changed, Res, Audio}, ui::{Interaction, Style, UiRect, Val, AlignItems, JustifyContent, Size}, text::{TextAlignment, TextStyle, Text}};
use interpolation::EaseFunction;

use crate::{plugins::{ui::ToggleExtraUiEvent, assets::{FontAssets, SoundAssets}}, animation::{Animator, Tween, TweeningDirection, RepeatStrategy, Tweenable}, language::LanguageContent, common::{lens::TextFontSizeLens, helpers}};

use super::{SettingsButtonContainer, SettingsButtonText};

#[autodefault(except(TextFontSizeLens))]
pub fn spawn_ingame_settings_button(
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
                padding: UiRect::all(Val::Px(10.)),
                size: Size {
                    width: Val::Px(100.),
                    height: Val::Px(38.),
                },
            },
            visibility: Visibility::Hidden,
        })
        .insert(SettingsButtonContainer)
        .with_children(|c| {
            c.spawn(TextBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_shrink: 0.,
                },
                text: Text::from_section(
                    language_content.ui.settings.clone(),
                    TextStyle {
                        font: fonts.andy_bold.clone_weak(),
                        font_size: 32.,
                        color: Color::WHITE,
                    },
                )
                .with_alignment(TextAlignment::Center),
            })
            .insert(Name::new("Settings button"))
            .insert(Interaction::default())
            .insert(SettingsButtonText)
            .insert(Animator::new(tween));
        })
        .id()
}

pub(super) fn set_btn_visibility(
    mut query: Query<&mut Visibility, With<SettingsButtonContainer>>,
    mut events: EventReader<ToggleExtraUiEvent>,
) {
    for event in events.iter() {
        for visibility in &mut query {
            helpers::set_visibility(visibility, event.0);
        }
    }
}

pub(super) fn update(
    mut query: Query<
        (&Interaction, &mut Animator<Text>),
        (With<SettingsButtonText>, Changed<Interaction>),
    >,
    audio: Res<Audio>,
    sounds: Res<SoundAssets>
) {
    for (interaction, mut animator) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                audio.play(sounds.menu_tick.clone_weak());

                animator.start();

                let tweenable = animator.tweenable_mut().as_any_mut().downcast_mut::<Tween<Text>>().unwrap();
                tweenable.set_progress(1. - tweenable.progress());
                tweenable.set_direction(TweeningDirection::Forward);
            }
            Interaction::None => {
                let tweenable = animator.tweenable_mut().as_any_mut().downcast_mut::<Tween<Text>>().unwrap();
                tweenable.set_progress(1. - tweenable.progress());
                tweenable.set_direction(TweeningDirection::Backward);
            }
            _ => {}
        }
    }
}