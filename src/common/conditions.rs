use bevy::{prelude::{State, Res, Component, Changed, With, Button, Query}, ui::Interaction};

use super::state::GameState;

pub fn in_menu_state(state: Res<State<GameState>>) -> bool {
    matches!(&state.0, GameState::Menu(_))
}

pub fn on_btn_clicked<B: Component>(
    query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<B>)>,
) -> bool {
    for interaction in query.iter() {
        if *interaction == Interaction::Clicked {
            return true;
        }
    }

    false
}