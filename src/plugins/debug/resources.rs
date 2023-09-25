use bevy::{prelude::{Resource, Vec2, Vec3, ReflectDefault, Color, Vec4}, reflect::Reflect};
use bevy_ecs_tilemap::{tiles::TilePos, helpers::square_grid::neighbors::Neighbors};

use crate::world::block::BlockType;

#[derive(Resource)]
pub(super) struct HoverBlockData {
    pub(super) pos: TilePos,
    pub(super) block_type: Option<BlockType>,
    pub(super) neighbors: Neighbors<BlockType>
}

#[derive(Resource)]
pub(crate) struct DebugConfiguration {
    pub(crate) free_camera: bool,
    pub(crate) instant_break: bool,

    pub(crate) show_hitboxes: bool,
    pub(crate) show_collisions: bool,
    pub(crate) show_tiles: bool,
    pub(crate) show_walls: bool,
    pub(crate) shadow_tiles: bool,
    pub(crate) player_speed: Vec2,
}

impl Default for DebugConfiguration {
    fn default() -> Self {
        Self {
            free_camera: false,
            instant_break: false,
            show_hitboxes: false,
            shadow_tiles: false,
            show_collisions: true,
            show_tiles: true,
            show_walls: true,
            player_speed: Default::default()
        }
    }
}

#[derive(Resource)]
pub(super) struct MouseParticleSettings {
    pub(super) index: usize,
    pub(super) velocity: Vec2,
    pub(super) count: u32,
    pub(super) spawn_type: ParticleSpawnType,
    pub(super) light_color: Option<Vec3>,
    pub(super) lifetime: f64
}

impl Default for MouseParticleSettings {
    fn default() -> Self {
        Self {
            index: 0,
            velocity: Vec2::splat(1.),
            lifetime: 1.,
            count: 5,
            spawn_type: Default::default(),
            light_color: Default::default()
        }
    }
}

#[derive(Reflect)]
#[reflect(Default)]
pub(super) enum ParticleSpawnType {
    Circle { width: f32, height: f32, radius: f32 },
    Ring { width: f32, height: f32 },
    Cone { direction: Vec2, angle: f32, radius: f32 }
}

impl Default for ParticleSpawnType {
    fn default() -> Self {
        ParticleSpawnType::Circle { width: 1., height: 1., radius: 1. }
    }
}

#[derive(Resource)]
pub(super) struct MouseLightSettings {
    pub(super) enabled: bool,
    pub(super) color: Vec3,
    pub(super) intensity: f32,
    pub(super) jitter_intensity: f32
}

impl Default for MouseLightSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            color: Vec4::from(Color::RED).truncate(),
            intensity: 1.,
            jitter_intensity: 0.,
        }
    }
}