use bevy::{prelude::{Handle, Image, ResMut, Assets, default, Commands, Res, World, FromWorld, AssetServer, Resource, UVec2}, render::{render_resource::{BindGroup, TextureFormat, FilterMode, Extent3d, TextureDimension, TextureUsages, AddressMode, SamplerDescriptor, BindGroupLayout, CachedComputePipelineId, BindGroupDescriptor, BindGroupEntry, BindingResource, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, StorageTextureAccess, TextureViewDimension, PipelineCache, ComputePipelineDescriptor, BufferBindingType, ShaderType, ShaderDefVal}, texture::ImageSampler, renderer::RenderDevice, render_asset::RenderAssets, extract_resource::ExtractResource}};

use crate::world::WorldData;

use super::{SUBDIVISION, pipeline_assets::PipelineAssets};

pub(super) const TARGET_FORMAT: TextureFormat = TextureFormat::R8Unorm;
pub(super) const TILES_FORMAT: TextureFormat = TextureFormat::R8Uint;

#[derive(Clone, Resource, ExtractResource, Default)]
pub(super) struct PipelineTargetsWrapper {
    pub(super) tiles: Option<Handle<Image>>,
    pub(super) light_map: Option<Handle<Image>>,
}

#[derive(Resource)]
pub(super) struct PipelineBindGroups {
    pub(super) scan_bind_group: BindGroup,
    pub(super) left_to_right_bind_group: BindGroup,
    pub(super) top_to_bottom_bind_group: BindGroup,
    pub(super) right_to_left_bind_group: BindGroup,
    pub(super) bottom_to_top_bind_group: BindGroup,
}

#[derive(Resource)]
pub(super) struct LightMapPipeline {
    pub(super) scan_layout: BindGroupLayout,
    pub(super) scan_pipeline: CachedComputePipelineId,
    pub(super) left_to_right_layout: BindGroupLayout,
    pub(super) left_to_right_pipeline: CachedComputePipelineId,
    pub(super) top_to_bottom_layout: BindGroupLayout,
    pub(super) top_to_bottom_pipeline: CachedComputePipelineId,
    pub(super) right_to_left_layout: BindGroupLayout,
    pub(super) right_to_left_pipeline: CachedComputePipelineId,
    pub(super) bottom_to_top_layout: BindGroupLayout,
    pub(super) bottom_to_top_pipeline: CachedComputePipelineId,
}


pub(super) fn create_texture_2d(size: (u32, u32), format: TextureFormat) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: size.0,
            height: size.1,
            ..Default::default()
        },
        TextureDimension::D2,
        &[0],
        format,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        address_mode_u: AddressMode::ClampToBorder,
        address_mode_v: AddressMode::ClampToBorder,
        address_mode_w: AddressMode::ClampToBorder,
        ..default()
    });

    image
}

pub(super) fn setup_pipeline_targets(
    world_data: Res<WorldData>,
    mut images: ResMut<Assets<Image>>,
    mut targets_wrapper: ResMut<PipelineTargetsWrapper>,
) {
    let width = world_data.size.width as u32 * SUBDIVISION;
    let height = world_data.size.height as u32 * SUBDIVISION;

    let texture = create_texture_2d((width, height), TARGET_FORMAT);

    targets_wrapper.light_map = Some(images.add(texture));
}

pub(super) fn queue_bind_groups(
    mut commands: Commands,
    pipeline: Res<LightMapPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    targets_wrapper: Option<Res<PipelineTargetsWrapper>>,
    render_device: Res<RenderDevice>,
    pipeline_assets: Res<PipelineAssets>
) {
    let Some(targets_wrapper) = targets_wrapper else { return; };
    let Some(tiles) = targets_wrapper.tiles.as_ref() else { return; };
    let Some(light_map) = targets_wrapper.light_map.as_ref() else { return; };

    if let (
        Some(min),
        Some(max)
    ) = (
        pipeline_assets.min.binding(),
        pipeline_assets.max.binding()
    ) {
        let tiles_image = &gpu_images[tiles];
        let lightmap_image = &gpu_images[light_map];

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
                    resource: min.clone()
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
                    resource: min.clone()
                },
                BindGroupEntry {
                    binding: 3,
                    resource: max.clone()
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
                    resource: min.clone()
                },
                BindGroupEntry {
                    binding: 3,
                    resource: max.clone()
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
                    resource: min.clone()
                },
                BindGroupEntry {
                    binding: 3,
                    resource: max.clone()
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
                    resource: min
                },
                BindGroupEntry {
                    binding: 3,
                    resource: max
                },
            ],
        });

        commands.insert_resource(PipelineBindGroups {
            scan_bind_group,
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
                            format: TARGET_FORMAT,
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
                            format: TARGET_FORMAT,
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
                            format: TARGET_FORMAT,
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
                            format: TARGET_FORMAT,
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
                            format: TARGET_FORMAT,
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


        let (shader_scan, shader_blur) = {
            let assets_server = world.resource::<AssetServer>();
            (
                assets_server.load("shaders/light_map/scan.wgsl"),
                assets_server.load("shaders/light_map/blur.wgsl"),
            )
        };

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let scan_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("scan_pipeline".into()),
            layout: vec![scan_layout.clone()],
            shader: shader_scan,
            shader_defs: vec![ShaderDefVal::UInt("SUBDIVISION".into(), SUBDIVISION)],
            entry_point: "scan".into(),
            push_constant_ranges: vec![],
        });

        let left_to_right_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("left_to_right_pipeline".into()),
            layout: vec![left_to_right_layout.clone()],
            shader: shader_blur.clone(),
            shader_defs: vec![ShaderDefVal::UInt("SUBDIVISION".into(), SUBDIVISION)],
            entry_point: "left_to_right".into(),
            push_constant_ranges: vec![],
        });

        let top_to_bottom_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("top_to_bottom_pipeline".into()),
            layout: vec![top_to_bottom_layout.clone()],
            shader: shader_blur.clone(),
            shader_defs: vec![ShaderDefVal::UInt("SUBDIVISION".into(), SUBDIVISION)],
            entry_point: "top_to_bottom".into(),
            push_constant_ranges: vec![],
        });

        let right_to_left_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("right_to_left_pipeline".into()),
            layout: vec![right_to_left_layout.clone()],
            shader: shader_blur.clone(),
            shader_defs: vec![ShaderDefVal::UInt("SUBDIVISION".into(), SUBDIVISION)],
            entry_point: "right_to_left".into(),
            push_constant_ranges: vec![],
        });

        let bottom_to_top_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("bottom_to_top_pipeline".into()),
            layout: vec![bottom_to_top_layout.clone()],
            shader: shader_blur,
            shader_defs: vec![ShaderDefVal::UInt("SUBDIVISION".into(), SUBDIVISION)],
            entry_point: "bottom_to_top".into(),
            push_constant_ranges: vec![],
        });

        LightMapPipeline {
            scan_layout,
            scan_pipeline,
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