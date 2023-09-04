use bevy::prelude::{Plugin, App, Update, IntoSystemConfigs, PostUpdate, OnEnter};
use bevy::sprite::Material2dPlugin;
use crate::common::state::GameState;
use crate::plugins::InGameSystemSet;

use self::compositing::{LightMapMaterial, PostProcessingMaterial};

pub mod compositing;

pub(crate) struct LightingPlugin;
impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<LightMapMaterial>::default(),
            Material2dPlugin::<PostProcessingMaterial>::default(),
        ));

        app.add_systems(
            Update,
            (
                compositing::update_image_to_window_size,
            )
        );

        app.add_systems(OnEnter(GameState::InGame), compositing::setup_post_processing_camera);

        app.add_systems(PostUpdate, compositing::update_light_map.in_set(InGameSystemSet::PostUpdate));
    }
}