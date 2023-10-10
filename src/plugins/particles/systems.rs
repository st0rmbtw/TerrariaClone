use bevy::{prelude::{Query, Transform, With, Commands, Res, Vec2, Entity}, sprite::TextureAtlasSprite, time::Time};

use crate::{common::math::map_range_f64, lighting::types::LightSource, plugins::entity::components::Velocity};

use super::{components::ParticleData, PARTICLE_SIZE};

pub(super) fn update_particle_position(
    mut query: Query<(&mut Transform, &Velocity), With<ParticleData>>
) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
    }
}

pub(super) fn update_particle_velocity(
    mut query: Query<(&mut Velocity, &ParticleData)>
) {
    for (mut velocity, particle_data) in &mut query {
        if particle_data.gravity {
            velocity.y -= 0.1;
        }
    }
}

pub(super) fn update_particle_rotation(
    mut query: Query<(&mut Transform, &ParticleData)>
) {
    for (mut transform, particle_data) in &mut query {
        transform.rotate_local_z(particle_data.rotation_speed);
    }
}

pub(super) fn update_particle_over_lifetime(
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
            light_source.intensity = factor;
        }
    }
}