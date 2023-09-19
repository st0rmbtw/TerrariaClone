use bevy::{render::{render_graph::{ViewNode, RenderGraphContext, NodeRunError}, renderer::RenderContext, render_resource::{PipelineCache, RenderPassDescriptor, RenderPassColorAttachment, Operations}, view::ViewTarget}, ecs::query::QueryItem, prelude::World};

use self::pipeline::{PostProcessPipelineBindGroups, PostProcessPipeline};

use super::PostProcessCamera;

pub(super) mod pipeline;
pub(super) mod assets;

#[derive(Default)]
pub(super) struct PostProcessNode;
impl PostProcessNode {
    pub(super) const NAME: &str = "post_process";
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
        view_query: QueryItem<Self::ViewQuery>,
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