use bevy::{prelude::{Plugin, Commands, Res, AssetServer, Transform, default, ImageBundle, Handle, Image, Color, NodeBundle, ParallelSystemDescriptorCoercion}, math::{Size, Rect}, ui::{Style, Val, AlignItems, JustifyContent}, hierarchy::BuildChildren};

const HOTBAR_SIZE: f32 = 36.;

pub const SPAWN_PLAYER_UI_LABEL: &str = "spawn_player_ui";

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(setup.label(SPAWN_PLAYER_UI_LABEL));
    }
}


fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>
) {
    let texture: Handle<Image> = assets.load("sprites/UI/InnerPanelBackground.png");

    let cell_count = (9 * HOTBAR_SIZE as i16) / 2;

    commands.spawn_bundle(NodeBundle {
        style: Style {
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexEnd,
            margin: Rect { 
                top: Val::Px(20.),
                left: Val::Px(20.),
                ..default()
            },
            ..default()
        },
        color: Color::rgba(0., 0., 0., 0.).into(),
        ..default()
    }).with_children(|children| {
        for x in (-cell_count..cell_count).step_by(HOTBAR_SIZE as usize) {
            children.spawn_bundle(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(HOTBAR_SIZE), Val::Px(HOTBAR_SIZE)),
                    margin: Rect {
                        left: Val::Px(2.5),
                        right: Val::Px(2.5),
                        ..default()
                    },
                    ..default()
                },
                image: texture.clone().into(),
                transform: Transform::from_xyz(x as f32, 0., 0.),
                ..default()
            });
        }
    });
}