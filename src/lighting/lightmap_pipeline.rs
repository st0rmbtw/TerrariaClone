use bevy::{prelude::{Image, ResMut, Assets, default, Commands, Res, World, FromWorld, AssetServer, Resource, UVec2}, render::{render_resource::{BindGroup, TextureFormat, FilterMode, Extent3d, TextureDimension, TextureUsages, SamplerDescriptor, BindGroupLayout, CachedComputePipelineId, BindGroupDescriptor, BindGroupEntry, BindingResource, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, StorageTextureAccess, TextureViewDimension, PipelineCache, ComputePipelineDescriptor, BufferBindingType, ShaderType, ShaderDefVal}, texture::ImageSampler, renderer::RenderDevice, render_asset::RenderAssets}};

use crate::{world::WorldData, plugins::{config::LightSmoothness, world::resources::WorldUndergroundLevel}};

use super::{pipeline_assets::LightMapPipelineAssets, LightMapTexture, TileTexture, gpu_types::GpuLightSourceBuffer};

pub(super) const LIGHTMAP_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
pub(super) const TILES_FORMAT: TextureFormat = TextureFormat::R8Uint;

#[derive(Resource)]
pub(super) struct LightMapPipelineBindGroups {
    pub(super) scan_bind_group: BindGroup,
    pub(super) light_sources_bind_group: BindGroup,
    pub(super) left_to_right_bind_group: BindGroup,
    pub(super) top_to_bottom_bind_group: BindGroup,
    pub(super) right_to_left_bind_group: BindGroup,
    pub(super) bottom_to_top_bind_group: BindGroup,
}

#[derive(Resource)]
pub(super) struct LightMapPipeline {
    pub(super) scan_layout: BindGroupLayout,
    pub(super) scan_pipeline: CachedComputePipelineId,

    pub(super) light_sources_layout: BindGroupLayout,
    pub(super) light_sources_pipeline: CachedComputePipelineId,

    pub(super) left_to_right_layout: BindGroupLayout,
    pub(super) left_to_right_pipeline: CachedComputePipelineId,

    pub(super) top_to_bottom_layout: BindGroupLayout,
    pub(super) top_to_bottom_pipeline: CachedComputePipelineId,

    pub(super) right_to_left_layout: BindGroupLayout,
    pub(super) right_to_left_pipeline: CachedComputePipelineId,

    pub(super) bottom_to_top_layout: BindGroupLayout,
    pub(super) bottom_to_top_pipeline: CachedComputePipelineId,
}

pub(super) fn init_light_map_texture(
    mut commands: Commands,
    world_data: Res<WorldData>,
    light_smoothness: Res<LightSmoothness>,
    mut images: ResMut<Assets<Image>>,
) {
    let width = world_data.size.width as u32 * light_smoothness.subdivision();
    let height = world_data.size.height as u32 * light_smoothness.subdivision();

    let mut texture = Image::new_fill(
        Extent3d {
            width,
            height,
            ..Default::default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        LIGHTMAP_FORMAT,
    );

    texture.texture_descriptor.usage = TextureUsages::COPY_SRC | TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING;

    texture.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        ..default()
    });

    commands.insert_resource(LightMapTexture(images.add(texture)));
}

pub(super) fn queue_lightmap_bind_groups(
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
        Some(light_sources),
    ) = (
        pipeline_assets.area_min.binding(),
        pipeline_assets.area_max.binding(),
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