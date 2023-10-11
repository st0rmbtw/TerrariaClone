use bevy::{prelude::{Plugin, App, SystemSet, FixedUpdate, IntoSystemSetConfigs, IntoSystemConfigs, Query, Res, Transform}, math::vec2};

use crate::world::WorldData;

use self::components::{EntityRect, Velocity};

use super::{InGameSystemSet, world::constants::TILE_SIZE};

pub(crate) mod components;

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub(crate) enum EntitySet {
    UpdateEntityRect,
    MoveEntity
}

pub(super) struct EntityPlugin;
impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            (
                EntitySet::UpdateEntityRect,
                EntitySet::MoveEntity
            )
            .chain()
            .in_set(InGameSystemSet::FixedUpdate)
        );

        app.add_systems(
            FixedUpdate,
            (
                update_entity_rect.in_set(EntitySet::UpdateEntityRect),
                move_entity.in_set(EntitySet::MoveEntity),
            )
            .chain()
        );
    }
}

fn update_entity_rect(
    world_data: Res<WorldData>,
    mut query: Query<(&mut EntityRect, &Velocity)>
) {
    for (mut entity_rect, velocity) in &mut query {
        let min_x: f32 = entity_rect.half_width() + world_data.playable_area.min.x as f32 * TILE_SIZE;
        let min_y: f32 = -(world_data.playable_area.height() as f32) * TILE_SIZE;

        let max_x = world_data.playable_area.max.x as f32 * TILE_SIZE - entity_rect.half_width();
        let max_y: f32 = -entity_rect.half_height();

        let new_position = (entity_rect.center() + velocity.0)
            .clamp(vec2(min_x, min_y), vec2(max_x, max_y));

        entity_rect.centerx = new_position.x;
        entity_rect.centery = new_position.y;
    }
}

fn move_entity(
    mut query: Query<(&mut Transform, &EntityRect)>
) {
    query.for_each_mut(|(mut transform, entity_rect)| {
        transform.translation.x = entity_rect.centerx;
        transform.translation.y = entity_rect.centery;
    });
}