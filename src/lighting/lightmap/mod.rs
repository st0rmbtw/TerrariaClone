use bevy::{render::{render_graph::{RenderGraphContext, Node, NodeRunError}, renderer::RenderContext, render_resource::{PipelineCache, ComputePassDescriptor}}, prelude::World};

use self::{pipeline::{LightMapPipelineBindGroups, LightMapPipeline}, assets::{BlurArea, LightSourceCount}};

use super::WORKGROUP;

pub(super) mod pipeline;
pub(super) mod assets;

pub(super) struct LightMapNode;
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

                    // Light sources
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