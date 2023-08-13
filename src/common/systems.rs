use bevy::{ui::Interaction, prelude::{Changed, Component, Query, With, EventWriter, Visibility, Resource, Res, DetectChanges}, text::Text};

use crate::{animation::{Animator, TweeningDirection, Tween, Tweenable}, plugins::audio::{PlaySoundEvent, SoundType}};

use super::{helpers, IsVisible};

pub(crate) fn animate_button_scale<B: Component>(
    mut query: Query<
        (&Interaction, &mut Animator<Text>),
        (With<B>, Changed<Interaction>),
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
            },
            _ => {}
        }
    }
}

pub(crate) fn play_sound_on_button_hover<B: Component>(
    mut query: Query<&Interaction, (With<B>, Changed<Interaction>)>,
    mut play_sound: EventWriter<PlaySoundEvent>
) {
    for interaction in query.iter_mut() {
        if let Interaction::Hovered = interaction {
            play_sound.send(PlaySoundEvent(SoundType::MenuTick));
        }
    }
}

pub(crate) fn set_visibility<C: Component, R: IsVisible + Resource>(
    mut query_visibility: Query<&mut Visibility, With<C>>,
    res_visibility: Res<R>
) {
    if res_visibility.is_changed() {
        for mut visibility in &mut query_visibility {
            helpers::set_visibility(&mut visibility, res_visibility.is_visible());
        }
    }
}