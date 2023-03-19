// #![windows_subsystem = "windows"]

use std::{error::Error};

use bevy::{prelude::*, log::{LogPlugin, Level}, window::{Cursor, WindowResolution}};
use bevy_ecs_tilemap::{TilemapPlugin, prelude::TilemapRenderSettings};
use bevy_hanabi::HanabiPlugin;
use game::{
    animation::TweeningPlugin,
    parallax::ParallaxPlugin,
    common::state::GameState, 
    plugins::{
        assets::AssetsPlugin,
        cursor::CursorPlugin,
        fps::FpsPlugin,
        menu::MenuPlugin,
        world::WorldPlugin, 
        inventory::PlayerInventoryPlugin,
        player::PlayerPlugin,
        ui::PlayerUiPlugin,
        background::BackgroundPlugin,
        camera::{CameraPlugin, UpdateLightEvent},
        settings::{SettingsPlugin, Resolution, VSync, FullScreen},
    }, 
    language::{load_language, Language},
    lighting::LightingPlugin, DebugConfiguration,
};
use rand::seq::SliceRandom;

fn main() -> Result<(), Box<dyn Error>> {
    let language_content = load_language(Language::English)?;

    let title = language_content.titles.choose(&mut rand::thread_rng()).unwrap();

    let mut app = App::new();

    app.add_plugin(SettingsPlugin);

    let resolution = app.world.resource::<Resolution>();
    let vsync = app.world.resource::<VSync>();
    let fullscreen = app.world.resource::<FullScreen>();

    app
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window { 
                    cursor: Cursor {
                        visible: false,
                        ..default()
                    },
                    present_mode: vsync.as_present_mode(),
                    mode: fullscreen.as_window_mode(),
                    resolution: WindowResolution::new(resolution.width, resolution.height),
                    title: title.to_owned(),
                    position: WindowPosition::Centered(MonitorSelection::Current),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes: true,
                ..default()
            })
            .set(LogPlugin {
                level: Level::ERROR,
                filter: "game=debug".to_string(),
            })
            .set(ImagePlugin::default_nearest())
        )
        .insert_resource(TilemapRenderSettings {
            render_chunk_size: UVec2::new(100, 100),
        })
        .insert_resource(language_content.clone())
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(FixedTime::new_from_secs(1. / 60.))
        .init_resource::<DebugConfiguration>()

        .add_event::<UpdateLightEvent>()

        .add_state::<GameState>()

        .add_plugin(TweeningPlugin)
        .add_plugin(TilemapPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(HanabiPlugin)

        .add_plugin(CursorPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(LightingPlugin)
        .add_plugin(ParallaxPlugin { initial_speed: 0.15 })
        .add_plugin(BackgroundPlugin)
        .add_plugin(PlayerUiPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(PlayerInventoryPlugin)
        .add_plugin(FpsPlugin)
        .add_plugin(PlayerPlugin);

    #[cfg(feature = "debug")] {
        use game::plugins::debug::DebugPlugin;
        app.add_plugin(DebugPlugin);
    }

    app.run();

    Ok(())
}