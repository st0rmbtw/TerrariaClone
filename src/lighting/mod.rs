use bevy::prelude::{Plugin, App, Update, resource_added, IntoSystemConfigs};
use bevy::sprite::Material2dPlugin;
use crate::plugins::InGameSystemSet;
use crate::plugins::world::resources::LightMap;

use crate::plugins::camera::events::UpdateLightEvent;

use self::compositing::{LightMapMaterial, PostProcessingMaterial};

pub mod compositing;

pub(crate) struct LightingPlugin;
impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<LightMapMaterial>::default(),
            Material2dPlugin::<PostProcessingMaterial>::default(),
        ));

        app.add_event::<UpdateLightEvent>();

        app.add_systems(
            Update,
            (
                compositing::setup_light_map_texture.run_if(resource_added::<LightMap>()),
                compositing::update_image_to_window_size,
                (
                    compositing::setup_post_processing_camera,
                    compositing::update_light_map   
                ).in_set(InGameSystemSet::Update)
            )
        );
    }
}