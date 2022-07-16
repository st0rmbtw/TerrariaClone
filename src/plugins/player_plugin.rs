use std::time::Duration;

use bevy::{prelude::*, sprite::Anchor, math::XY};
use bevy_rapier2d::{prelude::{RigidBody, Velocity, Sleeping, Ccd, Collider, ActiveEvents, LockedAxes}, pipeline::QueryFilter, plugin::RapierContext, math::Vect};

const SPRITE_WIDTH: f32 = 37.;
const SPRITE_HEIGHT: f32 = 53.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_system(check_is_on_ground)
            .add_system(update)
            .add_system(animate_sprite)
            .add_system(update_movement_state)
            .add_system(update_coords_text);
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerCoords;

#[derive(Component)]
struct Movement {
    direction: FaceDirection,
    state: MovementState,
    is_on_ground: bool
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            direction: FaceDirection::LEFT, 
            state: MovementState::IDLE,
            is_on_ground: false
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
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
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
    
        .insert(Player)
        .insert(Movement::default())
        .insert(AnimationTimer(Timer::new(Duration::from_millis(50), true)))

        // RigidBody
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(LockedAxes::ROTATION_LOCKED)
        .with_children(|children| {

            // Collider
            children.spawn()
                .insert(Collider::cuboid(SPRITE_WIDTH / 2. - 1., SPRITE_HEIGHT / 2.))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert_bundle(TransformBundle::from(Transform::from_xyz(SPRITE_WIDTH / 2., SPRITE_HEIGHT / 2., 0.)));
        });

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
    mut query: Query<(&mut Velocity, &Movement), With<Player>>,
) {
    let (mut velocity, movement) = query.single_mut();

    let left = keyinput.pressed(KeyCode::A) || keyinput.pressed(KeyCode::Left);
    let right = keyinput.pressed(KeyCode::D) || keyinput.pressed(KeyCode::Right);
    let up = (keyinput.pressed(KeyCode::Space) || keyinput.pressed(KeyCode::Up)) && movement.is_on_ground;

    let x_axis = -(left as i8) + right as i8;
    let y_axis = up as i8;
    
    let delta = Vec2::new(x_axis as f32, y_axis as f32);

    velocity.linvel = delta * 200.;
}

fn check_is_on_ground(
    rapier_context: Res<RapierContext>,
    mut query: Query<(&Transform, &mut Movement), With<Player>>
) {
    let (transform, mut movement) = query.single_mut();

    let cast_result = rapier_context.cast_ray(Vect::new(transform.translation.x - (SPRITE_WIDTH / 2.), transform.translation.y), Vect::new(0., -1.), 1., false, QueryFilter::default());

    movement.is_on_ground = match cast_result {
        Some(_) => true,
        None => false,
    };
}

fn update_coords_text(
    mut text_query: Query<(&mut Text, &mut Transform), (With<PlayerCoords>, Without<Player>)>,
    mut player_query: Query<&Transform, With<Player>>
) {
    let transform = player_query.single_mut();
    let (mut player_coords, mut text_transform) = text_query.single_mut();

    let x = transform.translation.x;
    let y = transform.translation.y;

    let mut new_translation = Vec3::from(transform.translation);

    new_translation.y += SPRITE_HEIGHT + 20.;
    new_translation.x -= SPRITE_WIDTH / 2.;

    player_coords.sections[0].value = format!("({:.1}, {:.1})", x, y);
    text_transform.translation = new_translation;
}

fn update_movement_state(
    mut query: Query<(&mut Movement, &Velocity), With<Player>>
) {
    let (mut movement, velocity) = query.single_mut();

    movement.state = match velocity.linvel.into() {
        XY { x, .. } if x != 0. => {
            movement.direction = if x > 0. {
                FaceDirection::RIGHT
            } else {
                FaceDirection::LEFT
            };

            MovementState::RUNNING
        },
        XY { y, ..  } if y != 0. => MovementState::FLYING,
        _ => MovementState::IDLE,
    };
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