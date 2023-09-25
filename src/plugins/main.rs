use std::time::Duration;

use bevy::{prelude::{Plugin, App, OnExit, PreUpdate, Update, PostUpdate, FixedUpdate, in_state, GizmoConfig, default, IntoSystemSetConfig, UVec2, Msaa}, winit::{WinitSettings, UpdateMode}};
use bevy_ecs_tilemap::prelude::TilemapRenderSettings;

use crate::{common::{systems::despawn_with, state::{GameState, MenuState}}, lighting::LightingPlugin, parallax::ParallaxPlugin, animation::TweeningPlugin, language::plugin::LanguagePlugin};

use super::{InGameSystemSet, MenuSystemSet, DespawnOnGameExit, audio::AudioPlugin, cursor::CursorPlugin, camera::CameraPlugin, background::BackgroundPlugin, ui::UiPlugin, world::WorldPlugin, inventory::PlayerInventoryPlugin, fps::FpsPlugin, player::PlayerPlugin, slider::SliderPlugin, assets::AssetsPlugin, particles::ParticlePlugin};

pub(crate) struct MainPlugin;
impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TweeningPlugin,
            AssetsPlugin,
            ParallaxPlugin,
            SliderPlugin,
        ));

        app.add_plugins((
            ParticlePlugin,
            LanguagePlugin,
            LightingPlugin,
            AudioPlugin,
            CursorPlugin,
            CameraPlugin,
            BackgroundPlugin,
            UiPlugin,
            WorldPlugin,
            PlayerInventoryPlugin,
            FpsPlugin,
            PlayerPlugin,
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

        app.insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::ReactiveLowPower {
                max_wait: Duration::from_millis(1000 / 30),
            },
            ..default()
        });

        app.insert_resource(Msaa::Off);

        app.configure_set(PreUpdate, InGameSystemSet::PreUpdate.run_if(in_state(GameState::InGame)));
        app.configure_set(Update, InGameSystemSet::Update.run_if(in_state(GameState::InGame)));
        app.configure_set(PostUpdate, InGameSystemSet::PostUpdate.run_if(in_state(GameState::InGame)));
        app.configure_set(FixedUpdate, InGameSystemSet::FixedUpdate.run_if(in_state(GameState::InGame)));
        
        app.configure_set(PreUpdate, MenuSystemSet::PreUpdate.run_if(in_state(GameState::Menu)));
        app.configure_set(Update, MenuSystemSet::Update.run_if(in_state(GameState::Menu)));
        app.configure_set(PostUpdate, MenuSystemSet::PostUpdate.run_if(in_state(GameState::Menu)));

        // #[cfg(debug_assertions)]
        // app.edit_schedule(PreUpdate, |schedule| {
        //     schedule.set_build_settings(ScheduleBuildSettings {
        //         ambiguity_detection: LogLevel::Error,
        //         report_sets: false,
        //         ..default()
        //     });
        // });

        // app.edit_schedule(Update, |schedule| {
        //     schedule.set_build_settings(ScheduleBuildSettings {
        //         ambiguity_detection: LogLevel::Error,
        //         ..default()
        //     });
        // });

        // app.edit_schedule(PostUpdate, |schedule| {
        //     schedule.set_build_settings(ScheduleBuildSettings {
        //         ambiguity_detection: LogLevel::Error,
        //         report_sets: false,
        //         ..default()
        //     });
        // });

        // #[cfg(debug_assertions)]
        // app.edit_schedule(FixedUpdate, |schedule| {
        //     schedule.set_build_settings(ScheduleBuildSettings {
        //         ambiguity_detection: LogLevel::Error,
        //         report_sets: false,
        //         ..default()
        //     });
        // });

        app.add_systems(OnExit(GameState::InGame), despawn_with::<DespawnOnGameExit>);
    }
}