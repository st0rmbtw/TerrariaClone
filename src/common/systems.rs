use bevy::{ui::{Interaction, Style, Display}, prelude::{Changed, Component, Query, With, EventWriter, Visibility, Resource, Res, DetectChanges, Event, Entity, Commands, DespawnRecursiveExt, States, ResMut, NextState, Color}, text::Text};

use crate::{animation::{Animator, TweeningDirection, Tween, Tweenable}, plugins::{audio::{SoundType, AudioCommandsExt}, ui::resources::{VisibilityMarker, IsVisible}}};

use super::{helpers, BoolValue, Toggle};

pub(crate) fn bind_visibility_to<M: VisibilityMarker, C: Component>(
    mut query_visibility: Query<&mut Visibility, With<C>>,
    opt_res_visibility: Option<Res<IsVisible<M>>>
) {
    let Some(res_visibility) = opt_res_visibility else { return; };

    if res_visibility.is_changed() {
        query_visibility.for_each_mut(|visibility| {
            helpers::set_visibility(visibility, res_visibility.value());
        });
    }
}

pub(crate) fn bind_not_visibility_to<M: VisibilityMarker, C: Component>(
    mut query_visibility: Query<&mut Visibility, With<C>>,
    opt_visible: Option<Res<IsVisible<M>>>
) {
    let Some(visible) = opt_visible else { return; };

    if visible.is_changed() {
        query_visibility.for_each_mut(|visibility| {
            helpers::set_visibility(visibility, !*visible);
        });
    }
}

pub(crate) fn bind_visibility2_to<M1: VisibilityMarker, M2: VisibilityMarker, C: Component>(
    mut query_visibility: Query<&mut Visibility, With<C>>,
    opt_visible1: Option<Res<IsVisible<M1>>>,
    opt_visible2: Option<Res<IsVisible<M2>>>
) {
    let Some(visible1) = opt_visible1 else { return; };
    let Some(visible2) = opt_visible2 else { return; };

    if visible1.is_changed() || visible2.is_changed() {
        query_visibility.for_each_mut(|visibility| {
            helpers::set_visibility(visibility, **visible1 && **visible2);
        });
    }
}

pub(crate) fn set_display<C: Component, R: BoolValue + Resource>(
    mut query_visibility: Query<&mut Style, With<C>>,
    opt_res_visibility: Option<Res<R>>
) {
    let Some(res_visibility) = opt_res_visibility else { return; };

    if res_visibility.is_changed() {
        query_visibility.for_each_mut(|mut style| {
            if res_visibility.value() {
                style.display = Display::Flex;
            } else {
                style.display = Display::None;
            }
        });
    }
}

pub(crate) fn toggle_visibility<C: Component>(
    mut query: Query<&mut Visibility, With<C>>
) {
    for mut visibility in &mut query {
        *visibility = match *visibility {
            Visibility::Inherited | Visibility::Visible => Visibility::Hidden,
            Visibility::Hidden => Visibility::Inherited,
        };
    }
}

pub(crate) fn send_event<E: Event + Clone>(event: E) -> impl FnMut(EventWriter<E>) {
    move |mut events: EventWriter<E>| {
        events.send(event.clone());
    }
}

pub(crate) fn despawn_with<C: Component>(mut commands: Commands, query: Query<Entity, With<C>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub(crate) fn component_equals<M: Component, C: Component + PartialEq>(component: C) -> impl Fn(Query<&C, With<M>>) -> bool {
    move |query: Query<&C, With<M>>| -> bool {
        let Ok(comp) = query.get_single() else { return false; };
        *comp == component
    }
}

pub(crate) fn set_state<S: States + Clone>(state: S) -> impl FnMut(ResMut<NextState<S>>) {
    move |mut next_state: ResMut<NextState<S>>| {
        next_state.set(state.clone());
    }
}

pub(crate) fn toggle_resource<T: Toggle + Resource>(res: Option<ResMut<T>>) {
    let Some(mut res) = res else { return; };
    res.toggle()
}

pub(crate) fn set_resource<R: Resource + Clone>(res: R) -> impl FnMut(Commands) {
    move |mut commands: Commands| {
        commands.insert_resource(res.clone());
    }
}

pub(crate) fn animate_button_scale<B: Component>(
    mut query: Query<
        (&Interaction, &mut Animator<Text>),
        (With<B>, Changed<Interaction>),
    >,
) {
    for (interaction, mut animator) in query.iter_mut() {
        match interaction {
            Interaction::Hovered | Interaction::Pressed => {
                animator.start();

                let tweenable = animator.tweenable_mut().as_any_mut().downcast_mut::<Tween<Text>>().unwrap();
                if tweenable.direction() != TweeningDirection::Forward {
                    tweenable.set_progress(1. - tweenable.progress());
                    tweenable.set_direction(TweeningDirection::Forward);
                }    
            }
            Interaction::None => {
                let tweenable = animator.tweenable_mut().as_any_mut().downcast_mut::<Tween<Text>>().unwrap();
                if tweenable.direction() != TweeningDirection::Backward {
                    tweenable.set_progress(1. - tweenable.progress());
                    tweenable.set_direction(TweeningDirection::Backward);
                }
            }
        }
    }
}

pub(crate) fn animate_button_color<B: Component>(
    from: Color, to: Color
) -> impl FnMut(Query<(&Interaction, &mut Text), (With<B>, Changed<Interaction>)>) {
    move |mut query: Query<(&Interaction, &mut Text), (With<B>, Changed<Interaction>)>| {
        for (interaction, mut text) in query.iter_mut() {
            match interaction {
                Interaction::Hovered => {
                    text.sections[0].style.color = to;
                }
                Interaction::None => {
                    text.sections[0].style.color = from;
                },
                _ => {}
            }
        }
    }
}

pub(crate) fn play_sound(sound: SoundType) -> impl FnMut(Commands) {
    move |mut commands: Commands| {
        commands.play_sound(sound);
    }
}