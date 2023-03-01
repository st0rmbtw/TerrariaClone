use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::ImageSampler;

use super::pipeline_assets::LightPassPipelineAssets;
use super::resource::ComputedTargetSizes;
use super::types_gpu::{GpuCameraParams, GpuLightSourceBuffer, GpuLightPassParams, GpuLightMap};

const LIGHTING_TARGET_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
const LIGHT_MAP_TARGET_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

const LIGHTING_PIPELINE_ENTRY: &str = "main";

#[derive(Clone, Resource, ExtractResource, Default)]
pub(super) struct PipelineTargetsWrapper {
    pub(super) targets: Option<GiPipelineTargets>,
}

#[derive(Clone)]
pub(super) struct GiPipelineTargets {
    pub(super) lighting_target: Handle<Image>,
    pub(super) light_map_target: Handle<Image>,
}

#[derive(Resource)]
pub(super) struct LightPassPipelineBindGroups {
    pub(super) lighting_bind_group: BindGroup,
    pub(super) light_map_bind_group: BindGroup,
}


fn create_texture_2d(size: Extent3d, format: TextureFormat, filter: FilterMode) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: size.width,
            height: size.height,
            ..Default::default()
        },
        TextureDimension::D2,
        &[
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
        format,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: filter,
        min_filter: filter,
        address_mode_u: AddressMode::ClampToBorder,
        address_mode_v: AddressMode::ClampToBorder,
        address_mode_w: AddressMode::ClampToBorder,
        ..Default::default()
    });

    image
}

pub(super) fn system_setup_gi_pipeline(
    mut images: ResMut<Assets<Image>>,
    mut targets_wrapper: ResMut<PipelineTargetsWrapper>,
    targets_sizes: Res<ComputedTargetSizes>
) {
    let lighting_tex = { 
        let size = Extent3d {
            width:  targets_sizes.primary_target_usize().x,
            height: targets_sizes.primary_target_usize().y,
            ..default()
        };

        create_texture_2d(
            size,
            LIGHTING_TARGET_FORMAT,
            FilterMode::Nearest,
        )
    };

    let light_map_tex = {
        let size = Extent3d {
            width:  1750,
            height: 900,
            ..default()
        };

        create_texture_2d(size, LIGHT_MAP_TARGET_FORMAT, FilterMode::Linear)
    };

    let lighting_target  = images.add(lighting_tex);
    let light_map_target  = images.add(light_map_tex);

    targets_wrapper.targets = Some(GiPipelineTargets {
        lighting_target,
        light_map_target
    });
}

#[derive(Resource)]
pub(super) struct LightPassPipeline {
    pub lighting_bind_group_layout: BindGroupLayout,
    pub lighting_pipeline: CachedComputePipelineId,
    pub light_map_bind_group_layout: BindGroupLayout,
    pub light_map_pipeline: CachedComputePipelineId,
}

pub(super) fn system_queue_bind_groups(
    mut commands: Commands,
    pipeline: Res<LightPassPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    targets_wrapper: Res<PipelineTargetsWrapper>,
    gi_compute_assets: Res<LightPassPipelineAssets>,
    render_device: Res<RenderDevice>,
) {
    if let (
        Some(light_sources),
        Some(light_map),
        Some(camera_params),
        Some(light_pass_params),
    ) = (
        gi_compute_assets.light_sources.binding(),
        gi_compute_assets.light_map.binding(),
        gi_compute_assets.camera_params.binding(),
        gi_compute_assets.light_pass_params.binding(),
    ) {
        let targets = targets_wrapper
            .targets
            .as_ref()
            .expect("Targets should be initialized");

        let lighting_image = &gpu_images[&targets.lighting_target];
        let light_map_image = &gpu_images[&targets.light_map_target];

        let lighting_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "lighting".into(),
            layout: &pipeline.lighting_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_params.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: light_pass_params,
                },
                BindGroupEntry {
                    binding: 2,
                    resource: light_sources,
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&lighting_image.texture_view),
                },
            ],
        });

        let light_map_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "light_map".into(),
            layout: &pipeline.light_map_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_params,
                },
                BindGroupEntry {
                    binding: 1,
                    resource: light_map,
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&light_map_image.texture_view),
                },
            ],
        });

        commands.insert_resource(LightPassPipelineBindGroups {
            lighting_bind_group,
            light_map_bind_group
        });
    }
}

impl FromWorld for LightPassPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let lighting_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("lighting_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(GpuCameraParams::min_size()),
                        },
                        count: None,
                    },
                    
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(GpuLightPassParams::min_size()),
                        },
                        count: None,
                    },
                    
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(GpuLightSourceBuffer::min_size()),
                        },
                        count: None,
                    },

                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: LIGHTING_TARGET_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let light_map_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("light_map_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(GpuCameraParams::min_size()),
                        },
                        count: None,
                    },

                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(GpuLightMap::min_size()),
                        },
                        count: None,
                    },

                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: LIGHT_MAP_TARGET_FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let assets_server = world.resource::<AssetServer>();
        
        let lighting = assets_server.load("shaders/lighting.wgsl");
        let light_map_shader = assets_server.load("shaders/light_map.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        let lighting_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("lighting_pipeline".into()),
            layout: Some(vec![lighting_bind_group_layout.clone()]),
            shader: lighting.clone(),
            shader_defs: vec![],
            entry_point: LIGHTING_PIPELINE_ENTRY.into(),
        });

        let light_map_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("light_map_pipeline".into()),
            layout: Some(vec![light_map_bind_group_layout.clone()]),
            shader: light_map_shader,
            shader_defs: vec![],
            entry_point: LIGHTING_PIPELINE_ENTRY.into(),
        });

        LightPassPipeline {
            lighting_bind_group_layout,
            lighting_pipeline,
            light_map_bind_group_layout,
            light_map_pipeline
        }
    }
}
