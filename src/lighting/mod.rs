use bevy::prelude::{Plugin, App, Update, IntoSystemConfigs, resource_added};

use crate::{plugins::world::resources::LightMap, lighting::compositing::setup_post_processing_camera};

pub mod compositing;

pub(crate) struct LightingPlugin;
impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, setup_post_processing_camera.run_if(resource_added::<LightMap>()));
    }
}