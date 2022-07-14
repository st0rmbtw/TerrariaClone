use bevy::{prelude::*, window::PresentMode};
use bevy_rapier2d::plugin::{RapierPhysicsPlugin, NoUserData};
use game::plugins::{
    setup_plugin::SetupPlugin,
    player_plugin::PlayerPlugin, 
    fps_plugin::FpsPlugin
};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Terraria".to_string(),
            present_mode: PresentMode::Fifo,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(SetupPlugin)
        .add_plugin(PlayerPlugin)
        // .add_plugin(FpsPlugin)
        .run();
}