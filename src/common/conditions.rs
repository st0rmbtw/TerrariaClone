use bevy::{prelude::{State, Res, Component, Changed, With, Button, Query}, ui::Interaction};

use super::state::GameState;

pub(crate) fn in_menu_state(state: Res<State<GameState>>) -> bool {
    matches!(state.get(), GameState::Menu(_))
}

pub(crate) fn on_btn_clicked<B: Component>(
    query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<B>)>,
) -> bool {
    let Ok(interaction) = query.get_single() else { 
        return false;
    };

    matches!(interaction, Interaction::Pressed)
}