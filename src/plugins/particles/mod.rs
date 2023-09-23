mod systems;
pub(crate) mod components;

use bevy::{prelude::{Plugin, App, FixedUpdate, Vec2, World, Transform, GlobalTransform, Visibility, ComputedVisibility, Commands, IntoSystemConfigs, KeyCode, Update, Color, UVec2, Vec4}, ecs::system::Command, utils::default, sprite::TextureAtlasSprite, input::common_conditions::input_pressed, time::Time, render::view::RenderLayers};
use rand::{thread_rng, Rng};

use crate::{common::components::Velocity, lighting::types::LightSource, world::block::BlockType};

use self::components::{ParticleBundle, ParticleData};

use super::{assets::ParticleAssets, InGameSystemSet};

pub(crate) const PARTICLE_SIZE: f32 = 8.;

pub(crate) struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                systems::update_particle_velocity,
                systems::update_particle_position
            )
            .chain()
            .in_set(InGameSystemSet::FixedUpdate)
        );

        app.add_systems(
            Update,
            (
                systems::update_particle_opacity,
                systems::try_spawn_particles
                    .run_if(input_pressed(KeyCode::P))
            )
            .in_set(InGameSystemSet::Update)
        );
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Particle {
    Dirt,
    Stone,
    Grass,
}

impl Particle {
    pub(crate) const fn get_by_block(block_type: BlockType) -> Option<Self> {
        match block_type {
            BlockType::Dirt => Some(Particle::Dirt),
            BlockType::Stone => Some(Particle::Stone),
            BlockType::Grass => Some(Particle::Grass),
            _ => None
        }
    }
}

struct SpawnParticleCommand {
    particle: Particle,
    velocity: Velocity,
    position: Vec2,
    lifetime: f64,
    size: Option<f32>,
    gravity: bool,
    render_layer: RenderLayers,
    light_source: Option<LightSource>
}

impl Command for SpawnParticleCommand {
    fn apply(self, world: &mut World) {
        let mut rng = thread_rng();

        let particle_assets = world.resource::<ParticleAssets>();
        let time = world.resource::<Time>();

        let mut entity = world.spawn(ParticleBundle {
            particle_data: ParticleData {
                lifetime: self.lifetime,
                size: self.size,
                gravity: self.gravity,
                spawn_time: time.elapsed_seconds_f64(),
            },
            sprite: TextureAtlasSprite {
                index: get_particle_index(self.particle, rng.gen_range(0..3)),
                custom_size: self.size.map(|size| Vec2::splat(size)),
                ..default()
            },
            texture_atlas: particle_assets.particles.clone_weak(),
            transform: Transform::from_xyz(self.position.x, self.position.y, 100.),
            velocity: self.velocity,
            render_layer: self.render_layer,
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
        });

        if let Some(light_source) = self.light_source {
            entity.insert(light_source);
        }
    }
}

pub(crate) trait ParticleCommandsExt {
    fn spawn_particle(&mut self, particle: Particle, position: Vec2, velocity: Velocity, lifetime: f64, size: Option<f32>, gravity: bool, render_layer: RenderLayers);
    fn spawn_particle_light(&mut self, particle: Particle, position: Vec2, velocity: Velocity, lifetime: f64, size: Option<f32>, gravity: bool, render_layer: RenderLayers, light_color: Color);
}

impl ParticleCommandsExt for Commands<'_, '_> {
    fn spawn_particle(
        &mut self,
        particle: Particle,
        position: Vec2,
        velocity: Velocity,
        lifetime: f64,
        size: Option<f32>,
        gravity: bool,
        render_layer: RenderLayers,
    ) {
        self.add(SpawnParticleCommand {
            particle,
            velocity,
            position,
            lifetime,
            size,
            gravity,
            render_layer,
            light_source: None
        });
    }

    fn spawn_particle_light(
        &mut self,
        particle: Particle,
        position: Vec2,
        velocity: Velocity,
        lifetime: f64,
        size: Option<f32>,
        gravity: bool,
        render_layer: RenderLayers,
        light_color: Color,
    ) {
        self.add(SpawnParticleCommand {
            particle,
            velocity,
            position,
            lifetime,
            size,
            gravity,
            render_layer,
            light_source: Some(LightSource {
                size: UVec2::splat(1),
                color: Vec4::from(light_color).truncate(),
                intensity: 1.,
                jitter_intensity: 0.
            })
        });
    }
}

fn get_particle_index(particle: Particle, variant: u8) -> usize {
    let start = match particle {
        Particle::Dirt => 0,
        Particle::Stone => 1,
        Particle::Grass => 2,
    };
    let y = variant as usize;

    start + y * ParticleAssets::COLUMNS
}