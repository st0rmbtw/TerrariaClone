use bevy::{render::{render_resource::{BindGroupLayout, CachedRenderPipelineId, PipelineCache, RenderPipelineDescriptor, PrimitiveState, MultisampleState, FragmentState, ColorTargetState, ColorWrites, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, BufferBindingType, ShaderType, TextureSampleType, TextureViewDimension, SamplerBindingType, BindGroupDescriptor, BindGroupEntry, BindingResource, BindGroup, ShaderDefVal}, render_asset::RenderAssets, renderer::RenderDevice, texture::BevyDefault}, prelude::{Commands, Image, Res, Resource, FromWorld, World, AssetServer}, core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state};

use crate::{lighting::{BackgroundTexture, InGameBackgroundTexture, WorldTexture, LightMapTexture, gpu_types::GpuCameraParams}, plugins::world::WorldSize};

use super::assets::PostProcessPipelineAssets;

#[derive(Resource)]
pub(crate) struct PostProcessPipeline {
    pub(crate) layout: BindGroupLayout,
    pub(crate) pipeline: CachedRenderPipelineId
}

#[derive(Resource)]
pub(crate) struct PostProcessPipelineBindGroups(pub(crate) BindGroup);


pub(crate) fn queue_postprocess_bind_groups(
    mut commands: Commands,
    pipeline: Res<PostProcessPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    render_device: Res<RenderDevice>,
    pipeline_assets: Res<PostProcessPipelineAssets>,

    background_texture: Res<BackgroundTexture>,
    ingame_background_texture: Res<InGameBackgroundTexture>,
    world_texture: Res<WorldTexture>,
    lightmap_texture: Res<LightMapTexture>,
) {
    if let Some(camera_params) = pipeline_assets.camera_params.binding() {
        let background_image = &gpu_images[&background_texture.0];
        let ingame_background_image = &gpu_images[&ingame_background_texture.0];
        let world_image = &gpu_images[&world_texture.0];
        let lightmap_image = &gpu_images[&lightmap_texture.0];

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "post_process_bind_group".into(),
            layout: &pipeline.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&background_image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&background_image.sampler),
                },

                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&ingame_background_image.texture_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&ingame_background_image.sampler),
                },

                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&world_image.texture_view),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::Sampler(&world_image.sampler),
                },

                BindGroupEntry {
                    binding: 8,
                    resource: BindingResource::TextureView(&lightmap_image.texture_view),
                },
                BindGroupEntry {
                    binding: 9,
                    resource: BindingResource::Sampler(&lightmap_image.sampler),
                },
                
                BindGroupEntry {
                    binding: 10,
                    resource: camera_params
                },
            ],
        });

        commands.insert_resource(PostProcessPipelineBindGroups(bind_group));
    }
}

impl FromWorld for PostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let world_size = *world.resource::<WorldSize>();

        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("post_process_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },

                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },

                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 5,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },

                    BindGroupLayoutEntry {
                        binding: 8,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 9,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },

                    BindGroupLayoutEntry {
                        binding: 10,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(GpuCameraParams::min_size())
                        },
                        count: None,
                    },
                ],
            });

        let assets_server = world.resource::<AssetServer>();
        let shader = assets_server.load("shaders/post_processing.wgsl");

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let pipeline = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("post_process_pipeline".into()),
            layout: vec![layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader,
                shader_defs: vec![
                    ShaderDefVal::UInt("WORLD_WIDTH".into(), world_size.x),
                    ShaderDefVal::UInt("WORLD_HEIGHT".into(), world_size.y),
                ],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: BevyDefault::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL
                })],
            }),
            push_constant_ranges: vec![],
            primitive: PrimitiveState::default(),
            multisample: MultisampleState::default(),
            depth_stencil: None,
        });

        PostProcessPipeline {
            layout,
            pipeline,
        }
    }
}