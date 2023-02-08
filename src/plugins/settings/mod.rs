use bevy::{prelude::{Plugin, App, IntoSystemDescriptor}, text::Text};
use iyes_loopless::prelude::ConditionSet;

use crate::{state::GameState, animation::{component_animator_system, AnimationSystem}};

pub use components::*;
pub use systems::*;

mod components;
mod systems;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(update)
                .with_system(set_btn_visibility)
                .into(),
        )
        .add_system(component_animator_system::<Text>.label(AnimationSystem::AnimationUpdate));
    }
}