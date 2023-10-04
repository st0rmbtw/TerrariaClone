use bevy::{prelude::{Component, Changed, With, Query, Res}, ui::Interaction};

use crate::plugins::ui::MouseOverUi;

pub(crate) fn on_click<B: Component>(
    query: Query<&Interaction, (Changed<Interaction>, With<B>)>,
) -> bool {
    let Ok(interaction) = query.get_single() else { 
        return false;
    };

    matches!(interaction, Interaction::Pressed)
}

pub(crate) fn mouse_over_ui(
    mouse_over_ui: Option<Res<MouseOverUi>>
) -> bool {
    mouse_over_ui.is_some_and(|res| res.0)
}