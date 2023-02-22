// #![windows_subsystem = "windows"]

use std::{error::Error, time::Duration};

use bevy::{prelude::*, render::settings::{WgpuSettings, WgpuFeatures}};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_hanabi::HanabiPlugin;
use game::{
    animation::TweeningPlugin,
    parallax::ParallaxPlugin,
    state::GameState, 
    plugins::{
        assets::AssetsPlugin, cursor::CursorPlugin, camera::CameraPlugin, background::BackgroundPlugin, 
        ui::PlayerUiPlugin, settings::{SettingsPlugin, Resolution, VSync, FullScreen}, menu::MenuPlugin, world::WorldPlugin, 
        inventory::PlayerInventoryPlugin, fps::FpsPlugin, settings_menu::{SettingsMenuState, SettingsMenuPlugin}
    }, 
    language::{load_language, Language}, lighting::LightingPlugin,
};
use iyes_loopless::prelude::{AppLooplessStateExt, AppLooplessFixedTimestepExt};
use rand::seq::SliceRandom;

fn main() -> Result<(), Box<dyn Error>> {
    let language_content = load_language(Language::US)?;

    let title = language_content.titles.choose(&mut rand::thread_rng()).unwrap();

    let mut app = App::new();

    app.add_plugin(SettingsPlugin);

    let mut wgpu_settings = WgpuSettings::default();
    wgpu_settings.features.set(WgpuFeatures::ADDRESS_MODE_CLAMP_TO_BORDER, true);

    app.insert_resource(wgpu_settings);

    let resolution = app.world.resource::<Resolution>().data();
    let vsync = app.world.resource::<VSync>();
    let fullscreen = app.world.resource::<FullScreen>();

    app
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                window: WindowDescriptor {
                    width: resolution.width,
                    height: resolution.height,
                    title: title.to_owned(),
                    present_mode: vsync.as_present_mode(),
                    cursor_visible: false,
                    position: WindowPosition::Centered,
                    mode: fullscreen.as_window_mode(),
                    ..default()
                },
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes: true,
                ..default()
            })
            .set(ImagePlugin::default_nearest())
        )
        .insert_resource(language_content.clone())
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::rgb(
            110. / 255.,
            151. / 255.,
            244. / 255.,
        )))
        .add_loopless_state(GameState::AssetLoading)
        .add_loopless_state(SettingsMenuState::None)
        .add_fixed_timestep(Duration::from_secs_f32(1. / 60.), "fixed_update")
        .add_plugin(TweeningPlugin)
        .add_plugin(TilemapPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(CursorPlugin)
        .add_plugin(SettingsMenuPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(LightingPlugin)
        .add_plugin(ParallaxPlugin { initial_speed: 0.2 })
        .add_plugin(HanabiPlugin)
        .add_plugin(BackgroundPlugin)
        .add_plugin(PlayerUiPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(PlayerInventoryPlugin)
        .add_plugin(FpsPlugin);

    #[cfg(not(feature = "free_camera"))] {
        use game::plugins::player::PlayerPlugin;
        app.add_plugin(PlayerPlugin);
    }

    #[cfg(feature = "debug")] {
        use game::plugins::debug::DebugPlugin;
        app.add_plugin(DebugPlugin);
    }

    app.run();

    Ok(())
}