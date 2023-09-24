mod systems;
pub(crate) mod components;

use bevy::{prelude::{Plugin, App, FixedUpdate, Vec2, World, Transform, GlobalTransform, Visibility, ComputedVisibility, Commands, IntoSystemConfigs, Update, UVec2, Vec4}, ecs::system::Command, utils::default, sprite::TextureAtlasSprite, time::Time, render::view::RenderLayers};
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
                (
                    systems::update_particle_velocity,
                    systems::update_particle_position,
                ).chain(),
                systems::update_particle_rotation
            )
            .in_set(InGameSystemSet::FixedUpdate)
        );

        app.add_systems(
            Update,
            (
                systems::update_particle_over_lifetime,
                // systems::try_spawn_particles
                //     .run_if(input_pressed(KeyCode::P))
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
    Wood
}

impl Particle {
    pub(crate) const fn get_by_block(block_type: BlockType) -> Option<Self> {
        match block_type {
            BlockType::Dirt => Some(Particle::Dirt),
            BlockType::Stone => Some(Particle::Stone),
            BlockType::Grass => Some(Particle::Grass),
            BlockType::Tree(_) => Some(Particle::Wood)
        }
    }
}

pub(crate) struct ParticleBuilder {
    index: usize,
    position: Vec2,
    velocity: Velocity,
    lifetime: f64,
    gravity: bool,
    size: Option<f32>,
    light_source: Option<LightSource>,
    render_layer: RenderLayers,
    rotation_speed: f32
}

impl ParticleBuilder {
    pub(crate) fn new(particle: Particle, position: Vec2, velocity: Vec2, lifetime: f64) -> Self {
        let mut rng = thread_rng();

        ParticleBuilder {
            index: get_particle_index(particle, rng.gen_range(0..3)),
            position,
            velocity: velocity.into(),
            lifetime,
            gravity: false,
            size: None,
            light_source: None,
            render_layer: RenderLayers::default(),
            rotation_speed: 0.
        }
    }

    pub(crate) fn from_index(index: usize, position: Vec2, velocity: Vec2, lifetime: f64) -> Self {
        ParticleBuilder {
            index,
            position,
            velocity: velocity.into(),
            lifetime,
            gravity: false,
            size: None,
            light_source: None,
            render_layer: RenderLayers::default(),
            rotation_speed: 0.
        }
    }

    pub(crate) fn with_size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    pub(crate) fn with_render_layer(mut self, render_layer: RenderLayers) -> Self {
        self.render_layer = render_layer;
        self
    }

    pub(crate) fn with_light_color(mut self, light_color: impl Into<Vec4>) -> Self {
        self.light_source = Some(LightSource {
            size: UVec2::splat(1),
            color: light_color.into().truncate(),
            intensity: 1.,
            jitter_intensity: 0.,
        });
        self
    }

    pub(crate) fn with_gravity(mut self, gravity: bool) -> Self {
        self.gravity = gravity;
        self
    }

    pub(crate) fn with_rotation(mut self, speed: f32) -> Self {
        self.rotation_speed = speed;
        self
    }

    fn build(self) -> SpawnParticleCommand {
        SpawnParticleCommand {
            index: self.index,
            velocity: self.velocity,
            position: self.position,
            lifetime: self.lifetime,
            size: self.size,
            gravity: self.gravity,
            render_layer: self.render_layer,
            light_source: self.light_source,
            rotation_speed: self.rotation_speed
        }
    }
}

struct SpawnParticleCommand {
    index: usize,
    velocity: Velocity,
    position: Vec2,
    lifetime: f64,
    size: Option<f32>,
    gravity: bool,
    render_layer: RenderLayers,
    light_source: Option<LightSource>,
    rotation_speed: f32
}

impl Command for SpawnParticleCommand {
    fn apply(self, world: &mut World) {
        let particle_assets = world.resource::<ParticleAssets>();
        let time = world.resource::<Time>();

        let mut entity = world.spawn(ParticleBundle {
            particle_data: ParticleData {
                lifetime: self.lifetime,
                size: self.size,
                gravity: self.gravity,
                rotation_speed: self.rotation_speed,
                spawn_time: time.elapsed_seconds_f64(),
            },
            sprite: TextureAtlasSprite {
                index: self.index,
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
    fn spawn_particle(&mut self, particle_bulder: ParticleBuilder);
}

impl ParticleCommandsExt for Commands<'_, '_> {
    fn spawn_particle(
        &mut self,
        particle_builder: ParticleBuilder
    ) {
        self.add(particle_builder.build());
    }
}

fn get_particle_index(particle: Particle, variant: u8) -> usize {
    let index = match particle {
        Particle::Dirt => 0,
        Particle::Stone => 1,
        Particle::Grass => 2,
        Particle::Wood => 7,
    };
    let variant = variant as usize;

    let y = index / ParticleAssets::COLUMNS;
    let x = index % ParticleAssets::COLUMNS;

    (y * 3 + variant) * ParticleAssets::COLUMNS + x
}