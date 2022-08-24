use bevy::{prelude::*, window::PresentMode, asset::AssetServerSettings, render::texture::ImageSettings};
use game::{parallax::ParallaxPlugin, animation::TweeningPlugin};
use bevy_rapier2d::plugin::{RapierPhysicsPlugin, NoUserData, RapierConfiguration};
use game::{plugins::{PlayerPlugin, FpsPlugin, WorldPlugin, AssetsPlugin, SetupPlugin, MenuPlugin}, state::GameState};
use iyes_loopless::prelude::AppLooplessStateExt;

fn main() {
    let mut app = App::new();

    app
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Terraria".to_string(),
            present_mode: PresentMode::Fifo,
            cursor_visible: false,
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
        .add_loopless_state(GameState::AssetLoading)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(SetupPlugin)
        .add_plugin(ParallaxPlugin {
            initial_speed: 0.3,
        })
        .add_plugin(MenuPlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(FpsPlugin);
    

    #[cfg(debug_assertions)]
    app.add_plugin(game::plugins::DebugPlugin);

    app.run();
}