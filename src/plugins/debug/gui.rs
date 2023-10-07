use bevy::{prelude::{ResMut, Res, Query, AppTypeRegistry, With, Vec3}, time::Time, reflect::{ReflectMut, Reflect}, window::{PrimaryWindow, Window}};
use bevy_inspector_egui::{bevy_egui::EguiContexts, egui::{self, Align2, ScrollArea, CollapsingHeader, DragValue, Checkbox, Grid, Vec2, Layout, Align}, reflect_inspector};

use crate::world::{chunk::ChunkContainer, block::BlockType};

use super::resources::{DebugConfiguration, HoverBlockData, MouseParticleSettings, MouseLightSettings};

pub(super) fn debug_gui(
    mut contexts: EguiContexts, 
    mut debug_config: ResMut<DebugConfiguration>,
    mut time: ResMut<Time>,
    type_registry: Res<AppTypeRegistry>,
    query_chunk: Query<&ChunkContainer>
) {
    let chunk_count = query_chunk.iter().count();

    let egui_context = contexts.ctx_mut();

    let mut time_speed = time.relative_speed();

    egui::Window::new("Debug Menu")
        .anchor(Align2::RIGHT_TOP, (-10., 10.))
        .resizable(false)
        .show(egui_context, |ui| {
            ui.checkbox(&mut debug_config.free_camera, "Free Camera");
            ui.checkbox(&mut debug_config.show_hitboxes, "Show Hitboxes");
            ui.checkbox(&mut debug_config.show_collisions, "Show Collisions");
            ui.checkbox(&mut debug_config.show_tiles, "Show Tiles");
            ui.checkbox(&mut debug_config.show_walls, "Show Walls");
            ui.checkbox(&mut debug_config.shadow_tiles, "Shadow Tiles");
            ui.checkbox(&mut debug_config.instant_break, "Break tiles instantly");

            if let ReflectMut::Struct(str) = time.reflect_mut() {
                ui.heading("Game Time");
                ui.separator();
                ui.columns(2, |columns| {
                    columns[0].label("Pause:");
                    reflect_inspector::ui_for_value(str.field_mut("paused").unwrap(), &mut columns[1], &type_registry.0.read());
                    columns[0].label("Speed:");
                    columns[1].add(DragValue::new(&mut time_speed)
                        .max_decimals(4)
                        .speed(0.01)
                        .clamp_range(0.001..=1.0));
                });
            }

            ui.heading("Player Speed");
            ui.separator();
            ui.columns(2, |columns| {
                columns[0].label("Horizontal:");
                reflect_inspector::ui_for_value_readonly(&debug_config.player_speed.x, &mut columns[1], &type_registry.0.read());
                columns[0].label("Vertical:");
                reflect_inspector::ui_for_value_readonly(&debug_config.player_speed.y, &mut columns[1], &type_registry.0.read());
            });

            ui.separator();
            ui.columns(2, |columns| {
                columns[0].label("Chunks:");
                reflect_inspector::ui_for_value_readonly(&chunk_count, &mut columns[1], &type_registry.0.read());
            });
        });

    time.set_relative_speed(time_speed);
}

pub(super) fn block_gui(
    mut contexts: EguiContexts,
    block_data: Res<HoverBlockData>,
    type_registry: Res<AppTypeRegistry>,
) {
    let egui_context = contexts.ctx_mut();

    egui::Window::new("Hover Block Data")
        .anchor(Align2::RIGHT_BOTTOM, (-10., -10.))
        .resizable(false)
        .default_size((320., 200.))
        .default_open(false)
        .show(egui_context, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.columns(3, |columns| {
                    columns[0].label("Tile Pos:");
                    reflect_inspector::ui_for_value_readonly(&block_data.pos.x, &mut columns[1], &type_registry.0.read());
                    reflect_inspector::ui_for_value_readonly(&block_data.pos.y, &mut columns[2], &type_registry.0.read());
                });

                ui.columns(2, |columns| {
                    columns[0].label("Block Type:");
                    
                    if let Some(block_type) = &block_data.block_type {
                        reflect_inspector::ui_for_value_readonly(block_type, &mut columns[1], &type_registry.0.read());
                    } else {
                        reflect_inspector::ui_for_value_readonly(&Option::<BlockType>::None, &mut columns[1], &type_registry.0.read());
                    }
                });

                CollapsingHeader::new("Neighbors").show(ui, |ui| {
                    ui.columns(2, |columns| {
                        columns[0].label("North:");
                        columns[0].label("South:");
                        columns[0].label("East:");
                        columns[0].label("West:");
                        columns[0].label("NorthWest:");
                        columns[0].label("NorthEast:");

                        if let Some(n) = &block_data.neighbors.north {
                            reflect_inspector::ui_for_value_readonly(n, &mut columns[1], &type_registry.0.read());
                        } else {
                            reflect_inspector::ui_for_value_readonly(&Option::<BlockType>::None, &mut columns[1], &type_registry.0.read());
                        }
                        
                        if let Some(n) = &block_data.neighbors.south {
                            reflect_inspector::ui_for_value_readonly(n, &mut columns[1], &type_registry.0.read());
                        } else {
                            reflect_inspector::ui_for_value_readonly(&Option::<BlockType>::None, &mut columns[1], &type_registry.0.read());
                        }

                        if let Some(n) = &block_data.neighbors.east {
                            reflect_inspector::ui_for_value_readonly(n, &mut columns[1], &type_registry.0.read());
                        } else {
                            reflect_inspector::ui_for_value_readonly(&Option::<BlockType>::None, &mut columns[1], &type_registry.0.read());
                        }

                        if let Some(n) = &block_data.neighbors.west {
                            reflect_inspector::ui_for_value_readonly(n, &mut columns[1], &type_registry.0.read());
                        } else {
                            reflect_inspector::ui_for_value_readonly(&Option::<BlockType>::None, &mut columns[1], &type_registry.0.read());
                        }

                        if let Some(n) = &block_data.neighbors.north_west {
                            reflect_inspector::ui_for_value_readonly(n, &mut columns[1], &type_registry.0.read());
                        } else {
                            reflect_inspector::ui_for_value_readonly(&Option::<BlockType>::None, &mut columns[1], &type_registry.0.read());
                        }

                        if let Some(n) = &block_data.neighbors.north_east {
                            reflect_inspector::ui_for_value_readonly(n, &mut columns[1], &type_registry.0.read());
                        } else {
                            reflect_inspector::ui_for_value_readonly(&Option::<BlockType>::None, &mut columns[1], &type_registry.0.read());
                        }
                    });
                });

                ui.allocate_space(ui.available_size());
            });
        });
}

