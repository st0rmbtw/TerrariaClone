use bevy::{prelude::{Commands, SpatialBundle, Res, Query, Transform, With, Visibility, Color, Name, TextBundle, Input, KeyCode, ResMut, UVec2, Vec4, DetectChanges}, text::{TextStyle, Text, TextSection}, ui::{Style, Val, PositionType}, utils::default, window::{Window, PrimaryWindow}};
use bevy_inspector_egui::bevy_egui::EguiContexts;
use rand::{thread_rng, Rng};

use crate::{plugins::{DespawnOnGameExit, cursor::{position::CursorPosition, components::CursorContainer}, camera::components::MainCamera, assets::{FontAssets, ParticleAssets}, particles::{ParticleBuilder, ParticleCommandsExt}}, lighting::types::LightSource, common::helpers::{self, random_point_circle, random_point_ring, random_point_cone}, world::WorldData};

use super::{resources::{MouseLightSettings, DebugConfiguration, MouseParticleSettings, ParticleSpawnType, HoverBlockData}, components::{MouseLight, FreeCameraLegendText}};

pub(super) fn spawn_mouse_light(mut commands: Commands) {
    commands.spawn((
        DespawnOnGameExit,
        SpatialBundle::default(),
        LightSource {
            size: UVec2::splat(1),
            color: Vec4::from(Color::RED).truncate(),
            intensity: 1.,
            jitter_intensity: 0.0,
        }, 
        MouseLight
    ));
}

pub(super) fn update_mouse_light(
    cursor_pos: Res<CursorPosition<MainCamera>>,
    mouse_light: Res<MouseLightSettings>,
    mut query: Query<(&mut Transform, &mut Visibility, &mut LightSource), With<MouseLight>>
) {
    let Ok((mut transform, mut visibility, mut light_source)) = query.get_single_mut() else { return; };

    if mouse_light.enabled {
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }

    light_source.color = mouse_light.color;
    light_source.intensity = mouse_light.intensity;
    light_source.jitter_intensity = mouse_light.jitter_intensity;

    transform.translation.x = cursor_pos.world.x;
    transform.translation.y = cursor_pos.world.y;
}

pub(super) fn set_free_camera_legend_visibility(
    mut query: Query<&mut Visibility, With<FreeCameraLegendText>>,
    debug_config: Res<DebugConfiguration>
) {
    if debug_config.is_changed() {
        let visibility = query.single_mut();
        helpers::set_visibility(visibility, debug_config.free_camera);
    }
}

pub(super) fn spawn_free_camera_legend(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
) {
    let text_style = TextStyle {
        font: font_assets.andy_regular.clone_weak(),
        font_size: 22.,
        color: Color::WHITE,
    };

    commands.spawn((
        Name::new("Free Camera Legend Text"),
        FreeCameraLegendText,
        DespawnOnGameExit,
        TextBundle {
            style: Style {
                left: Val::Px(20.),
                bottom: Val::Px(50.),
                position_type: PositionType::Absolute,
                ..default()
            },
            text: Text::from_sections([
                TextSection::new("Right Click to teleport\nWASD to pan, Shift for faster, Alt for slower", text_style),
            ]),
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
}

pub(super) fn spawn_particles(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    cursor_pos: Res<CursorPosition<MainCamera>>,
    mouse_particle: Res<MouseParticleSettings>
) { 
    if input.just_pressed(KeyCode::P) {
        let mut rng = thread_rng();

        for _ in 0..mouse_particle.count {
            let y = mouse_particle.index / ParticleAssets::COLUMNS;
            let x = mouse_particle.index % ParticleAssets::COLUMNS;
            let index = (y * 3 + rng.gen_range(0..3)) * ParticleAssets::COLUMNS + x;

            let velocity = match mouse_particle.spawn_type {
                ParticleSpawnType::Circle { width, height, radius } => {
                    let point = random_point_circle(width, height) * radius;

                    point * mouse_particle.velocity
                },
                ParticleSpawnType::Ring { width, height } => {
                    let point = random_point_ring(width, height);

                    point.normalize() * mouse_particle.velocity
                },
                ParticleSpawnType::Cone { direction, angle, radius } => {
                    let point = random_point_cone(direction, angle, radius);

                    point.normalize() * mouse_particle.velocity
                },
            };

            let mut particle = ParticleBuilder::from_index(index, cursor_pos.world, velocity, mouse_particle.lifetime);

            if let Some(light_color) = mouse_particle.light_color {
                particle = particle.with_light_color(light_color.extend(1.));
            }

            commands.spawn_particle(particle);
        }
    }
}

pub(super) fn block_hover(
    cursor: Res<CursorPosition<MainCamera>>,
    world_data: Res<WorldData>,
    mut block_data: ResMut<HoverBlockData>
) {
    let tile_pos = helpers::get_tile_pos_from_world_coords(world_data.size, cursor.world);
    let block_type = world_data.get_block(tile_pos).map(|b| *b);
    let neighbors = world_data.get_block_neighbors(tile_pos, true);

    block_data.pos = tile_pos;
    block_data.block_type = block_type.map(|b| b.block_type);
    block_data.neighbors = neighbors.map_ref(|b| b.block_type);
}

pub(super) fn cursor_visibility(
    mut query_window: Query<&mut Window, With<PrimaryWindow>>,
    mut query_cursor: Query<&mut Visibility, With<CursorContainer>>,
    mut egui: EguiContexts
) {
    let Ok(mut window) = query_window.get_single_mut() else { return; };
    let Ok(cursor_visibility) = query_cursor.get_single_mut() else { return; };

    let ctx = egui.ctx_mut();
    window.cursor.visible = ctx.is_pointer_over_area();
    helpers::set_visibility(cursor_visibility, !ctx.is_pointer_over_area());
}