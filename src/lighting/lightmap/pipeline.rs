use bevy::{prelude::{Image, Commands, Res, World, FromWorld, AssetServer, Resource, UVec2, Vec3}, render::{render_resource::{BindGroup, BindGroupLayout, CachedComputePipelineId, BindGroupDescriptor, BindGroupEntry, BindingResource, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, StorageTextureAccess, TextureViewDimension, PipelineCache, ComputePipelineDescriptor, BufferBindingType, ShaderType, ShaderDefVal}, renderer::RenderDevice, render_asset::RenderAssets}};

use crate::{plugins::{config::LightSmoothness, world::resources::WorldUndergroundLevel}, lighting::{LightMapTexture, TileTexture, LIGHTMAP_FORMAT, TILES_FORMAT, gpu_types::GpuLightSourceBuffer}};

use super::assets::LightMapPipelineAssets;

#[derive(Resource)]
pub(crate) struct LightMapPipelineBindGroups {
    pub(crate) scan_bind_group: BindGroup,
    pub(crate) light_sources_bind_group: BindGroup,
    pub(crate) left_to_right_bind_group: BindGroup,
    pub(crate) top_to_bottom_bind_group: BindGroup,
    pub(crate) right_to_left_bind_group: BindGroup,
    pub(crate) bottom_to_top_bind_group: BindGroup,
}

#[derive(Resource)]
pub(crate) struct LightMapPipeline {
    pub(crate) scan_layout: BindGroupLayout,
    pub(crate) scan_pipeline: CachedComputePipelineId,

    pub(crate) light_sources_layout: BindGroupLayout,
    pub(crate) light_sources_pipeline: CachedComputePipelineId,

    pub(crate) left_to_right_layout: BindGroupLayout,
    pub(crate) left_to_right_pipeline: CachedComputePipelineId,

    pub(crate) top_to_bottom_layout: BindGroupLayout,
    pub(crate) top_to_bottom_pipeline: CachedComputePipelineId,

    pub(crate) right_to_left_layout: BindGroupLayout,
    pub(crate) right_to_left_pipeline: CachedComputePipelineId,

    pub(crate) bottom_to_top_layout: BindGroupLayout,
    pub(crate) bottom_to_top_pipeline: CachedComputePipelineId,
}

pub(crate) fn queue_lightmap_bind_groups(
    mut commands: Commands,
    pipeline: Res<LightMapPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    render_device: Res<RenderDevice>,
    pipeline_assets: Res<LightMapPipelineAssets>,
    tile_texture: Res<TileTexture>,
    lightmap_texture: Res<LightMapTexture>,
) {
    if let (
        Some(area_min),
        Some(area_max),
        Some(sky_color),
        Some(light_sources),
    ) = (
        pipeline_assets.area_min.binding(),
        pipeline_assets.area_max.binding(),
        pipeline_assets.ambient_color.binding(),
        pipeline_assets.light_sources.binding(),
    ) {
        let tiles_image = &gpu_images[&tile_texture.0];
        let lightmap_image = &gpu_images[&lightmap_texture.0];

        let scan_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "scan_bind_group".into(),
            layout: &pipeline.scan_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&tiles_image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&lightmap_image.texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: area_min.clone()
                },
                BindGroupEntry {
                    binding: 3,
                    resource: sky_color.clone()
                },
            ],
        });

        let light_sources_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "light_sources_bind_group".into(),
            layout: &pipeline.light_sources_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&lightmap_image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: light_sources,
                },
            ],
        });

        let left_to_right_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "left_to_right_bind_group".into(),
            layout: &pipeline.left_to_right_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&tiles_image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&lightmap_image.texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: area_min.clone()
                },
                BindGroupEntry {
                    binding: 3,
                    resource: area_max.clone()
                },
            ],
        });

        let top_to_bottom_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "top_to_bottom_bind_group".into(),
            layout: &pipeline.top_to_bottom_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&tiles_image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&lightmap_image.texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: area_min.clone()
                },
                BindGroupEntry {
                    binding: 3,
                    resource: area_max.clone()
                },
            ],
        });

        let right_to_left_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "right_to_left_bind_group".into(),
            layout: &pipeline.right_to_left_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&tiles_image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&lightmap_image.texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: area_min.clone()
                },
                BindGroupEntry {
                    binding: 3,
                    resource: area_max.clone()
                },
            ],
        });

        let bottom_to_top_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "bottom_to_top_bind_group".into(),
            layout: &pipeline.bottom_to_top_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&tiles_image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&lightmap_image.texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: area_min
                },
                BindGroupEntry {
                    binding: 3,
                    resource: area_max
                },
            ],
        });

        commands.insert_resource(LightMapPipelineBindGroups {
            scan_bind_group,
            light_sources_bind_group,
            left_to_right_bind_group,
            top_to_bottom_bind_group,
            right_to_left_bind_group,
            bottom_to_top_bind_group,
        });
    }
}

