use bevy::prelude::{Plugin, App, OnExit, PreUpdate, Update, PostUpdate, FixedUpdate, in_state, GizmoConfig, default, IntoSystemSetConfig, UVec2};
use bevy_ecs_tilemap::prelude::TilemapRenderSettings;
use bevy_hanabi::HanabiPlugin;

use crate::{common::{systems::despawn_with, state::{GameState, MenuState}}, lighting::LightingPlugin, parallax::ParallaxPlugin, animation::TweeningPlugin};

use super::{InGameSystemSet, MenuSystemSet, DespawnOnGameExit, audio::AudioPlugin, cursor::CursorPlugin, camera::CameraPlugin, background::BackgroundPlugin, ui::UiPlugin, world::WorldPlugin, inventory::PlayerInventoryPlugin, fps::FpsPlugin, player::PlayerPlugin, slider::SliderPlugin, assets::AssetsPlugin};

pub(crate) struct MainPlugin;
impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TweeningPlugin,
            AssetsPlugin,
            HanabiPlugin,
            ParallaxPlugin,
            SliderPlugin,
        ));

        app.add_plugins((
            LightingPlugin,
            AudioPlugin,
            CursorPlugin,
            CameraPlugin,
            BackgroundPlugin,
            UiPlugin,
            WorldPlugin,
            PlayerInventoryPlugin,
            FpsPlugin,
            PlayerPlugin
        ));

        #[cfg(feature = "debug")] {
            use super::debug::DebugPlugin;
            app.add_plugins(DebugPlugin);
        }

        app.add_state::<GameState>();
        app.add_state::<MenuState>();

        app.insert_resource(GizmoConfig {
            line_width: 1.,
            depth_bias: -1.,
            ..default()
        });

        app.insert_resource(TilemapRenderSettings {
            render_chunk_size: UVec2::new(100, 100),
            y_sort: false,
        });

        app.configure_set(PreUpdate, InGameSystemSet::PreUpdate.run_if(in_state(GameState::InGame)));
        app.configure_set(Update, InGameSystemSet::Update.run_if(in_state(GameState::InGame)));
        app.configure_set(PostUpdate, InGameSystemSet::PostUpdate.run_if(in_state(GameState::InGame)));
        app.configure_set(FixedUpdate, InGameSystemSet::FixedUpdate.run_if(in_state(GameState::InGame)));
        
        app.configure_set(PreUpdate, MenuSystemSet::PreUpdate.run_if(in_state(GameState::Menu)));
        app.configure_set(Update, MenuSystemSet::Update.run_if(in_state(GameState::Menu)));
        app.configure_set(PostUpdate, MenuSystemSet::PostUpdate.run_if(in_state(GameState::Menu)));

        app.add_systems(OnExit(GameState::InGame), despawn_with::<DespawnOnGameExit>);
    }
}