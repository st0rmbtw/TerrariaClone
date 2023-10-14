#![allow(clippy::type_complexity)]
#![allow(clippy::needless_update)]
#![allow(clippy::too_many_arguments)]

use std::{error::Error, time::Duration};

use bevy::{
    DefaultPlugins,
    log::LogPlugin,
    prelude::{default, App, AssetPlugin, ClearColor, Color, FixedTime, ImagePlugin, PluginGroup},
    window::{Cursor, MonitorSelection, Window, WindowPlugin, WindowPosition, WindowResolution},
    asset::ChangeWatcher
};

use language::{load_language, Language};
use plugins::{
    config::{FullScreen, Resolution, ConfigPlugin, VSync}, main::MainPlugin,
};
use rand::seq::SliceRandom;

pub mod world;

pub(crate) mod animation;
pub(crate) mod common;
pub(crate) mod items;
pub(crate) mod language;
pub(crate) mod lighting;
pub(crate) mod parallax;
pub(crate) mod plugins;

pub(crate) const BACKGROUND_LAYER: f32 = 0.;
pub(crate) const WALL_LAYER: f32 = 1.;
pub(crate) const TILES_LAYER: f32 = 2.;
pub(crate) const PLAYER_LAYER: f32 = 3.;

pub fn create_app() -> Result<App, Box<dyn Error>> {
    let language_content = load_language(Language::English)?;
    let title = language_content.titles.choose(&mut rand::thread_rng()).unwrap();

    let mut app = App::new();

    app.add_plugins(ConfigPlugin);

    let resolution = *app.world.resource::<Resolution>();
    let vsync = *app.world.resource::<VSync>();
    let fullscreen = *app.world.resource::<FullScreen>();

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
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    resizable: false,
                    ..default()
                }),
                close_when_requested: false,
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(50)),
                ..default()
            })
            .set(LogPlugin::default())
            .set(ImagePlugin::default_nearest())
        )
        .insert_resource(language_content)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(FixedTime::new_from_secs(1. / 60.))
        .add_plugins(MainPlugin);

    Ok(app)
}