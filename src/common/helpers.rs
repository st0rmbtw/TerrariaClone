use std::f32::consts::PI;

use bevy::{prelude::{Visibility, Vec2, Mut, UVec2}, math::vec2};
use bevy_ecs_tilemap::tiles::TilePos;
use rand::{thread_rng, Rng};

use crate::{plugins::world::constants::TILE_SIZE, world::{block::BlockType, wall::WallType}};

use super::TextureAtlasPos;

pub(crate) const fn get_tile_start_index(block: BlockType) -> TextureAtlasPos {
    match block {
        BlockType::Dirt => TextureAtlasPos::ZERO,
        BlockType::Stone => TextureAtlasPos::new(0, 15),
        BlockType::Grass => TextureAtlasPos::new(0, 30),
        BlockType::Tree(_) => TextureAtlasPos::ZERO,
        BlockType::Wood => TextureAtlasPos::new(0, 52),
    }
}

pub(crate) const fn get_wall_start_index(wall: WallType) -> TextureAtlasPos {
    match wall {
        WallType::Stone => TextureAtlasPos::ZERO,
        WallType::Dirt => TextureAtlasPos::new(0, 5),
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

pub(crate) fn get_tile_pos_from_world_coords(world_size: UVec2, world_coords: Vec2) -> TilePos {
    let world_size = world_size - UVec2::ONE;

    let tile_pos = (vec2(world_coords.x, world_coords.y.abs()) / TILE_SIZE)
        .min(world_size.as_vec2());

    TilePos::new(tile_pos.x as u32, tile_pos.y as u32)
}

#[inline]
pub(crate) fn tile_to_world_pos(tile_pos: TilePos) -> Vec2 {
    vec2(tile_pos.x as f32 * TILE_SIZE + TILE_SIZE / 2., -(tile_pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.))
}

pub(crate) fn random_point_circle(width: f32, height: f32) -> Vec2 {
    debug_assert!((0.0..=1.0).contains(&width));
    debug_assert!((0.0..=1.0).contains(&height));

    let mut rng = thread_rng();
    
    let radius = vec2(width, height) * rng.gen_range(0f32..1f32).sqrt();
    let theta = rng.gen_range(0.0..1.0) * 2. * PI;

    let x = radius.x * theta.cos();
    let y = radius.y * theta.sin();

    vec2(x, y)
}

pub(crate) fn random_point_ring(width: f32, height: f32) -> Vec2 {
    debug_assert!((0.0..=1.0).contains(&width));
    debug_assert!((0.0..=1.0).contains(&height));

    let mut rng = thread_rng();
    
    let radius = vec2(width, height);
    let theta = rng.gen_range(0.0..1.0) * 2. * PI;

    let x = radius.x * theta.cos();
    let y = radius.y * theta.sin();

    vec2(x, y)
}

pub(crate) fn random_point_cone(direction: Vec2, angle: f32, radius: f32) -> Vec2 {
    debug_assert!((-1.0..=1.0).contains(&direction.x));
    debug_assert!((-1.0..=1.0).contains(&direction.y));
    debug_assert!((0.0..=180.0).contains(&angle));

    let mut rng = thread_rng();

    let rad = angle.to_radians();

    let a = direction.y.atan2(direction.x);

    let start_angle = -(rad / 2.) + a;
    let end_angle = rad / 2. + a;

    let theta = rng.gen_range(start_angle..end_angle);

    let x = theta.cos();
    let y = theta.sin();

    vec2(x, y) * rng.gen_range(0.0..radius)
}