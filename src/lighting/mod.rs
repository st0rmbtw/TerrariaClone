// Based on https://github.com/zaycev/bevy-magic-light-2d

use bevy::{
    render::{
        extract_resource::ExtractResourcePlugin, RenderApp,
        render_graph::{RenderGraph, self}, 
        renderer::RenderContext, 
        render_resource::{PipelineCache, ComputePassDescriptor, Extent3d}, ExtractSchedule, RenderSet, Render
    }, 
    prelude::{Shader, Vec2, ResMut, Res, World, Plugin, App, default, EventReader, Assets, Image, warn, resource_exists, in_state, Mesh, shape, Transform, Vec3, Name, Color, Commands, Component, Query, Input, KeyCode, MouseButton, With, Startup, Update, PostUpdate, IntoSystemConfigs},
    window::WindowResized,
    asset::load_internal_asset, sprite::{Material2dPlugin, ColorMaterial, MaterialMesh2dBundle},
};
use rand::{thread_rng, Rng};

use crate::{plugins::{camera::CameraSet, settings::Resolution, cursor::resources::CursorPosition}, lighting::{compositing::{PostProcessingMaterial, setup_post_processing_camera, update_image_to_window_size, update_lighting_material, update_light_map}, constants::{SHADER_ATTENUATION, SHADER_MATH}}, common::state::GameState, world::WorldData};

use self::{
    pipeline::{LightPassPipelineBindGroups, PipelineTargetsWrapper, system_setup_pipeline, LightPassPipeline, system_queue_bind_groups}, 
    resource::{LightPassParams, ComputedTargetSizes}, 
    constants::{SHADER_CAMERA, SHADER_TYPES, SCREEN_PROBE_SIZE}, 
    pipeline_assets::{LightPassPipelineAssets, system_extract_pipeline_assets, system_prepare_pipeline_assets}, types::LightSource
};

pub mod resource;
pub mod types;
pub mod types_gpu;
pub mod pipeline;
pub mod pipeline_assets;
pub mod constants;
pub mod compositing;

const WORKGROUP_SIZE: u32 = 8;

pub(crate) struct LightingPlugin;
impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<PipelineTargetsWrapper>::default(),
            Material2dPlugin::<PostProcessingMaterial>::default()
        ));
            
        app.init_resource::<PipelineTargetsWrapper>();
        app.init_resource::<ComputedTargetSizes>();
        app.insert_resource(LightPassParams {
            reservoir_size: 16
        });

        app.add_systems(Startup, detect_target_sizes);
        app.add_systems(Startup, system_setup_pipeline.after(detect_target_sizes));

        app.add_systems(Update, setup_post_processing_camera.run_if(resource_exists::<WorldData>()));
        app.add_systems(Update, update_image_to_window_size);
        app.add_systems(Update, resize_lighting_target);
        app.add_systems(Update, update_light_map.run_if(in_state(GameState::InGame)));
        
        app.add_systems(
            PostUpdate,
            update_lighting_material
                .run_if(in_state(GameState::InGame))
                .after(CameraSet::MoveCamera)
        );
        
        // app.add_systems(OnEnter(GameState::InGame), spawn_mouse_light);
        // app.add_systems(Update, control_mouse_light.run_if(in_state(GameState::InGame)));
        // app.add_systems(Update,
        //     toggle_visibility::<MouseLight>
        //         .run_if(GameState::InGame)
        //         .run_if(input_just_pressed(KeyCode::F1))
        // );

        #[cfg(feature = "debug")] {
            app.add_systems(
                Update,
                self::compositing::set_shadow_map_visibility
                    .run_if(in_state(GameState::InGame))
            );
        }

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
            .add_systems(ExtractSchedule, system_extract_pipeline_assets)
            .add_systems(Render, system_prepare_pipeline_assets.in_set(RenderSet::Prepare))
            .add_systems(Render, system_queue_bind_groups.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("light_pass", LightPass2DNode::default());
        render_graph
            .add_node_edge(
                "light_pass",
                bevy::render::main_graph::node::CAMERA_DRIVER,
            );
    }
}

#[derive(Default)]
struct LightPass2DNode {}

fn detect_target_sizes(
    mut target_sizes: ResMut<ComputedTargetSizes>,
    resolution: Res<Resolution>
) {
    let primary_size = Vec2::new(
        resolution.width,
        resolution.height
    );
    
    target_sizes.primary_target_size = primary_size;
}

fn resize_lighting_target(
    mut resize_events: EventReader<WindowResized>,
    mut target_sizes: ResMut<ComputedTargetSizes>,
    mut images: ResMut<Assets<Image>>,
    targets_wrapper: Res<PipelineTargetsWrapper>,
) {
    for event in resize_events.iter() {
        if event.width > 0. && event.height > 0. {
            target_sizes.primary_target_size = Vec2::new(
                event.width,
                event.height
            );

            if let Some(targets) = targets_wrapper.targets.as_ref() {
                let size = target_sizes.primary_target_usize();

                let extent = Extent3d {
                    width: size.x,
                    height: size.y,
                    ..default()  
                };

                images.get_mut(&targets.lighting_target)
                    .unwrap()
                    .resize(extent);
            }
        }
    }
}


#[derive(Component)]
pub(super) struct MouseLight;

fn spawn_mouse_light(
    mut commands: Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let block_mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::ZERO)));

    commands
        .spawn(MaterialMesh2dBundle {
            mesh: block_mesh.into(),
            material: color_materials.add(ColorMaterial::from(Color::YELLOW)),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1000.0),
                scale: Vec3::splat(8.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Cursor Light"))
        .insert(LightSource {
            intensity: 10.,
            radius: 100.,
            jitter_intensity: 0.7,
            jitter_translation: 0.1,
            color: Color::rgb_u8(254, 100, 34)
        })
        .insert(MouseLight);
}

pub(super) fn control_mouse_light(
    mut query: Query<(&mut Transform, &mut LightSource), With<MouseLight>>,
    cursor_position: Res<CursorPosition>,
    input_keyboard: Res<Input<KeyCode>>,
    input_mouse: Res<Input<MouseButton>>,
) {
    let mut rng = thread_rng();

    let (mut transform, mut light_source) = query.single_mut();

    transform.translation = cursor_position.world_position.extend(10.);

    if input_mouse.just_pressed(MouseButton::Right) && input_keyboard.pressed(KeyCode::ShiftLeft) {
        light_source.color = Color::rgba(rng.gen(), rng.gen(), rng.gen(), 1.0);
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
                            label: Some("light_pass"),
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
