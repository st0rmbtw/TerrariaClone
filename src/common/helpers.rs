use std::f32::consts::PI;

use bevy::{prelude::{Visibility, Vec2, Mut}, math::vec2};
use bevy_ecs_tilemap::tiles::TilePos;
use rand::{thread_rng, Rng};

use crate::{plugins::world::constants::TILE_SIZE, world::{block::BlockType, wall::Wall, Size}};

use super::TextureAtlasPos;

pub(crate) const fn get_tile_start_index(block: BlockType) -> TextureAtlasPos {
    match block {
        BlockType::Dirt => TextureAtlasPos::ZERO,
        BlockType::Stone => TextureAtlasPos::new(0, 15),
        BlockType::Grass => TextureAtlasPos::new(0, 30),
        BlockType::Tree(_) => TextureAtlasPos::ZERO,
    }
}

pub(crate) const fn get_wall_start_index(wall: Wall) -> TextureAtlasPos {
    match wall {
        Wall::Stone => TextureAtlasPos::ZERO,
        Wall::Dirt => TextureAtlasPos::new(0, 5),
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
        .min(Vec2::new(world_size.width as f32, world_size.height as f32));

    TilePos::new(tile_pos.x as u32, tile_pos.y as u32)
}

pub(crate) fn tile_pos_to_world_coords(tile_pos: TilePos) -> Vec2 {
    vec2(tile_pos.x as f32 * TILE_SIZE + TILE_SIZE / 2., -(tile_pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.))
}

pub(crate) fn random_point_circle(width: f32, height: f32) -> Vec2 {
    debug_assert!((0.0..=1.0).contains(&width));
    debug_assert!((0.0..=1.0).contains(&height));

    let mut rng = thread_rng();
    
    let radius = Vec2::new(width, height) * rng.gen_range(0f32..1f32).sqrt();
    let theta = rng.gen_range(0.0..1.0) * 2. * PI;

    let theta_cos = theta.cos();
    let theta_sin = theta.sin();

    let x = radius.x * theta_cos;
    let y = radius.y * theta_sin;

    Vec2::new(x, y)
}

pub(crate) fn random_point_ring(width: f32, height: f32) -> Vec2 {
    debug_assert!((0.0..=1.0).contains(&width));
    debug_assert!((0.0..=1.0).contains(&height));

    let mut rng = thread_rng();
    
    let radius = Vec2::new(width, height);
    let theta = rng.gen_range(0.0..1.0) * 2. * PI;

    let theta_cos = theta.cos();
    let theta_sin = theta.sin();

    let x = radius.x * theta_cos;
    let y = radius.y * theta_sin;

    Vec2::new(x, y)
}

pub(crate) fn random_point_cone(direction: Vec2, angle: f32, radius: f32) -> Vec2 {
    debug_assert!((0.0..=1.0).contains(&direction.x));
    debug_assert!((0.0..=1.0).contains(&direction.y));
    debug_assert!((0.0..=180.0).contains(&angle));

    let mut rng = thread_rng();

    let rad = angle.to_radians();

    let a = direction.y.atan2(direction.x);

    let start_angle = -(rad / 2.) + a;
    let end_angle = rad / 2. + a;

    let theta = rng.gen_range(start_angle..end_angle);

    let x = theta.cos();
    let y = theta.sin();

    Vec2::new(x, y) * rng.gen_range(0.0..radius)
}