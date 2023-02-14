use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Query, Visibility, With, EventReader, Button, Name, Color, TextBundle, Entity, Commands, NodeBundle, BuildChildren, Changed}, ui::{Interaction, Style, UiRect, Val, AlignItems, JustifyContent, Size}, text::{TextAlignment, TextStyle, Text}};
use interpolation::EaseFunction;

use crate::{plugins::{ui::ToggleExtraUiEvent, assets::FontAssets}, animation::{Animator, Tween, TweeningDirection, RepeatStrategy, Tweenable}, lens::TextFontSizeLens, language::LanguageContent};

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
            visibility: Visibility { is_visible: false },
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

pub fn set_btn_visibility(
    mut query: Query<&mut Visibility, With<SettingsButtonContainer>>,
    mut events: EventReader<ToggleExtraUiEvent>,
) {
    for event in events.iter() {
        for mut visibility in &mut query {
            visibility.is_visible = event.0;
        }
    }
}

pub fn update(
    mut query: Query<
        (&Interaction, &mut Animator<Text>),
        (With<Button>, With<Text>, Changed<Interaction>),
    >,
) {
    for (interaction, mut animator) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
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