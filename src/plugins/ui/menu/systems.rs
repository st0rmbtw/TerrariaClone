use bevy::{prelude::{Component, Query, With, Button, Changed, Color}, text::Text, ui::{Interaction, BackgroundColor}};

use crate::plugins::slider::Slider;
use super::TEXT_COLOR;

pub(super) fn animate_button_color(
    mut query: Query<(&Interaction, &mut Text), (With<Button>, Changed<Interaction>)>,
) {
    for (interaction, mut text) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                text.sections[0].style.color = Color::YELLOW;
            }
            Interaction::None => {
                text.sections[0].style.color = TEXT_COLOR;
            },
            _ => {}
        }
    }
}

pub(super) fn animate_slider_border_color(
    mut query: Query<(&Interaction, &mut BackgroundColor), (With<Slider>, Changed<Interaction>)>,
) {
    for (interaction, mut background_color) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                *background_color = Color::YELLOW.into();
            }
            Interaction::None => {
                *background_color = Color::WHITE.into();
            },
            _ => {}
        }
    }
}

pub(super) fn bind_slider_to_output<S: Component, O: Component>(
    query_slider: Query<&Slider, With<S>>,
    mut query_output: Query<&mut Text, With<O>>
) {
    let slider = query_slider.single();
    let mut text = query_output.single_mut();

    text.sections[0].value = format!("{:.0}", slider.value());
}