impl FromWorld for LightMapPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let light_smoothness = world.resource::<LightSmoothness>();
        let underground_level = world.resource::<WorldUndergroundLevel>().0;
        
        let subdivision = light_smoothness.subdivision();
        let light_smoothness = light_smoothness.to_u8() as u32;

        let scan_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("scan_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TILES_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: LIGHTMAP_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(UVec2::min_size())
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(Vec3::min_size())
                        },
                        count: None,
                    },
                ],
            });

        let light_sources_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("light_sources_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: LIGHTMAP_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(GpuLightSourceBuffer::min_size())
                        },
                        count: None,
                    },
                ],
            });

        let left_to_right_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("left_to_right_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TILES_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },

                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: LIGHTMAP_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(UVec2::min_size())
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(UVec2::min_size())
                        },
                        count: None,
                    },
                ],
            });

        let top_to_bottom_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("top_to_bottom_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TILES_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: LIGHTMAP_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(UVec2::min_size())
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(UVec2::min_size())
                        },
                        count: None,
                    },
                ],
            });

        let right_to_left_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("right_to_left_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TILES_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },

                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: LIGHTMAP_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(UVec2::min_size())
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(UVec2::min_size())
                        },
                        count: None,
                    },
                ],
            });

        let bottom_to_top_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("bottom_to_top_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TILES_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },

                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: LIGHTMAP_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(UVec2::min_size())
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(UVec2::min_size())
                        },
                        count: None,
                    },
                ],
            });


        let (
            shader_scan,
            shader_light_sources,
            shader_blur
        ) = {
            let assets_server = world.resource::<AssetServer>();
            (
                assets_server.load("shaders/light_map/scan.wgsl"),
                assets_server.load("shaders/light_map/light_sources.wgsl"),
                assets_server.load("shaders/light_map/blur.wgsl"),
            )
        };

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let scan_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("scan_pipeline".into()),
            layout: vec![scan_layout.clone()],
            shader: shader_scan,
            shader_defs: vec![
                ShaderDefVal::UInt("WORLD_UNDERGROUND_LEVEL".into(), underground_level),
                ShaderDefVal::UInt("SUBDIVISION".into(), subdivision),
            ],
            entry_point: "scan".into(),
            push_constant_ranges: vec![],
        });

        let light_sources_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("light_sources_pipeline".into()),
            layout: vec![light_sources_layout.clone()],
            shader: shader_light_sources,
            shader_defs: vec![],
            entry_point: "light_sources".into(),
            push_constant_ranges: vec![],
        });

        let left_to_right_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("left_to_right_pipeline".into()),
            layout: vec![left_to_right_layout.clone()],
            shader: shader_blur.clone(),
            shader_defs: vec![
                ShaderDefVal::UInt("LIGHT_SMOOTHNESS".into(), light_smoothness),
                ShaderDefVal::UInt("SUBDIVISION".into(), subdivision),
            ],
            entry_point: "left_to_right".into(),
            push_constant_ranges: vec![],
        });

        let top_to_bottom_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("top_to_bottom_pipeline".into()),
            layout: vec![top_to_bottom_layout.clone()],
            shader: shader_blur.clone(),
            shader_defs: vec![
                ShaderDefVal::UInt("LIGHT_SMOOTHNESS".into(), light_smoothness),
                ShaderDefVal::UInt("SUBDIVISION".into(), subdivision),
            ],
            entry_point: "top_to_bottom".into(),
            push_constant_ranges: vec![],
        });

        let right_to_left_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("right_to_left_pipeline".into()),
            layout: vec![right_to_left_layout.clone()],
            shader: shader_blur.clone(),
            shader_defs: vec![
                ShaderDefVal::UInt("LIGHT_SMOOTHNESS".into(), light_smoothness),
                ShaderDefVal::UInt("SUBDIVISION".into(), subdivision),
            ],
            entry_point: "right_to_left".into(),
            push_constant_ranges: vec![],
        });

        let bottom_to_top_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("bottom_to_top_pipeline".into()),
            layout: vec![bottom_to_top_layout.clone()],
            shader: shader_blur,
            shader_defs: vec![
                ShaderDefVal::UInt("LIGHT_SMOOTHNESS".into(), light_smoothness),
                ShaderDefVal::UInt("SUBDIVISION".into(), subdivision),
            ],
            entry_point: "bottom_to_top".into(),
            push_constant_ranges: vec![],
        });

        LightMapPipeline {
            scan_layout,
            scan_pipeline,

            light_sources_layout,
            light_sources_pipeline,

            left_to_right_layout,
            left_to_right_pipeline,

            top_to_bottom_layout,
            top_to_bottom_pipeline,

            right_to_left_layout,
            right_to_left_pipeline,

            bottom_to_top_layout,
            bottom_to_top_pipeline,
        }
    }
}