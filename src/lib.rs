#![allow(clippy::type_complexity)]
#![allow(clippy::needless_update)]
#![allow(clippy::too_many_arguments)]

use std::error::Error;

use animation::TweeningPlugin;
use bevy::{prelude::{Color, Resource, App, Msaa, default, PluginGroup, AssetPlugin, ImagePlugin, ClearColor, FixedTime, UVec2}, window::{WindowPlugin, Window, Cursor, WindowResolution, WindowPosition, MonitorSelection}, DefaultPlugins, log::{LogPlugin, Level}};
use bevy_ecs_tilemap::{prelude::TilemapRenderSettings, TilemapPlugin};
use bevy_hanabi::HanabiPlugin;
use common::state::GameState;
use language::{load_language, Language};
use lighting::LightingPlugin;
use parallax::ParallaxPlugin;
use plugins::{settings::{SettingsPlugin, Resolution, VSync, FullScreen}, camera::{UpdateLightEvent, CameraPlugin}, assets::AssetsPlugin, cursor::CursorPlugin, background::BackgroundPlugin, ui::PlayerUiPlugin, menu::MenuPlugin, world::WorldPlugin, inventory::PlayerInventoryPlugin, fps::FpsPlugin, player::PlayerPlugin};
use rand::seq::SliceRandom;

pub(crate) mod animation;
pub(crate) mod common;
pub(crate) mod items;
pub(crate) mod language;
pub(crate) mod lighting;
pub(crate) mod parallax;
pub(crate) mod plugins;

pub(crate) const TEXT_COLOR: Color = Color::rgb(156. / 255., 156. / 255., 156. / 255.);

#[derive(Default, Resource)]
pub(crate) struct DebugConfiguration {
    pub(crate) free_camera: bool,
    pub(crate) instant_break: bool,

    #[cfg(feature = "debug")]
    pub(crate) show_hitboxes: bool,
    #[cfg(feature = "debug")]
    pub(crate) show_collisions: bool,
    #[cfg(feature = "debug")]
    pub(crate) player_speed: bevy::prelude::Vec2
}

pub struct GameApp;
impl GameApp {
    pub fn new() -> Result<App, Box<dyn Error>> {
        let language_content = load_language(Language::English)?;
        let title = language_content.titles.choose(&mut rand::thread_rng()).unwrap();

        let mut app = App::new();

        app.add_plugin(SettingsPlugin);

        let resolution = *app.world.resource::<Resolution>();
        let vsync = *app.world.resource::<VSync>();
        let fullscreen = *app.world.resource::<FullScreen>();

        app
            .insert_resource(Msaa::Off)
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
                y_sort: false
            })
            .insert_resource(language_content)
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
            use plugins::debug::DebugPlugin;
            app.add_plugin(DebugPlugin);
        }

        Ok(app)
    }
}
