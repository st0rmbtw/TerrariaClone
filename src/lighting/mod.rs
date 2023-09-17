use bevy::core_pipeline::core_2d;
use bevy::prelude::{Plugin, App, Update, IntoSystemConfigs, OnEnter, World, OnExit, PostUpdate, Event, in_state, Handle, Image, Resource, Deref, not, Condition, Commands, state_changed, Component};
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::extract_resource::{ExtractResourcePlugin, ExtractResource};
use bevy::render::render_graph::{RenderGraph, Node, RenderGraphContext, NodeRunError, ViewNode, RenderGraphApp, ViewNodeRunner};
use bevy::render::render_resource::{PipelineCache, ComputePassDescriptor, RenderPassDescriptor, RenderPassColorAttachment, Operations};
use bevy::render::renderer::RenderContext;
use bevy::render::view::ViewTarget;
use bevy::render::{RenderApp, Render, RenderSet, ExtractSchedule};
use crate::common::state::GameState;
use crate::plugins::InGameSystemSet;

use self::lightmap_pipeline::{LightMapPipeline, LightMapPipelineBindGroups};
use self::pipeline_assets::{BlurArea, LightMapPipelineAssets, LightSourceCount, PostProcessPipelineAssets};
use self::postprocess_pipeline::{PostProcessPipeline, PostProcessPipelineBindGroups};

pub(crate) mod compositing;
pub(super) mod lightmap_pipeline;
pub(super) mod postprocess_pipeline;
pub(super) mod pipeline_assets;
pub(super) mod types;
pub(super) mod gpu_types;

const WORKGROUP: u32 = 16;

#[derive(Event, Clone, Copy)]
pub(crate) struct UpdateTilesTextureEvent {
    pub(crate) x: usize,
    pub(crate) y: usize
}

#[derive(Resource, ExtractResource, Clone)]
struct BackgroundTexture(Handle<Image>);

#[derive(Resource, ExtractResource, Clone)]
struct InGameBackgroundTexture(Handle<Image>);

#[derive(Resource, ExtractResource, Clone)]
struct WorldTexture(Handle<Image>);

#[derive(Resource, ExtractResource, Clone)]
struct MainTexture(Handle<Image>);

#[derive(Resource, ExtractResource, Clone)]
struct TileTexture(Handle<Image>);

#[derive(Resource, ExtractResource, Clone, Deref)]
struct LightMapTexture(Handle<Image>);

#[derive(Component, ExtractComponent, Clone)]
struct PostProcessCamera;

