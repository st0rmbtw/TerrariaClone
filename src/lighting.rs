use bevy::{prelude::{Plugin, App, Color, OrthographicProjection, With, Query, Res, Transform, Vec2}, sprite::TextureAtlasSprite};
use iyes_loopless::prelude::ConditionSet;

use crate::{plugins::{MainCamera, WorldData, Player}, state::GameState, util::get_tile_coords};

// region: Plugin
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(lighting)
                    .into()
            );
    }
}
// endregion

fn lighting(
    camera_query: Query<&OrthographicProjection, With<MainCamera>>,
    player_query: Query<&Transform, With<Player>>,
    mut color_query: Query<(&mut TextureAtlasSprite, &Transform)>,
    world_data: Res<WorldData>
) {
    let projection = camera_query.single();
    let player_transform = player_query.single();

    let player_position = get_tile_coords(player_transform.translation.truncate());

    let width = (projection.left.abs() + projection.right) * projection.scale / 16.;
    // let height = (projection.bottom.abs() + projection.top) * projection.scale / 16.;

    let light = |tile_pos: Vec2| -> Color {
        let d = tile_pos.distance_squared(player_position);

        let c = 0.7_f32.powf(d / width);

        Color::rgb(c, c, c)
    };

    for chunk in world_data.get_visible_chunks() {
        for cell in chunk.cells.iter() {
            if let Some(entity) = cell.tile_entity {
                if let Ok((mut sprite, tile_transform)) = color_query.get_mut(entity) {
                    let tile_pos = get_tile_coords(tile_transform.translation.truncate());

                    sprite.color = light(tile_pos);
                }
            }

            if let Some(entity) = cell.wall_entity {
                if let Ok((mut sprite, tile_transform)) = color_query.get_mut(entity) {
                    let tile_pos = get_tile_coords(tile_transform.translation.truncate());

                    sprite.color = light(tile_pos);
                }
            }
        }   
    }
}