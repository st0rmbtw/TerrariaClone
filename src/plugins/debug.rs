use bevy::{prelude::{App, Plugin, ResMut, Commands, TextBundle, Res, Color, OnEnter, Query, Visibility, With, DetectChanges, Name, AppTypeRegistry, Vec2, Update, IntoSystemConfigs, Resource, Component}, utils::default, text::{Text, TextSection, TextStyle}, ui::{Style, Val, PositionType}, sprite::TextureAtlasSprite, time::Time, reflect::{Reflect, ReflectMut}};
use bevy_ecs_tilemap::{tiles::TilePos, helpers::square_grid::neighbors::Neighbors};
use bevy_inspector_egui::{bevy_egui::{egui, EguiContexts}, egui::{Align2, CollapsingHeader, ScrollArea}, quick::WorldInspectorPlugin, reflect_inspector};

use crate::{common::{state::GameState, helpers::{self, get_tile_pos_from_world_coords}}, world::{block::BlockType, WorldData, chunk::ChunkContainer}};

use super::{assets::FontAssets, inventory::{UseItemAnimationIndex, UseItemAnimationData}, camera::components::MainCamera, cursor::position::CursorPosition, DespawnOnGameExit, InGameSystemSet};

pub(crate) struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldInspectorPlugin::new());

        app.insert_resource(HoverBlockData {
            pos: TilePos::default(),
            block_type: None,
            neighbors: Neighbors {
                east: None,
                north_east: None,
                north: None,
                north_west: None,
                west: None,
                south_west: None,
                south: None,
                south_east: None,
            },
        });

        app.insert_resource(DebugConfiguration::default());

        app.register_type::<TextureAtlasSprite>();
        app.register_type::<UseItemAnimationIndex>();
        app.register_type::<UseItemAnimationData>();

        app.add_systems(OnEnter(GameState::InGame), spawn_free_camera_legend);
        app.add_systems(
            Update,
            (
                debug_gui,
                block_gui,
                set_free_camera_legend_visibility,
                block_hover,
            )
            .in_set(InGameSystemSet::Update)
        );
    }
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
            player_speed: default()
        }
    }
}

#[derive(Component)]
struct FreeCameraLegendText;

fn debug_gui(
    mut contexts: EguiContexts, 
    mut debug_config: ResMut<DebugConfiguration>,
    mut time: ResMut<Time>,
    type_registry: Res<AppTypeRegistry>,
    query_chunk: Query<&ChunkContainer>
) {
    let chunk_count = query_chunk.iter().count();

    let egui_context = contexts.ctx_mut();

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
                    reflect_inspector::ui_for_value(str.field_mut("relative_speed").unwrap(), &mut columns[1], &type_registry.0.read());
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
}

fn set_free_camera_legend_visibility(
    mut query: Query<&mut Visibility, With<FreeCameraLegendText>>,
    debug_config: Res<DebugConfiguration>
) {
    if debug_config.is_changed() {
        let visibility = query.single_mut();
        helpers::set_visibility(visibility, debug_config.free_camera);
    }
}

fn spawn_free_camera_legend(
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

#[derive(Resource)]
struct HoverBlockData {
    pos: TilePos,
    block_type: Option<BlockType>,
    neighbors: Neighbors<BlockType>
}

fn block_gui(
    mut contexts: EguiContexts,
    block_data: Res<HoverBlockData>,
    type_registry: Res<AppTypeRegistry>,
) {
    let egui_context = contexts.ctx_mut();

    egui::Window::new("Hover Block Data")
        .anchor(Align2::RIGHT_BOTTOM, (-10., -10.))
        .resizable(false)
        .default_size((320., 200.))
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

fn block_hover(
    cursor: Res<CursorPosition<MainCamera>>,
    world_data: Res<WorldData>,
    mut block_data: ResMut<HoverBlockData>
) {
    let tile_pos = get_tile_pos_from_world_coords(cursor.world);
    let block_type = world_data.get_block(tile_pos).map(|b| *b);
    let neighbors = world_data.get_block_neighbors(tile_pos, true);

    block_data.pos = tile_pos;
    block_data.block_type = block_type.map(|b| b.block_type);
    block_data.neighbors = neighbors.map_ref(|b| b.block_type);
}