pub(super) fn particle_gui(
    mut contexts: EguiContexts,
    mut mouse_particle: ResMut<MouseParticleSettings>,
    type_registry: Res<AppTypeRegistry>,
    query_window: Query<&Window, With<PrimaryWindow>>
) {
    let window = query_window.single();
    let width = window.width();
    let height = window.height();

    let egui_context = contexts.ctx_mut();
    
    let mut light_color = mouse_particle.light_color.is_some();
    let mut color = mouse_particle.light_color.unwrap_or(Vec3::new(1., 0., 0.)).to_array();

    let pos = (
        0. * width + 10.,
        0.5 * height + 120.
    );

    egui::Window::new("Spawn Particles")
        .resizable(false)
        .default_pos(pos)
        .default_open(false)
        .show(&egui_context, |ui| {
            Grid::new("grid")
                .num_columns(2)
                .show(ui, |grid| {
                    grid.allocate_ui_with_layout(
                        grid.max_rect().size() * Vec2::new(0.5, 1.),
                        Layout::top_down_justified(Align::Min),
                        |ui| ui.label("Index")
                    );
                    grid.add_sized(
                        grid.available_size_before_wrap(),
                        DragValue::new(&mut mouse_particle.index)
                            .clamp_range(0..=323)
                            .speed(1.)
                    );
                    grid.end_row();

                    grid.label("Velocity");
                    grid.columns(2, |columns| {
                        reflect_inspector::ui_for_value(&mut mouse_particle.velocity.x, &mut columns[0], &type_registry.0.read());
                        reflect_inspector::ui_for_value(&mut mouse_particle.velocity.y, &mut columns[1], &type_registry.0.read());
                    });
                    grid.end_row();

                    grid.label("Lifetime");
                    grid.add_sized(grid.available_size_before_wrap(), DragValue::new(&mut mouse_particle.lifetime).speed(0.1));
                    grid.end_row();

                    grid.label("Count");
                    grid.add_sized(grid.available_size_before_wrap(), DragValue::new(&mut mouse_particle.count));
                    grid.end_row();

                    grid.label("Spawn type");
                    reflect_inspector::ui_for_value(&mut mouse_particle.spawn_type, grid, &type_registry.0.read());
                    grid.end_row();

                    grid.label("Light color");
                    grid.columns(2, |columns| {
                        columns[0].add(Checkbox::without_text(&mut light_color));
                        if light_color {
                            columns[1].color_edit_button_rgb(&mut color);
                        }
                    });
                });
        });

    mouse_particle.light_color = light_color.then_some(Vec3::from_array(color));
}

pub(super) fn mouse_light_gui(
    mut contexts: EguiContexts,
    mut mouse_light: ResMut<MouseLightSettings>,
    query_window: Query<&Window, With<PrimaryWindow>>
) {
    let window = query_window.single();
    let width = window.width();
    let height = window.height();

    let egui_context = contexts.ctx_mut();
    
    let mut color = mouse_light.color.to_array();

    let pos = (
        0. * width + 10.,
        0.5 * height - 25.
    );

    egui::Window::new("Mouse Light")
        .resizable(false)
        .default_open(false)
        .default_pos(pos)
        .show(&egui_context, |ui| {
            Grid::new("grid")
                .num_columns(2)
                .show(ui, |grid| {
                    grid.allocate_ui_with_layout(
                        grid.max_rect().size() * Vec2::new(0.5, 1.),
                        Layout::top_down_justified(Align::Min),
                        |ui| ui.label("Color")
                    );
                    grid.color_edit_button_rgb(&mut color);
                    grid.end_row();

                    grid.label("Intensity");
                    grid.add_sized(
                        grid.available_size_before_wrap(),
                        DragValue::new(&mut mouse_light.intensity)
                            .speed(0.1)
                            .clamp_range(0.0..=1.0)
                    );
                    grid.end_row();

                    grid.label("Jitter intensity");
                    grid.add_sized(
                        grid.available_size_before_wrap(),
                        DragValue::new(&mut mouse_light.jitter_intensity)
                            .speed(0.1)
                            .clamp_range(0.0..=1.0)
                    );
                    grid.end_row();

                    grid.label("Enabled");
                    grid.add(Checkbox::without_text(&mut mouse_light.enabled));
                });
        });

    mouse_light.color = Vec3::from_array(color);
}