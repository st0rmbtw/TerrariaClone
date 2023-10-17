use bevy::{prelude::{Commands, State, Res, DetectChanges, ResMut}, render::Extract};

use crate::{common::state::GameState, plugins::{config::LightSmoothness, world::{resources::WorldUndergroundLevel, WorldSize}}};

use super::{BackgroundTexture, InGameBackgroundTexture, WorldTexture, TileTexture, LightMapTexture, lightmap::assets::BlurArea};

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

pub(super) fn extract_world_underground_level(
    mut commands: Commands,
    underground_level: Extract<Option<Res<WorldUndergroundLevel>>>,
) {
    let Some(underground_level) = underground_level.as_ref() else { return; };

    if underground_level.is_changed() {
        commands.insert_resource(**underground_level);
    }
}

pub(super) fn extract_world_size(
    mut commands: Commands,
    res_world_size: Extract<Option<Res<WorldSize>>>,
) {
    let Some(world_size) = res_world_size.as_ref() else { return; };

    if world_size.is_changed() {
        commands.insert_resource(**world_size);
    }
}

pub(super) fn extract_textures(
    mut commands: Commands,
    background_texture: Extract<Option<Res<BackgroundTexture>>>,
    ingame_background_texture: Extract<Option<Res<InGameBackgroundTexture>>>,
    world_texture: Extract<Option<Res<WorldTexture>>>,
    tile_texture: Extract<Option<Res<TileTexture>>>,
    lightmap_texture: Extract<Option<Res<LightMapTexture>>>,
) {
    let Some(background_texture) = background_texture.as_ref() else { return; };
    let Some(ingame_background_texture) = ingame_background_texture.as_ref() else { return; };
    let Some(world_texture) = world_texture.as_ref() else { return; };
    let Some(tile_texture) = tile_texture.as_ref() else { return; };
    let Some(lightmap_texture) = lightmap_texture.as_ref() else { return; };

    if background_texture.is_changed() {
        commands.insert_resource((**background_texture).clone());
    }

    if ingame_background_texture.is_changed() {
        commands.insert_resource((**ingame_background_texture).clone());
    }

    if world_texture.is_changed() {
        commands.insert_resource((**world_texture).clone());
    }

    if lightmap_texture.is_changed() {
        commands.insert_resource((**lightmap_texture).clone());
    }

    if tile_texture.is_changed() {
        commands.insert_resource((**tile_texture).clone());
    }
}