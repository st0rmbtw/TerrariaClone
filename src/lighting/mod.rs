use bevy::prelude::{Plugin, App, Update, resource_added, IntoSystemConfigs};
use crate::plugins::world::resources::LightMap;

use crate::plugins::camera::events::UpdateLightEvent;

pub mod compositing;

pub(crate) struct LightingPlugin;
impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateLightEvent>();
        
        app.add_systems(Update, compositing::setup_post_processing_camera.run_if(resource_added::<LightMap>()));
    }
}