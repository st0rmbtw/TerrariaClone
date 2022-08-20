use bevy::{prelude::*, window::PresentMode, asset::AssetServerSettings, render::texture::ImageSettings};
use bevy_rapier2d::plugin::{RapierPhysicsPlugin, NoUserData, RapierConfiguration};
use bevy_tweening::TweeningPlugin;
use game::{plugins::{PlayerPlugin, FpsPlugin, WorldPlugin, DebugPlugin, AssetsPlugin, SetupPlugin, MenuPlugin}, state::GameState};

fn main() {
    let mut app = App::new();

    app
        .insert_resource(WindowDescriptor {
            title: "Terraria".to_string(),
            present_mode: PresentMode::Immediate,
            ..default()
        })
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(ClearColor(Color::rgb(110. / 255., 151. / 255., 244. / 255.)))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0., -30.),
            ..default()
        })
        .add_state(GameState::MainMenu)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(SetupPlugin)
        .add_plugin(MenuPlugin);
        // .add_plugin(WorldPlugin)
        // .add_plugin(PlayerPlugin)
        // .add_plugin(FpsPlugin);
    

    #[cfg(debug_assertions)]
    app.add_plugin(DebugPlugin);

    app.run();
}