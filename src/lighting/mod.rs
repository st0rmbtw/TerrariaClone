use bevy::core_pipeline::core_2d;
use bevy::prelude::{Plugin, App, Update, IntoSystemConfigs, OnEnter, OnExit, PostUpdate, in_state, Handle, Image, Resource, Deref, not, Condition, Commands, state_changed, Component, on_event, ResMut, EventReader, Query, With, resource_equals};
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_graph::{RenderGraph, RenderGraphApp, ViewNodeRunner};
use bevy::render::render_resource::TextureFormat;
use bevy::render::{RenderApp, Render, RenderSet, ExtractSchedule};
use bevy::transform::TransformSystem;
use bevy::window::{WindowResized, PrimaryWindow};
use crate::common::state::GameState;
use crate::plugins::InGameSystemSet;
use crate::plugins::world::WorldSize;
use crate::plugins::world::events::{BreakTileEvent, PlaceTileEvent};
use crate::plugins::world::resources::WorldUndergroundLevel;

use self::lightmap::LightMapNode;
use self::lightmap::assets::{BlurArea, LightMapPipelineAssets, LightSourceCount};
use self::lightmap::pipeline::{LightMapPipeline, LightMapPipelineBindGroups};
use self::postprocess::PostProcessNode;
use self::postprocess::assets::PostProcessPipelineAssets;
use self::postprocess::pipeline::{PostProcessPipeline, PostProcessPipelineBindGroups};

pub(crate) mod compositing;
pub(crate) mod extract;
pub(super) mod types;
pub(super) mod gpu_types;
pub(super) mod lightmap;
pub(super) mod postprocess;

#[derive(Resource, ExtractResource, Clone)]
pub(crate) struct BackgroundTexture(Handle<Image>);

#[derive(Resource, ExtractResource, Clone)]
pub(crate) struct InGameBackgroundTexture(Handle<Image>);

#[derive(Resource, ExtractResource, Clone)]
pub(crate) struct WorldTexture(Handle<Image>);

#[derive(Resource, ExtractResource, Clone, Deref)]
pub(crate) struct TileTexture(Handle<Image>);

#[derive(Resource, ExtractResource, Clone, Deref)]
pub(crate) struct LightMapTexture(Handle<Image>);

#[derive(Resource, Clone, Copy, Deref, PartialEq, Eq)]
pub(crate) struct DoLighting(pub(crate) bool);

#[derive(Component, ExtractComponent, Clone)]
struct PostProcessCamera;

const WORKGROUP: u32 = 16;
const LIGHTMAP_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
const TILES_FORMAT: TextureFormat = TextureFormat::R8Uint;

pub(crate) struct LightingPlugin;
impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<PostProcessCamera>::default());

        app.init_resource::<BlurArea>();
        app.insert_resource(DoLighting(true));

        app.add_systems(
            OnExit(GameState::WorldLoading),
            (
                lightmap::assets::init_tiles_texture,
                lightmap::assets::init_light_map_texture
            )
        );

        app.add_systems(OnEnter(GameState::InGame), compositing::setup_post_processing_camera);

        app.add_systems(
            Update,
            (
                toggle_do_lighting.run_if(on_event::<WindowResized>()),
                (
                    lightmap::assets::handle_update_tiles_texture_event
                        .run_if(on_event::<BreakTileEvent>().or_else(on_event::<PlaceTileEvent>())),
                    compositing::update_image_to_window_size,
                ).in_set(InGameSystemSet::Update)
            )
        );

        app.add_systems(
            PostUpdate,
            lightmap::assets::update_blur_area
                .in_set(InGameSystemSet::PostUpdate)
                .after(TransformSystem::TransformPropagate)
        );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_systems(
                ExtractSchedule,
                (
                    extract::extract_textures,
                    extract::extract_resource::<WorldUndergroundLevel>,
                    extract::extract_resource::<WorldSize>,
                    extract::extract_resource::<DoLighting>,
                    extract::extract_state,
                    (
                        extract::extract_light_smoothness,
                        extract::extract_blur_area,
                        lightmap::assets::extract_lightmap_pipeline_assets,
                    ).chain(),
                    postprocess::assets::extract_postprocess_pipeline_assets
                )
            )
            .add_systems(
                Render,
                (
                    (
                        init_pipeline.run_if(state_changed::<GameState>().and_then(in_state(GameState::InGame))),
                        (
                            lightmap::assets::prepare_lightmap_pipeline_assets,
                            postprocess::assets::prepare_postprocess_pipeline_assets
                        )
                        .run_if(resource_equals(DoLighting(true))),
                    ).in_set(RenderSet::Prepare),

                    (
                        lightmap::pipeline::queue_lightmap_bind_groups,
                        postprocess::pipeline::queue_postprocess_bind_groups
                    )
                    .run_if(resource_equals(DoLighting(true)))
                    .run_if(in_state(GameState::InGame))
                    .in_set(RenderSet::Queue),

                    remove_pipeline
                        .run_if(state_changed::<GameState>().and_then(not(in_state(GameState::InGame))))
                        .in_set(RenderSet::Cleanup)
                ),
            );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("light_map", LightMapNode);
        render_graph.add_node_edge(
            "light_map",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );

        render_app.add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(
            core_2d::graph::NAME,
            PostProcessNode::NAME,
        );
        render_app.add_render_graph_edges(
            core_2d::graph::NAME,
            &[
                core_2d::graph::node::MAIN_PASS,
                PostProcessNode::NAME,
                core_2d::graph::node::END_MAIN_PASS_POST_PROCESSING,
            ],
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<BlurArea>();
        render_app.init_resource::<LightSourceCount>();
        render_app.init_resource::<LightMapPipelineAssets>();
        render_app.init_resource::<PostProcessPipelineAssets>();
    }
}

fn init_pipeline(mut commands: Commands) {
    commands.init_resource::<LightMapPipeline>();
    commands.init_resource::<PostProcessPipeline>();
}

fn remove_pipeline(mut commands: Commands) {
    commands.remove_resource::<LightMapPipeline>();
    commands.remove_resource::<LightMapPipelineBindGroups>();

    commands.remove_resource::<PostProcessPipeline>();
    commands.remove_resource::<PostProcessPipelineBindGroups>();
}

fn toggle_do_lighting(
    mut do_lighting: ResMut<DoLighting>,
    mut events: EventReader<WindowResized>,
    query_primary_window: Query<(), With<PrimaryWindow>>
) {
    let Some(event) = events.iter().last() else { return; };
    
    if query_primary_window.contains(event.window) {
        do_lighting.0 = event.width > 0. && event.height > 0.;
    }
}