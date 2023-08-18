use bevy::{prelude::{Component, Changed, With, Button, Query}, ui::Interaction};

pub(crate) fn on_click<B: Component>(
    query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<B>)>,
) -> bool {
    let Ok(interaction) = query.get_single() else { 
        return false;
    };

    matches!(interaction, Interaction::Pressed)
}