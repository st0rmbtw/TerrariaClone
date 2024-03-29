use std::f32::consts::PI;

use bevy::prelude::{Vec2, OrthographicProjection, UVec2, Commands};
use bevy_ecs_tilemap::tiles::TilePos;
use rand::{thread_rng, Rng};

use crate::{world::chunk::ChunkPos, common::helpers::{random_point_circle, tile_to_world_pos}, plugins::particles::{PARTICLE_SIZE, Particle, ParticleCommandsExt, ParticleBuilder}};

use super::{constants::{CHUNK_SIZE_U, CHUNK_SIZE, TILE_SIZE}, CameraFov, ChunkRange, WORLD_RENDER_LAYER};

#[inline(always)]
pub(super) fn get_chunk_pos(pos: TilePos) -> ChunkPos {
    ChunkPos::from(pos) / CHUNK_SIZE_U
}

#[inline(always)]
pub(super) fn get_chunk_tile_pos(map_tile_pos: TilePos) -> TilePos {
    TilePos { 
        x: map_tile_pos.x % CHUNK_SIZE_U, 
        y: CHUNK_SIZE_U - 1 - (map_tile_pos.y % CHUNK_SIZE_U)
    }
}

#[inline(always)]
pub(super) fn get_camera_fov(camera_pos: Vec2, projection: &OrthographicProjection) -> CameraFov {
    CameraFov {
        min: camera_pos + projection.area.min,
        max: camera_pos + projection.area.max
    }
}

pub(super) fn get_chunk_range_by_camera_fov(camera_fov: CameraFov, world_size: UVec2) -> ChunkRange {
    let left = ((camera_fov.min.x / TILE_SIZE - 1.) / CHUNK_SIZE).floor() as u32;
    let mut right = ((camera_fov.max.x / TILE_SIZE + 1.) / CHUNK_SIZE).ceil() as u32;
    let top = ((camera_fov.max.y / TILE_SIZE + 1.) / CHUNK_SIZE).ceil().min(0.).abs() as u32;
    let mut bottom = ((camera_fov.min.y / TILE_SIZE - 1.) / CHUNK_SIZE).floor().abs() as u32;

    let max_chunk_x = world_size.x / CHUNK_SIZE_U;
    let max_chunk_y = world_size.y / CHUNK_SIZE_U;

    if right > max_chunk_x {
        right = max_chunk_x;
    }
    
    if bottom > max_chunk_y {
        bottom = max_chunk_y;
    }

    ChunkRange {
        min: UVec2::new(left, top),
        max: UVec2::new(right, bottom),
    }
}

pub(super) fn spawn_particles_on_dig(commands: &mut Commands, particle: Particle, tile_pos: TilePos) {
    let mut rng = thread_rng();

    for _ in 0..3 {
        let point = random_point_circle(1., 1.) * 8.;
        let velocity = point.normalize() * 1.5;
        let size = rng.gen_range(0.3..1.0) * PARTICLE_SIZE;

        commands.spawn_particle(
            ParticleBuilder::new(particle, tile_to_world_pos(tile_pos), velocity, 1.)
                .with_size(size)
                .with_gravity(true)
                .with_render_layer(WORLD_RENDER_LAYER)
                .with_rotation(PI / 12.)
        );
    }
}

pub(super) fn spawn_particles_on_break(commands: &mut Commands, particle: Particle, tile_pos: TilePos) {
    let mut rng = thread_rng();

    for _ in 0..10 {
        let point = random_point_circle(1., 1.) * 8.;
        let velocity = point.normalize() * 1.5;
        let size = rng.gen_range(0.3..1.0) * PARTICLE_SIZE;

        commands.spawn_particle(
            ParticleBuilder::new(particle, tile_to_world_pos(tile_pos), velocity, 1.5)
                .with_size(size)
                .with_gravity(true)
                .with_render_layer(WORLD_RENDER_LAYER)
                .with_rotation(PI / 12.)
        );
    }
}