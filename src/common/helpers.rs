use bevy::prelude::{Visibility, Component, With, Query, Mut, Vec2};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{plugins::world::{BlockType, Wall, TILE_SIZE}};

use super::TextureAtlasPos;

pub fn get_tile_start_index(block: BlockType) -> TextureAtlasPos {
    match block {
        BlockType::Dirt => TextureAtlasPos::ZERO,
        BlockType::Stone => TextureAtlasPos::new(0, 15),
        BlockType::Grass => TextureAtlasPos::new(0, 30),
        BlockType::Tree(_) => TextureAtlasPos::ZERO,
    }
}

pub fn get_wall_start_index(wall: Wall) -> TextureAtlasPos {
    match wall {
        Wall::Stone => TextureAtlasPos::ZERO,
        Wall::Dirt => TextureAtlasPos::new(0, 5),
    }
}

pub fn toggle_visibility<C: Component>(
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
pub fn set_visibility(mut visibility: Mut<Visibility>, visible: bool) {
    if visible {
        *visibility = Visibility::Inherited;
    } else {
        *visibility = Visibility::Hidden;
    }
}

pub fn get_tile_coords(world_coords: Vec2) -> Vec2 {
    (world_coords / TILE_SIZE).round().abs()
}

pub fn tile_to_world_coords(tile_pos: TilePos) -> Vec2 {
    Vec2::new(tile_pos.x as f32 * TILE_SIZE, -(tile_pos.y as f32) * TILE_SIZE)
}