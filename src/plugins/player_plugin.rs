use std::time::Duration;

use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::{RigidBody, Velocity, Sleeping, Ccd};

const SPRITE_WIDTH: f32 = 38.;
const SPRITE_HEIGHT: f32 = 53.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_system(update)
            .add_system(animate_sprite)
            .add_system(update_coords_text);
    }
}

#[derive(Component)]
struct Player {
    nickname: String,
    sprite_size: Vec2
}

#[derive(Component)]
struct PlayerCoords;

#[derive(Component)]
struct Movement {
    coords: (f32, f32),
    direction: FaceDirection,
    state: MovementState
}

impl Default for Movement {
    fn default() -> Self {
        Self { 
            coords: (0., 0.), 
            direction: FaceDirection::LEFT, 
            state: MovementState::IDLE
        }
    }
}

enum MovementState {
    IDLE,
    RUNNING,
    FLYING
}

#[derive(PartialEq, Eq)]
enum FaceDirection {
    LEFT,
    RIGHT
}

impl FaceDirection {
    fn is_right(&self) -> bool {
        *self == FaceDirection::RIGHT
    }

    fn is_left(&self) -> bool {
        *self == FaceDirection::LEFT
    }
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {
    let texture_handle = assets.load("sprites/npc_22.png");
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle, Vec2::new(SPRITE_WIDTH, SPRITE_HEIGHT), 1, 16, Vec2::new(0., 3.)
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            ..default()
        })
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(Player {
            nickname: "Test nickname".to_string(),
            sprite_size: Vec2::new(SPRITE_WIDTH, SPRITE_HEIGHT)
        })
        .insert(Movement::default())
        .insert(AnimationTimer(Timer::new(Duration::from_millis(50), true)))

        // RigidBody
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled());

        // // Collider
        // .insert(Collider::cuboid(2., 1.))
        // .insert(ActiveEvents::COLLISION_EVENTS);

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section(
                "", 
                TextStyle {
                    font: assets.load("fonts/andyb.ttf"),
                    font_size: 20.,
                    color: Color::WHITE
                },
                TextAlignment::default()
            ),
            ..default()
        })
        .insert(PlayerCoords);

}

fn update(
    keyinput: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Movement), With<Player>>
) {
    let (mut velocity, mut movement) = query.single_mut();

    let left = keyinput.pressed(KeyCode::A) || keyinput.pressed(KeyCode::Left);
    let right = keyinput.pressed(KeyCode::D) || keyinput.pressed(KeyCode::Right);

    movement.state = MovementState::IDLE;

    if left {
        movement.direction = FaceDirection::LEFT;
        movement.state = MovementState::RUNNING;
    }

    if right {
        movement.direction = FaceDirection::RIGHT;
        movement.state = MovementState::RUNNING;
    }

    let x_axis = -(left as i8) + right as i8;

    let mut delta = Vec2::new(x_axis as f32, 0.);
    if delta != Vec2::ZERO {
        delta /= delta.length()
    }

    velocity.linvel = delta * 100.;
}

fn update_coords_text(
    mut text_query: Query<(&mut Text, &mut Transform), (With<PlayerCoords>, Without<Player>)>,
    mut player_query: Query<(&Transform, &Player)>
) {
    let (transform, player) = player_query.single_mut();
    let (mut player_coords, mut text_transform) = text_query.single_mut();

    let x = transform.translation.x;
    let y = transform.translation.y;

    let mut new_translation = Vec3::from(transform.translation);

    new_translation.y = player.sprite_size.y + 20.;

    player_coords.sections[0].value = format!("({:.1}, {:.1})", x, y);
    text_transform.translation = new_translation;
}

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
        &Movement
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle, movement) in query.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();

            sprite.flip_x = movement.direction.is_right();
            sprite.anchor = Anchor::BottomLeft;

            sprite.index = match movement.state {
                MovementState::IDLE => 0,
                MovementState::RUNNING => {
                    ((sprite.index + 1) % texture_atlas.textures.len()).clamp(2, 16)
                },
                MovementState::FLYING => 1,
            }
        }
    }
}