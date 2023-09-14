use bevy::prelude::{Plugin, App, Update, IntoSystemConfigs, OnEnter, World, OnExit, PostUpdate, Event, in_state, Handle, Image, Resource, Deref, not, resource_changed, State, Condition};
use bevy::render::extract_resource::{ExtractResourcePlugin, ExtractResource};
use bevy::render::render_graph::{RenderGraph, Node, RenderGraphContext, NodeRunError};
use bevy::render::render_resource::{PipelineCache, ComputePassDescriptor};
use bevy::render::renderer::RenderContext;
use bevy::render::{RenderApp, Render, RenderSet, ExtractSchedule};
use bevy::sprite::Material2dPlugin;
use crate::common::state::GameState;
use crate::plugins::InGameSystemSet;

use self::compositing::{LightMapMaterial, PostProcessingMaterial};
use self::pipeline::{LightMapPipeline, PipelineBindGroups};
use self::pipeline_assets::{BlurArea, PipelineAssets};

pub(crate) mod compositing;
pub(super) mod pipeline;
pub(super) mod pipeline_assets;

const WORKGROUP: u32 = 16;

#[derive(Event, Clone, Copy)]
pub(crate) struct UpdateTilesTextureEvent {
    pub(crate) x: usize,
    pub(crate) y: usize
}

#[derive(Resource, ExtractResource, Clone)]
struct TileTexture(Handle<Image>);

#[derive(Resource, ExtractResource, Clone, Deref)]
pub(crate) struct LightMapTexture(Handle<Image>);

pub(crate) struct LightingPlugin;
impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<LightMapMaterial>::default(),
            Material2dPlugin::<PostProcessingMaterial>::default(),
            ExtractResourcePlugin::<BlurArea>::default(),
        ));

        app.init_resource::<BlurArea>();
        app.add_event::<UpdateTilesTextureEvent>();

        app.add_systems(
            OnExit(GameState::WorldLoading),
            (
                pipeline_assets::init_tiles_texture,
                pipeline::init_light_map_texture,
            )
        );

        app.add_systems(OnEnter(GameState::InGame), compositing::setup_post_processing_camera);

        app.add_systems(
            Update,
            (
                compositing::update_image_to_window_size,
                pipeline_assets::handle_update_tiles_texture_event.in_set(InGameSystemSet::Update)
            )
        );
        app.add_systems(PostUpdate, pipeline_assets::update_blur_area.in_set(InGameSystemSet::PostUpdate));

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_systems(
                ExtractSchedule,
                (
                    pipeline_assets::extract_state,
                    pipeline_assets::extract_pipeline_assets,
                    pipeline_assets::extract_textures,
                    pipeline_assets::extract_world_underground_level,
                    pipeline_assets::extract_light_smoothness
                )
            )
            .add_systems(
                Render,
                (
                    pipeline::init_pipeline
                        .in_set(RenderSet::Prepare)
                        .run_if(resource_changed::<State<GameState>>().and_then(
                                in_state(GameState::InGame))
                        ),
                    pipeline_assets::prepare_pipeline_assets.in_set(RenderSet::Prepare),
                    pipeline::queue_bind_groups
                        .run_if(in_state(GameState::InGame))
                        .in_set(RenderSet::Queue),
                    pipeline::remove_pipeline
                        .run_if(resource_changed::<State<GameState>>().and_then(
                                not(in_state(GameState::InGame)))
                        )
                        .in_set(RenderSet::Cleanup)
                ),
            );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("light_map", LightMapNode);
        render_graph.add_node_edge(
            "light_map",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        )
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<PipelineAssets>();
    }
}

struct LightMapNode;
impl Node for LightMapNode {
    fn run(
        &self,
        _: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if let Some(pipeline_bind_groups) = world.get_resource::<PipelineBindGroups>() {
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<LightMapPipeline>();
            let blur_area = world.resource::<BlurArea>();

            if blur_area.size().x > 0 && blur_area.size().y > 0 {
                if let (
                    Some(scan_pipeline),
                    Some(left_to_right_pipeline),
                    Some(right_to_left_pipeline),
                    Some(top_to_bottom_pipeline),
                    Some(bottom_to_top_pipeline),
                ) = (
                    pipeline_cache.get_compute_pipeline(pipeline.scan_pipeline),
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