use bevy::{prelude::{Query, Transform, With, Commands, Res, Vec2, Entity, Color}, sprite::TextureAtlasSprite, time::Time};
use rand::{thread_rng, Rng};

use crate::{common::{components::Velocity, math::map_range_f64, helpers::{random_point_cone, random_point_circle, random_point_ring}}, plugins::{cursor::position::CursorPosition, camera::components::MainCamera}, lighting::types::LightSource};

use super::{Particle, ParticleCommandsExt, components::ParticleData, PARTICLE_SIZE};

pub(super) fn update_particle_position(
    mut query: Query<(&mut Transform, &Velocity), With<ParticleData>>
) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
    }
}

pub(super) fn update_particle_opacity(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TextureAtlasSprite, &ParticleData, Option<&mut LightSource>)>
) {
    for (entity, mut sprite, particle_data, light_source) in &mut query {
        if time.elapsed_seconds_f64() >= particle_data.spawn_time + particle_data.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        let factor = map_range_f64(
            particle_data.spawn_time + particle_data.lifetime / 2., particle_data.spawn_time + particle_data.lifetime,
            1., 0.,
            time.elapsed_seconds_f64()
        ).clamp(0., 1.) as f32;

        sprite.custom_size = Some(Vec2::splat(factor * particle_data.size.unwrap_or(PARTICLE_SIZE)));

        if let Some(mut light_source) = light_source {
            light_source.intensity = factor as f32;
        }
    }
}

pub(super) fn try_spawn_particles(
    cursor_pos: Res<CursorPosition<MainCamera>>,
    mut commands: Commands
) {
    let center = cursor_pos.world;

    let mut rng = thread_rng();

    for _ in 0..30 {
        let lifetime = rng.gen_range(0.5f64..2f64);

        // let point = random_point_ring(1., 1.) * 100.;
        let point = random_point_circle(1., 1.);

        let position = center + point;
        let velocity = point;
        // let velocity = Vec2::ZERO;

        commands.spawn_particle_light(
            Particle::Grass,
            position,
            velocity.into(),
            lifetime,
            Color::GREEN,
            None,
        );
    }
}