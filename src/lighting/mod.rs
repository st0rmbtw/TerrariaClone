use bevy::{
    render::{
        extract_resource::ExtractResourcePlugin, RenderApp, RenderStage, 
        render_graph::{RenderGraph, self}, 
        renderer::RenderContext, 
        render_resource::{PipelineCache, ComputePassDescriptor}
    }, 
    prelude::{Shader, Vec2, ResMut, Res, World, Plugin, App, IntoSystemDescriptor, default, EventReader},
    window::{Windows, WindowResized},
    asset::load_internal_asset, sprite::Material2dPlugin,
};
use iyes_loopless::prelude::IntoConditionalSystem;
use tracing::warn;

use crate::{plugins::{camera::UpdateLightEvent, world::WorldData}, lighting::{compositing::{PostProcessingMaterial, setup_post_processing_camera, update_image_to_window_size, update_lighting_material, update_light_map}, constants::{SHADER_GI_HALTON, SHADER_GI_ATTENUATION, SHADER_GI_MATH, SHADER_GI_RAYMARCH}}, state::GameState};

use self::{
    pipeline::{LightPassPipelineBindGroups, PipelineTargetsWrapper, system_setup_gi_pipeline, LightPassPipeline, system_queue_bind_groups}, 
    resource::{LightPassParams, ComputedTargetSizes}, 
    constants::{SHADER_GI_CAMERA, SHADER_GI_TYPES, GI_SCREEN_PROBE_SIZE}, 
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
            .add_event::<UpdateLightEvent>()
            .add_plugin(ExtractResourcePlugin::<PipelineTargetsWrapper>::default())
            .add_plugin(Material2dPlugin::<PostProcessingMaterial>::default())
            .init_resource::<PipelineTargetsWrapper>()
            .init_resource::<ComputedTargetSizes>()
            .insert_resource(LightPassParams {
                reservoir_size: 16,
                smooth_kernel_size: (2, 1),
                direct_light_contrib: 0.2,
                indirect_light_contrib: 0.8,
                ..default()
            })

            .add_startup_system(detect_target_sizes)
            .add_startup_system(system_setup_gi_pipeline.after(detect_target_sizes))
            .add_system(setup_post_processing_camera.run_if_resource_exists::<WorldData>())
            .add_system(update_image_to_window_size)
            .add_system(update_lighting_material.run_in_state(GameState::InGame))
            .add_system(update_light_map.run_in_state(GameState::InGame))
            .add_system(resize_primary_target);

        load_internal_asset!(
            app,
            SHADER_GI_CAMERA,
            "shaders/gi_camera.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER_GI_TYPES,
            "shaders/gi_types.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER_GI_HALTON,
            "shaders/gi_halton.wgsl",
            Shader::from_wgsl
        );

         load_internal_asset!(
            app,
            SHADER_GI_ATTENUATION,
            "shaders/gi_attenuation.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER_GI_MATH,
            "shaders/gi_math.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER_GI_RAYMARCH,
            "shaders/gi_raymarch.wgsl",
            Shader::from_wgsl
        );

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<LightPassPipeline>()
            .init_resource::<LightPassPipelineAssets>()
            .init_resource::<ComputedTargetSizes>()
            .add_system_to_stage(RenderStage::Extract, system_extract_pipeline_assets)
            .add_system_to_stage(RenderStage::Prepare, system_prepare_pipeline_assets)
            .add_system_to_stage(RenderStage::Queue, system_queue_bind_groups);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("light_pass_2d", LightPass2DNode::default());
        render_graph
            .add_node_edge(
                "light_pass_2d",
                bevy::render::main_graph::node::CAMERA_DRIVER,
            )
            .unwrap();
    }
}

#[derive(Default)]
struct LightPass2DNode {}

pub fn detect_target_sizes(
    windows: Res<Windows>,
    mut target_sizes: ResMut<ComputedTargetSizes>
) {

    let window       = windows.get_primary().expect("No primary window");
    let primary_size = Vec2::new(
        (window.physical_width()  as f64 / window.backend_scale_factor()) as f32,
        (window.physical_height() as f64 / window.backend_scale_factor()) as f32,
    );

    target_sizes.primary_target_size  = primary_size;
}

pub fn resize_primary_target(
    mut resize_events: EventReader<WindowResized>,
    mut target_sizes: ResMut<ComputedTargetSizes>
) {
    for resize_event in resize_events.iter() {
        let size = Vec2::new(resize_event.width, resize_event.height);

        target_sizes.primary_target_size = size;
    }
}

impl render_graph::Node for LightPass2DNode {
    fn update(&mut self, _world: &mut World) {}

    #[rustfmt::skip]
    fn run(
        &self,
        _:              &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world:          &World,
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
                        .command_encoder
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some("light_pass_2d".into()),
                        });

                {
                    let grid_w = (primary_w / GI_SCREEN_PROBE_SIZE as u32) / WORKGROUP_SIZE;
                    let grid_h = (primary_h / GI_SCREEN_PROBE_SIZE as u32) / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.lighting_bind_group, &[]);
                    pass.set_pipeline(&lighting_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }
            }
        } else {
            warn!("Failed to get bind groups");
        }

        Ok(())
    }
}