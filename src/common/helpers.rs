use bevy::{prelude::{Visibility, Component, With, Query, Vec2, Mut}, math::vec2};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{plugins::world::constants::TILE_SIZE, world::{block::BlockType, wall::Wall, Size}};

use super::TextureAtlasPos;

pub(crate) fn get_tile_start_index(block: BlockType) -> TextureAtlasPos {
    match block {
        BlockType::Dirt => TextureAtlasPos::ZERO,
        BlockType::Stone => TextureAtlasPos::new(0, 15),
        BlockType::Grass => TextureAtlasPos::new(0, 30),
        BlockType::Tree(_) => TextureAtlasPos::ZERO,
    }
}

pub(crate) fn get_wall_start_index(wall: Wall) -> TextureAtlasPos {
    match wall {
        Wall::Stone => TextureAtlasPos::ZERO,
        Wall::Dirt => TextureAtlasPos::new(0, 5),
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

#[inline(always)]
pub(crate) fn set_visibility(mut visibility: Mut<Visibility>, visible: bool) {
    if visible {
        *visibility = Visibility::Inherited;
    } else {
        *visibility = Visibility::Hidden;
    }
}

pub(crate) fn get_tile_pos_from_world_coords(world_size: Size, world_coords: Vec2) -> TilePos {
    let tile_pos = (Vec2::new(world_coords.x, world_coords.y.abs()) / TILE_SIZE)
        .round()
        .clamp(Vec2::new(0., 0.), Vec2::new(world_size.width as f32, world_size.height as f32));

    TilePos::new(tile_pos.x as u32, tile_pos.y as u32)
}

pub(crate) fn tile_pos_to_world_coords(tile_pos: TilePos) -> Vec2 {
    vec2(tile_pos.x as f32 * TILE_SIZE, -(tile_pos.y as f32) * TILE_SIZE)
}