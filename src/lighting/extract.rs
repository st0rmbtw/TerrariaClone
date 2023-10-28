use bevy::{prelude::{Commands, State, Res, DetectChanges, ResMut, Resource}, render::Extract};

use crate::{common::state::GameState, plugins::config::LightSmoothness};

use super::lightmap::assets::BlurArea;

pub(super) fn extract_state(
    mut commands: Commands,
    state: Extract<Res<State<GameState>>>,
) {
    commands.insert_resource(State::new(*state.get()));
}

pub(super) fn extract_light_smoothness(
    mut light_smoothness: ResMut<LightSmoothness>,
    extracted_light_smoothness: Extract<Res<LightSmoothness>>,
) {
    if extracted_light_smoothness.is_changed() {
        *light_smoothness = **extracted_light_smoothness;
    }
}

pub(super) fn extract_blur_area(
    mut blur_area: ResMut<BlurArea>,
    extracted_blur_area: Extract<Res<BlurArea>>,
) {
    if extracted_blur_area.is_changed() {
        *blur_area = **extracted_blur_area;
    }
}

pub(super) fn extract_resource<T: Resource + Clone>(
    mut commands: Commands,
    res: Extract<Option<Res<T>>>
) {
    let Some(resource) = res.as_ref() else { return; };

    if resource.is_changed() {
        commands.insert_resource((**resource).clone());
    }
}