pub(crate) struct LightingPlugin;
impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<BlurArea>::default(),
            ExtractComponentPlugin::<PostProcessCamera>::default()
        ));

        app.init_resource::<BlurArea>();
        app.add_event::<UpdateTilesTextureEvent>();

        app.add_systems(
            OnExit(GameState::WorldLoading),
            (
                pipeline_assets::init_tiles_texture,
                lightmap_pipeline::init_light_map_texture,
            )
        );

        app.add_systems(
            OnEnter(GameState::InGame),
            (
                compositing::setup_post_processing_camera,
                compositing::spawn_mouse_light
            )
        );

        app.add_systems(
            Update,
            (
                compositing::update_image_to_window_size,
                pipeline_assets::handle_update_tiles_texture_event,
                compositing::update_mouse_light
            ).in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            PostUpdate,
            (
                pipeline_assets::update_blur_area,
            )
            .in_set(InGameSystemSet::PostUpdate)
        );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_systems(
                ExtractSchedule,
                (
                    pipeline_assets::extract_textures,
                    pipeline_assets::extract_world_underground_level,
                    pipeline_assets::extract_state,
                    (
                        pipeline_assets::extract_light_smoothness,
                        pipeline_assets::extract_lightmap_pipeline_assets,
                    ).chain(),
                    pipeline_assets::extract_postprocess_pipeline_assets
                )
            )
            .add_systems(
                Render,
                (
                    (
                        init_pipeline
                            .run_if(state_changed::<GameState>().and_then(in_state(GameState::InGame))),
                        pipeline_assets::prepare_lightmap_pipeline_assets,
                        pipeline_assets::prepare_postprocess_pipeline_assets,
                    ).in_set(RenderSet::Prepare),

                    (
                        lightmap_pipeline::queue_lightmap_bind_groups,
                        postprocess_pipeline::queue_postprocess_bind_groups
                    )
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
                core_2d::graph::node::FXAA,
                PostProcessNode::NAME,
                core_2d::graph::node::END_MAIN_PASS_POST_PROCESSING,
            ],
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
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

struct LightMapNode;
impl Node for LightMapNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if let Some(pipeline_bind_groups) = world.get_resource::<LightMapPipelineBindGroups>() {
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<LightMapPipeline>();
            let blur_area = world.resource::<BlurArea>();
            let light_source_count = *world.resource::<LightSourceCount>();

            if blur_area.width() > 0 && blur_area.height() > 0 {
                if let (
                    Some(scan_pipeline),
                    Some(place_light_pipeline),
                    Some(left_to_right_pipeline),
                    Some(right_to_left_pipeline),
                    Some(top_to_bottom_pipeline),
                    Some(bottom_to_top_pipeline),
                ) = (
                    pipeline_cache.get_compute_pipeline(pipeline.scan_pipeline),
                    pipeline_cache.get_compute_pipeline(pipeline.light_sources_pipeline),
                    pipeline_cache.get_compute_pipeline(pipeline.left_to_right_pipeline),
                    pipeline_cache.get_compute_pipeline(pipeline.right_to_left_pipeline),
                    pipeline_cache.get_compute_pipeline(pipeline.top_to_bottom_pipeline),
                    pipeline_cache.get_compute_pipeline(pipeline.bottom_to_top_pipeline),
                ) {
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some("light_map"),
                        });

                    let grid_w = blur_area.width() / WORKGROUP;
                    let grid_h = blur_area.height() / WORKGROUP;

                    // Scan
                    pass.set_bind_group(0, &pipeline_bind_groups.scan_bind_group, &[]);
                    pass.set_pipeline(scan_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);

                    // Place light
                    pass.set_bind_group(0, &pipeline_bind_groups.light_sources_bind_group, &[]);
                    pass.set_pipeline(place_light_pipeline);
                    pass.dispatch_workgroups(*light_source_count, 1, 1);
                    
                    // First blur pass
                    pass.set_bind_group(0, &pipeline_bind_groups.top_to_bottom_bind_group, &[]);
                    pass.set_pipeline(top_to_bottom_pipeline);
                    pass.dispatch_workgroups(grid_w, 1, 1);

                    pass.set_bind_group(0, &pipeline_bind_groups.left_to_right_bind_group, &[]);
                    pass.set_pipeline(left_to_right_pipeline);
                    pass.dispatch_workgroups(1, grid_h, 1);

                    pass.set_bind_group(0, &pipeline_bind_groups.bottom_to_top_bind_group, &[]);
                    pass.set_pipeline(bottom_to_top_pipeline);
                    pass.dispatch_workgroups(grid_w, 1, 1);

                    pass.set_bind_group(0, &pipeline_bind_groups.right_to_left_bind_group, &[]);
                    pass.set_pipeline(right_to_left_pipeline);
                    pass.dispatch_workgroups(1, grid_h, 1);


                    // Second blur pass
                    pass.set_bind_group(0, &pipeline_bind_groups.top_to_bottom_bind_group, &[]);
                    pass.set_pipeline(top_to_bottom_pipeline);
                    pass.dispatch_workgroups(grid_w, 1, 1);

                    pass.set_bind_group(0, &pipeline_bind_groups.left_to_right_bind_group, &[]);
                    pass.set_pipeline(left_to_right_pipeline);
                    pass.dispatch_workgroups(1, grid_h, 1);

                    pass.set_bind_group(0, &pipeline_bind_groups.bottom_to_top_bind_group, &[]);
                    pass.set_pipeline(bottom_to_top_pipeline);
                    pass.dispatch_workgroups(grid_w, 1, 1);

                    pass.set_bind_group(0, &pipeline_bind_groups.right_to_left_bind_group, &[]);
                    pass.set_pipeline(right_to_left_pipeline);
                    pass.dispatch_workgroups(1, grid_h, 1);


                    // Third blur pass
                    pass.set_bind_group(0, &pipeline_bind_groups.top_to_bottom_bind_group, &[]);
                    pass.set_pipeline(top_to_bottom_pipeline);
                    pass.dispatch_workgroups(grid_w, 1, 1);

                    pass.set_bind_group(0, &pipeline_bind_groups.left_to_right_bind_group, &[]);
                    pass.set_pipeline(left_to_right_pipeline);
                    pass.dispatch_workgroups(1, grid_h, 1);

                    pass.set_bind_group(0, &pipeline_bind_groups.bottom_to_top_bind_group, &[]);
                    pass.set_pipeline(bottom_to_top_pipeline);
                    pass.dispatch_workgroups(grid_w, 1, 1);

                    pass.set_bind_group(0, &pipeline_bind_groups.right_to_left_bind_group, &[]);
                    pass.set_pipeline(right_to_left_pipeline);
                    pass.dispatch_workgroups(1, grid_h, 1);
                }
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct PostProcessNode;
impl PostProcessNode {
    const NAME: &str = "post_process";
}

impl ViewNode for PostProcessNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static PostProcessCamera
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        view_query: bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if let Some(bind_groups) = world.get_resource::<PostProcessPipelineBindGroups>() {
            let (view_target, _) = view_query;

            let post_process_pipeline = world.resource::<PostProcessPipeline>();

            let pipeline_cache = world.resource::<PipelineCache>();

            let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline) else {
                return Ok(());
            };

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("post_process_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: view_target.main_texture_view(),
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_render_pipeline(pipeline);
            render_pass.set_bind_group(0, &bind_groups.0, &[]);
            render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}