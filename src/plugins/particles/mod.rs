mod systems;
pub(crate) mod components;

use bevy::{prelude::{Plugin, App, FixedUpdate, Vec2, World, Transform, GlobalTransform, Visibility, ComputedVisibility, Commands, IntoSystemConfigs, KeyCode, Update, Color, UVec2, Vec4}, ecs::system::Command, utils::default, sprite::TextureAtlasSprite, input::common_conditions::input_pressed, time::Time};
use rand::{thread_rng, Rng};

use crate::{common::components::Velocity, lighting::types::LightSource};

use self::components::{ParticleBundle, ParticleData};

use super::{assets::ParticleAssets, InGameSystemSet};

const PARTICLE_SIZE: f32 = 8.;

pub(crate) struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            systems::update_particle_position
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

struct SpawnParticleCommand {
    particle: Particle,
    velocity: Velocity,
    position: Vec2,
    lifetime: f64,
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
                spawn_time: time.elapsed_seconds_f64()
            },
            sprite: TextureAtlasSprite {
                index: get_particle_index(self.particle, rng.gen_range(0..3)),
                ..default()
            },
            texture_atlas: particle_assets.particles.clone_weak(),
            transform: Transform::from_xyz(self.position.x, self.position.y, 10.),
            velocity: self.velocity,
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
    fn spawn_particle(&mut self, particle: Particle, position: Vec2, velocity: Velocity, lifetime: f64);
    fn spawn_particle_light(&mut self, particle: Particle, position: Vec2, velocity: Velocity, lifetime: f64, light_color: Color);
}

impl ParticleCommandsExt for Commands<'_, '_> {
    fn spawn_particle(&mut self, particle: Particle, position: Vec2, velocity: Velocity, lifetime: f64) {
        self.add(SpawnParticleCommand {
            particle,
            velocity,
            position,
            lifetime,
            light_source: None
        });
    }

    fn spawn_particle_light(&mut self, particle: Particle, position: Vec2, velocity: Velocity, lifetime: f64, light_color: Color) {
        self.add(SpawnParticleCommand {
            particle,
            velocity,
            position,
            lifetime,
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