use bevy::prelude::{Plugin, App};

use crate::plugins::camera::events::UpdateLightEvent;

pub mod compositing;

pub(crate) struct LightingPlugin;
impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateLightEvent>();
        
        // app.add_systems(Update, setup_post_processing_camera.run_if(resource_added::<LightMap>()));
    }
}