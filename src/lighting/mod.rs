use bevy::{
    render::{
        extract_resource::ExtractResourcePlugin, RenderApp,
        render_graph::{RenderGraph, self}, 
        renderer::RenderContext, 
        render_resource::{PipelineCache, ComputePassDescriptor, Extent3d}, ExtractSchedule, RenderSet
    }, 
    prelude::{Shader, Vec2, ResMut, Res, World, Plugin, App, default, EventReader, Assets, Image, warn, IntoSystemConfig, resource_exists, Query, OnUpdate, With, IntoSystemAppConfig},
    window::{WindowResized, Window, PrimaryWindow},
    asset::load_internal_asset, sprite::Material2dPlugin,
};

use crate::{plugins::world::WorldData, lighting::{compositing::{PostProcessingMaterial, setup_post_processing_camera, update_image_to_window_size, update_lighting_material, update_light_map}, constants::{SHADER_HALTON, SHADER_ATTENUATION, SHADER_MATH}}, state::GameState};

use self::{
    pipeline::{LightPassPipelineBindGroups, PipelineTargetsWrapper, system_setup_pipeline, LightPassPipeline, system_queue_bind_groups}, 
    resource::{LightPassParams, ComputedTargetSizes}, 
    constants::{SHADER_CAMERA, SHADER_TYPES, SCREEN_PROBE_SIZE}, 
    pipeline_assets::{LightPassPipelineAssets, system_extract_pipeline_assets, system_prepare_pipeline_assets}
};

pub mod resource;
pub mod types;
pub mod types_gpu;
pub mod pipeline;
pub mod pipeline_assets;
pub mod constants;
pub mod compositing;

const WORKGROUP_SIZE: u32 = 8;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(ExtractResourcePlugin::<PipelineTargetsWrapper>::default())
            .add_plugin(Material2dPlugin::<PostProcessingMaterial>::default())
            
            .init_resource::<PipelineTargetsWrapper>()
            .init_resource::<ComputedTargetSizes>()
            .insert_resource(LightPassParams {
                reservoir_size: 16
            })

            .add_startup_system(detect_target_sizes)
            .add_startup_system(system_setup_pipeline.after(detect_target_sizes))

            .add_system(setup_post_processing_camera.run_if(resource_exists::<WorldData>()))
            .add_system(update_image_to_window_size)
            .add_system(update_lighting_material.in_set(OnUpdate(GameState::InGame)))
            .add_system(update_light_map.in_set(OnUpdate(GameState::InGame)))
            .add_system(resize_primary_target);

        load_internal_asset!(
            app,
            SHADER_CAMERA,
            "shaders/camera.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER_TYPES,
            "shaders/types.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER_HALTON,
            "shaders/halton.wgsl",
            Shader::from_wgsl
        );

         load_internal_asset!(
            app,
            SHADER_ATTENUATION,
            "shaders/attenuation.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER_MATH,
            "shaders/math.wgsl",
            Shader::from_wgsl
        );

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<LightPassPipeline>()
            .init_resource::<LightPassPipelineAssets>()
            .init_resource::<ComputedTargetSizes>()
            .add_system(system_extract_pipeline_assets.in_schedule(ExtractSchedule))
            .add_system(system_prepare_pipeline_assets.in_set(RenderSet::Prepare))
            .add_system(system_queue_bind_groups.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("light_pass_2d", LightPass2DNode::default());
        render_graph
            .add_node_edge(
                "light_pass_2d",
                bevy::render::main_graph::node::CAMERA_DRIVER,
            );
    }
}

#[derive(Default)]
struct LightPass2DNode {}

pub fn detect_target_sizes(
    query_windows: Query<&Window, With<PrimaryWindow>>,
    mut target_sizes: ResMut<ComputedTargetSizes>
) {

    let window = query_windows.get_single().expect("No primary window");
    let primary_size = Vec2::new(
        window.width(),
        window.height()
    );
    
    target_sizes.primary_target_size = primary_size;
}

pub fn resize_primary_target(
    query_windows: Query<&Window, With<PrimaryWindow>>,
    mut resize_events: EventReader<WindowResized>,
    mut target_sizes: ResMut<ComputedTargetSizes>,
    mut images: ResMut<Assets<Image>>,
    targets_wrapper: Res<PipelineTargetsWrapper>,
) {
    let window = query_windows.get_single().expect("No primary window");

    for _ in resize_events.iter() {
        target_sizes.primary_target_size = Vec2::new(
            window.width(),
            window.height()
        );

        if let Some(targets) = targets_wrapper.targets.as_ref() {
            let size = target_sizes.primary_target_usize();

            let extent = Extent3d {
                width: size.x,
                height: size.y,
                ..default()  
            };

            images.get_mut(&targets.lighting_target.clone_weak())
                .unwrap()
                .resize(extent);
        }
    }
}

impl render_graph::Node for LightPass2DNode {
    fn update(&mut self, _world: &mut World) {}

    fn run(
        &self,
        _: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {

        if let Some(pipeline_bind_groups) = world.get_resource::<LightPassPipelineBindGroups>() {

            let pipeline_cache  = world.resource::<PipelineCache>();
            let pipeline        = world.resource::<LightPassPipeline>();
            let target_sizes    = world.resource::<ComputedTargetSizes>();

            if let Some(lighting_pipeline) = 
                pipeline_cache.get_compute_pipeline(pipeline.lighting_pipeline) 
            {
                let primary_w = target_sizes.primary_target_usize().x;
                let primary_h = target_sizes.primary_target_usize().y;

                let mut pass =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some("light_pass_2d"),
                        });

                {
                    let grid_w = (primary_w / SCREEN_PROBE_SIZE as u32) / WORKGROUP_SIZE;
                    let grid_h = (primary_h / SCREEN_PROBE_SIZE as u32) / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.lighting_bind_group, &[]);
                    pass.set_pipeline(lighting_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }
            }
        } else {
            warn!("Failed to get bind groups");
        }

        Ok(())
    }
}
