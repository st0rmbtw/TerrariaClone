use std::{fs::OpenOptions, io::{BufReader, BufWriter}, error::Error};

use bevy::{prelude::{Plugin, App, IntoSystemConfigs, Update, Res, on_event}, app::AppExit, window::{WindowCloseRequested, PrimaryWindow}};
use serde::{Deserialize, Serialize};

use crate::common::systems::despawn_with;

mod resources;

pub(crate) use resources::*;

use super::camera::resources::Zoom;

const CONFIG_FILENAME: &str = "config.json";

pub(super) const RESOLUTIONS: [Resolution; 16] = [
    Resolution::new(800., 600.),
    Resolution::new(1024., 768.),
    Resolution::new(1152., 864.),
    Resolution::new(1176., 664.),
    Resolution::new(1280., 720.),
    Resolution::new(1280., 768.),
    Resolution::new(1280., 800.),
    Resolution::new(1280., 960.),
    Resolution::new(1280., 1024.),
    Resolution::new(1360., 768.),
    Resolution::new(1366., 768.),
    Resolution::new(1440., 900.),
    Resolution::new(1600., 900.),
    Resolution::new(1600., 1024.),
    Resolution::new(1680., 1050.),
    Resolution::new(1920., 1080.),
];

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Config {
    pub(crate) full_screen: bool,
    pub(crate) show_tile_grid: bool,
    #[serde(rename = "VSync")]
    pub(crate) vsync: bool,
    pub(crate) resolution: Resolution,
    pub(crate) cursor_color: CursorColor,
    pub(crate) zoom: f32,
    pub(crate) sound_volume: f32,
    pub(crate) music_volume: f32
}


impl Default for Config {
    fn default() -> Self {
        Self { 
            full_screen: true,
            show_tile_grid: false,
            vsync: true,
            resolution: Resolution::new(1920., 1080.),
            cursor_color: CursorColor::default(),
            zoom: 0.67,
            sound_volume: 100.,
            music_volume: 100.
        }
    }
}

pub(crate) struct ConfigPlugin;
impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        let config = load_config().unwrap_or_default();

        app.insert_resource(FullScreen(config.full_screen));
        app.insert_resource(ShowTileGrid(config.show_tile_grid));
        app.insert_resource(VSync(config.vsync));
        app.insert_resource(MusicVolume::new(config.music_volume));
        app.insert_resource(SoundVolume::new(config.sound_volume));
        app.insert_resource(Zoom::new(config.zoom));
        app.insert_resource(config.cursor_color);
        app.insert_resource(config.resolution);

        app.add_systems(
            Update,
            (
                on_exit.run_if(on_event::<AppExit>()),
                (on_exit, despawn_with::<PrimaryWindow>).run_if(on_event::<WindowCloseRequested>())
            )
        );
    }
}

fn on_exit(
    fullscreen: Res<FullScreen>,
    show_tile_grid: Res<ShowTileGrid>,
    vsync: Res<VSync>,
    resolution: Res<Resolution>,
    cursor_color: Res<CursorColor>,
    music_volume: Res<MusicVolume>,
    sound_volume: Res<SoundVolume>,
    zoom: Res<Zoom>,
) {
    save_config(Config {
        full_screen: fullscreen.0,
        show_tile_grid: show_tile_grid.0,
        vsync: vsync.0,
        resolution: *resolution,
        cursor_color: *cursor_color,
        sound_volume: sound_volume.get(),
        music_volume: music_volume.get(),
        zoom: zoom.get()
    });
}

fn load_config() -> Result<Config, Box<dyn Error>> {
    let file = OpenOptions::new()
        .read(true)
        .open(CONFIG_FILENAME)?;

    let reader = BufReader::new(file);

    let config: Config = serde_json::from_reader(reader)?;

    Ok(config)
}

pub(super) fn save_config(config: Config) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(CONFIG_FILENAME)
        .unwrap();

    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, &config).unwrap();
}