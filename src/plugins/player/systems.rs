use autodefault::autodefault;
use bevy::{prelude::*, sprite::Anchor, math::Vec3Swizzles};
use bevy_hanabi::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    state::{GameState, MovementState}, 
    plugins::{
        world::{WorldData, TILE_SIZE, BlockPlaceEvent}, 
        assets::{PlayerAssets, ItemAssets}, 
        inventory::{SelectedItem, Inventory}, 
        cursor::CursorPosition
    }, 
    world_generator::{WORLD_SIZE_X, WORLD_SIZE_Y}, 
    util::{move_towards, map_range, get_rotation_by_direction, get_tile_coords}, 
    items::{get_animation_points, Item}
};

use super::*;

#[autodefault(except(GroundSensor, PlayerParticleEffects))]
pub fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let player = commands
        .spawn()
        .insert(Player)
        .insert_bundle(SpatialBundle {
            transform: Transform::from_xyz(WORLD_SIZE_X as f32 * 16. / 2., WORLD_SIZE_Y as f32 * TILE_SIZE, 3.)
        })
        .insert(Name::new("Player"))
        .insert(MovementState::default())
        .insert(FaceDirection::default())
        .with_children(|cmd| {
            // region: Hair
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(0.55, 0.23, 0.14),
                },
                transform: Transform::from_xyz(0., 0., 0.1),
                texture_atlas: player_assets.hair.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle::default())
            .insert(Name::new("Player hair"));
            // endregion

            // region: Head
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(0.92, 0.45, 0.32),
                },
                texture_atlas: player_assets.head.clone(),
                transform: Transform::from_xyz(0., 0., 0.003),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle::default())
            .insert(Name::new("Player head"));
            // endregion

            // region: Eyes
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::WHITE,
                },
                transform: Transform::from_xyz(0., 0., 0.1),
                texture_atlas: player_assets.eyes_1.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData {
                    offset: 6,
                    count: 14,
                }
            })
            .insert(Name::new("Player left eye"));

            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(89. / 255., 76. / 255., 64. / 255.),
                },
                transform: Transform::from_xyz(0., 0., 0.01),
                texture_atlas: player_assets.eyes_2.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData {
                    offset: 6,
                    count: 14,
                }
            })
            .insert(Name::new("Player right eye"));

            // endregion

            // region: Arms
            // region: Left arm
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(0.58, 0.55, 0.47),
                },
                transform: Transform::from_xyz(0., -8., 0.2),
                texture_atlas: player_assets.left_shoulder.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData {
                    offset: 13,
                    count: 13,
                },
                flying: FlyingAnimationData(2)
            })
            .insert(UseItemAnimationData(2))
            .insert(Name::new("Player left shoulder"));

            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(0.92, 0.45, 0.32),
                },
                transform: Transform::from_xyz(0., -8., 0.2),
                texture_atlas: player_assets.left_hand.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData {
                    offset: 13,
                    count: 13,
                },
                flying: FlyingAnimationData(2)
            })
            .insert(UseItemAnimationData(2))
            .insert(Name::new("Player left hand"));
            // endregion

            // region: Right arm
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(0.92, 0.45, 0.32),
                },
                transform: Transform::from_xyz(0., -20., 0.001),
                texture_atlas: player_assets.right_arm.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData { count: 13 },
                idle: IdleAnimationData(14),
                flying: FlyingAnimationData(13),
            })
            .insert(UseItemAnimationData(15))
            .insert(Name::new("Player right hand"));
            // endregion

            // endregion

            // region: Chest
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: 0,
                    color: Color::rgb(0.58, 0.55, 0.47),
                },
                transform: Transform::from_xyz(0., 0., 0.002),
                texture_atlas: player_assets.chest.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle::default())
            .insert(Name::new("Player chest"));
            // endregion

            // region: Feet
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(190. / 255., 190. / 255., 156. / 255.),
                },
                texture_atlas: player_assets.feet.clone(),
                transform: Transform::from_xyz(0., 0., 0.15),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData {
                    offset: 6,
                    count: 13,
                },
                flying: FlyingAnimationData(5),
            })
            .insert(Name::new("Player feet"));
            // endregion

            // region: Using item
            cmd.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                },
                visibility: Visibility {
                    is_visible: false
                },
                transform: Transform::from_xyz(0., 0., 0.15),
            })
            .insert(ChangeFlip)
            .insert(UsingItemMarker)
            .insert(Name::new("Using item"));

            // endregion

            // cmd.spawn_bundle(SpriteBundle {
            //     sprite: Sprite {
            //         color: Color::RED,
            //         custom_size: Some(Vec2::new(PLAYER_SPRITE_WIDTH, 1.))
            //     },
            //     transform: Transform::from_xyz(0., -PLAYER_SPRITE_HEIGHT / 2., 0.5),
            // });

            // cmd.spawn_bundle(SpriteBundle {
            //     sprite: Sprite {
            //         color: Color::RED,
            //         custom_size: Some(Vec2::new(PLAYER_SPRITE_WIDTH, 1.))
            //     },
            //     transform: Transform::from_xyz(0., PLAYER_SPRITE_HEIGHT / 2., 0.5),
            // });

            // cmd.spawn_bundle(SpriteBundle {
            //     sprite: Sprite {
            //         color: Color::RED,
            //         custom_size: Some(Vec2::new(1., PLAYER_SPRITE_HEIGHT))
            //     },
            //     transform: Transform::from_xyz(-PLAYER_SPRITE_WIDTH / 2., 0., 0.5),
            // });

            // cmd.spawn_bundle(SpriteBundle {
            //     sprite: Sprite {
            //         color: Color::RED,
            //         custom_size: Some(Vec2::new(1., PLAYER_SPRITE_HEIGHT))
            //     },
            //     transform: Transform::from_xyz(PLAYER_SPRITE_WIDTH / 2., 0., 0.5),
            // });
        })
        .id();

    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(114. / 255., 81. / 255., 56. / 255., 1.));

    let spawner = Spawner::rate(20.0.into());

    // Create the effect asset
    let effect = effects.add(
        EffectAsset {
            name: "MyEffect".to_string(),
            // Maximum number of particles alive at a time
            capacity: 30,
            spawner,
        }
        .init(PositionCone3dModifier {
            base_radius: 0.5,
            top_radius: 0.,
            height: 1.,
            dimension: ShapeDimension::Volume,
            speed: 10.0.into(),
        })
        .update(AccelModifier {
            accel: Vec3::new(0., 0., 0.),
        })
        // Render the particles with a color gradient over their
        // lifetime.
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::constant(Vec2::splat(3.)),
        })
        .init(ParticleLifetimeModifier { lifetime: 0.1 })
        .render(ColorOverLifetimeModifier { gradient }),
    );

    let effect_entity = commands
        .spawn_bundle(ParticleEffectBundle::new(effect).with_spawner(spawner))
        .insert(Name::new("Particle Spawner"))
        .id();

    commands.entity(player).add_child(effect_entity);

    commands.entity(player).insert(PlayerParticleEffects {
        walking: effect_entity,
    });
}

#[cfg(feature = "debug_movement")]
pub fn debug_horizontal_movement(
    axis: Res<InputAxis>,
    mut velocity: ResMut<PlayerVelocity>
) {
    velocity.x = axis.x * 10.;
}

#[cfg(feature = "debug_movement")]
pub fn debug_vertical_movement(
    input: Res<Input<KeyCode>>,
    mut velocity: ResMut<PlayerVelocity>
) {
    let up = input.pressed(KeyCode::W);
    let down = input.pressed(KeyCode::S);

    let y = -(down as i8) + up as i8;

    velocity.y = y as f32 * 10.;
}

pub fn horizontal_movement(
    axis: Res<InputAxis>,
    mut velocity: ResMut<PlayerVelocity>
) {
    if axis.is_moving() {
        velocity.x += axis.x * ACCELERATION;
        velocity.x = velocity.x.clamp(-MAX_RUN_SPEED, MAX_RUN_SPEED);
    } else {
        velocity.x = move_towards(velocity.x, 0., SLOWDOWN);
    }
}

pub fn gravity(
    mut velocity: ResMut<PlayerVelocity>,
) {
    velocity.y -= GRAVITY;

    if velocity.y < -MAX_FALL_SPEED {
        velocity.y = -MAX_FALL_SPEED;
    }
}

pub fn update_jump(
    input: Res<Input<KeyCode>>,
    collisions: Res<Collisions>,
    mut velocity: ResMut<PlayerVelocity>,
    mut player_controller: ResMut<PlayerController>,
) {
    if input.just_pressed(KeyCode::Space) && collisions.bottom {
        player_controller.jump = JUMP_HEIGHT;
        velocity.y = JUMP_SPEED;
    }

    if input.pressed(KeyCode::Space) {
        if player_controller.jump > 0 {
            if velocity.y == 0. {
                player_controller.jump = 0;
            } else {
                velocity.y = JUMP_SPEED;

                player_controller.jump -= 1;
            }
        }
    } else {
        player_controller.jump = 0;
    }
}

pub fn move_character(
    velocity: Res<PlayerVelocity>,
    mut player_query: Query<&mut Transform, With<Player>>,
    #[cfg(feature = "debug")]
    mut lines: ResMut<bevy_prototype_debug_lines::DebugLines>
) {
    let mut transform = player_query.single_mut();

    const MIN: f32 = PLAYER_SPRITE_WIDTH * 0.75 / 2. - TILE_SIZE / 2.;
    const MAX: f32 = WORLD_SIZE_X as f32 * TILE_SIZE - PLAYER_SPRITE_WIDTH * 0.75 / 2. - TILE_SIZE / 2.;

    let raw = transform.translation.xy() + velocity.0;

    transform.translation.x = raw.x.clamp(MIN, MAX);
    transform.translation.y = raw.y;
    
    #[cfg(feature = "debug")] {
        // let player_rect = get_player_rect(raw, 1.);

        // let bottom = (player_rect.bottom / TILE_SIZE).floor() * TILE_SIZE + TILE_SIZE / 2.;
        // let top = (player_rect.top / TILE_SIZE).ceil() * TILE_SIZE - TILE_SIZE / 2.;
        // let left = (player_rect.left / TILE_SIZE).ceil() * TILE_SIZE + TILE_SIZE / 2.;
    
        // lines.line_colored(
        //     Vec3::new(0., bottom, 2.),
        //     Vec3::new(WORLD_SIZE_X as f32 * TILE_SIZE, bottom, 2.),
        //     0.,
        //     Color::RED
        // );

        // lines.line_colored(
        //     Vec3::new(0., top, 2.),
        //     Vec3::new(WORLD_SIZE_X as f32 * TILE_SIZE, top, 2.),
        //     0.,
        //     Color::RED
        // );

        // lines.line_colored(
        //     Vec3::new(left, 0., 2.),
        //     Vec3::new(left, WORLD_SIZE_Y as f32 * TILE_SIZE, 2.),
        //     0.,
        //     Color::RED
        // );
    }
}

pub fn collide(
    player_query: Query<&Transform, With<Player>>,
    world_data: Res<WorldData>,
    mut velocity: ResMut<PlayerVelocity>,
    mut collisions: ResMut<Collisions>
) {
    let transform = player_query.single();

    let (new_velocity, new_collisions) = get_collisions(transform.translation.xy(), velocity.0, &world_data);

    velocity.0 = new_velocity;
    *collisions = new_collisions;
}

pub fn spawn_particles(
    player: Query<(&MovementState, &FaceDirection, &PlayerParticleEffects), With<Player>>,
    mut effects: Query<(&mut ParticleEffect, &mut Transform)>,
) {
    for (movement_state, face_direction, particle_effects) in &player {
        let (mut effect, mut effect_transform) = effects.get_mut(particle_effects.walking).unwrap();

        effect_transform.translation = match face_direction {
            FaceDirection::LEFT => Vec3::new(0., -PLAYER_SPRITE_HEIGHT / 2., 0.),
            FaceDirection::RIGHT => Vec3::new(0., -PLAYER_SPRITE_HEIGHT / 2., 0.),
        };

        effect
            .maybe_spawner()
            .unwrap()
            .set_active(*movement_state == MovementState::WALKING);
    }
}

pub fn update_movement_state(
    player_controller: Res<PlayerController>,
    velocity: Res<PlayerVelocity>,
    mut query: Query<&mut MovementState, With<Player>>,
) {
    let mut movement_state = query.single_mut();

    *movement_state = match velocity.0 {
        Vec2 { x, y } if x != 0. && y == 0. => MovementState::WALKING,
        Vec2 { y, .. } if y != 0. || player_controller.jump > 0 => MovementState::FLYING,
        _ => MovementState::IDLE
    };
}

pub fn update_face_direction(axis: Res<InputAxis>, mut query: Query<&mut FaceDirection>) {
    let mut direction = query.single_mut();
    let axis: &InputAxis = &axis;

    if let Some(new_direction) = axis.into() {
        if *direction != new_direction {
            *direction = new_direction;
        }
    }
}

pub fn update_axis(input: Res<Input<KeyCode>>, mut axis: ResMut<InputAxis>) {
    let left = input.pressed(KeyCode::A);
    let right = input.pressed(KeyCode::D);

    let x = -(left as i8) + right as i8;

    axis.x = x as f32;
}

pub fn update_movement_animation_timer_duration(
    velocity: Res<PlayerVelocity>,
    mut timer: ResMut<AnimationTimer>,
) {
    if velocity.x != 0. {
        let mut time = 100. / velocity.x.abs();

        if time < 1. {
            time = 1.;
        }

        timer.set_duration(Duration::from_millis(time as u64));
    }
}

pub fn update_movement_animation_index(
    time: Res<Time>,
    mut timer: ResMut<AnimationTimer>,
    mut index: ResMut<MovementAnimationIndex>,
) {
    if timer.tick(time.delta()).just_finished() {
        index.0 = (index.0 + 1) % WALKING_ANIMATION_MAX_INDEX;
    }
}

pub fn flip_player(
    player_query: Query<&FaceDirection, (With<Player>, Changed<FaceDirection>)>,
    mut sprite_query: Query<&mut TextureAtlasSprite, With<ChangeFlip>>,
) {
    let direction = player_query.get_single();

    if let Ok(direction) = direction {
        sprite_query.for_each_mut(|mut sprite| {
            sprite.flip_x = direction.is_left();
        });
    }
}

pub fn walking_animation(
    index: Res<MovementAnimationIndex>,
    mut query: Query<
        (&mut TextureAtlasSprite, &WalkingAnimationData),
        With<PlayerBodySprite>,
    >,
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        let walking_anim_offset = anim_data.offset;
        let walking_anim_count = anim_data.count;

        sprite.index = walking_anim_offset + map_range(
            (0, WALKING_ANIMATION_MAX_INDEX),
            (0, walking_anim_count),
            index.0,
        );
    });
}

pub fn player_using_item(
    input: Res<Input<MouseButton>>,
    selected_item: Res<SelectedItem>,
    mut anim: ResMut<UseItemAnimation>,
) {
    let using_item = input.pressed(MouseButton::Left) && selected_item.is_some();

    if using_item {
        anim.0 = true;
    }
}

pub fn set_using_item_visibility(
    anim: Res<UseItemAnimation>,
    mut using_item_query: Query<&mut Visibility, With<UsingItemMarker>>,
) {
    let mut visibility = using_item_query.single_mut();
    visibility.is_visible = anim.0;
}

pub fn set_using_item_image(
    item_assets: Res<ItemAssets>,
    selected_item: Res<SelectedItem>,
    mut using_item_query: Query<&mut Handle<Image>, With<UsingItemMarker>>,
) {
    let mut image = using_item_query.single_mut();

    if let Some(item_stack) = selected_item.0 {
        *image = item_assets.get_by_item(item_stack.item);
    }
}

pub fn set_using_item_position(
    index: Res<UseItemAnimationIndex>,
    selected_item: Res<SelectedItem>,
    mut using_item_query: Query<&mut Transform, With<UsingItemMarker>>,
    player_query: Query<&FaceDirection, With<Player>>,
) {
    let mut transform = using_item_query.single_mut();
    let direction = player_query.single();

    if let Some(item_stack) = selected_item.0 {
        let position = get_animation_points(item_stack.item)[index.0];

        transform.translation.x = position.x * f32::from(*direction);
        transform.translation.y = position.y;
    }
}

pub fn set_using_item_rotation_on_player_direction_change(
    player_query: Query<&FaceDirection, (With<Player>, Changed<FaceDirection>)>,
    mut using_item_query: Query<&mut Transform, With<UsingItemMarker>>,
) {
    let player_query_result = player_query.get_single();
    let using_item_query_result = using_item_query.get_single_mut();

    if let Ok(mut transform) = using_item_query_result {
        if let Ok(direction) = player_query_result {
            transform.rotation = get_rotation_by_direction(*direction);
        }
    }
}

pub fn set_using_item_rotation(
    time: Res<Time>,
    index: Res<UseItemAnimationIndex>,
    selected_item: Res<SelectedItem>,
    mut using_item_query: Query<&mut Transform, With<UsingItemMarker>>,
    player_query: Query<&FaceDirection, With<Player>>,
) {
    const ROTATION_STEP: f32 = -11.;

    let direction = player_query.single();
    let mut transform = using_item_query.single_mut();

    if selected_item.is_some() {
        let item_type = selected_item.unwrap().item;
        let direction_f = f32::from(*direction);

        let position = get_animation_points(item_type)[index.0];

        if index.0 == 0 && index.is_changed() {
            transform.rotation = get_rotation_by_direction(*direction);
        }

        transform.rotate_around(
            position.extend(0.15),
            Quat::from_rotation_z(ROTATION_STEP * direction_f * time.delta_seconds()),
        );
    }
}

pub fn update_use_item_animation_index(
    time: Res<Time>,
    mut index: ResMut<UseItemAnimationIndex>,
    mut timer: ResMut<UseItemAnimationTimer>,
    mut anim: ResMut<UseItemAnimation>,
) {
    if timer.tick(time.delta()).just_finished() {
        index.0 = (index.0 + 1) % USE_ITEM_ANIMATION_FRAMES_COUNT;
    }

    if index.is_changed() && index.0 == 0 {
        anim.0 = false;
    }
}

pub fn use_item_animation(
    index: Res<UseItemAnimationIndex>,
    mut query: Query<(&mut TextureAtlasSprite, &UseItemAnimationData), With<PlayerBodySprite>>,
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        sprite.index = anim_data.0 + index.0;
    });
}

pub fn use_item(
    input: Res<Input<MouseButton>>,
    cursor: Res<CursorPosition>,
    inventory: Res<Inventory>,
    mut block_place_event_writer: EventWriter<BlockPlaceEvent>
) {
    if input.pressed(MouseButton::Left) {
        let selected_item_index = inventory.selected_slot;

        if let Some(item_stack) = inventory.selected_item() {
            match item_stack.item {
                Item::Pickaxe(_) => (),
                Item::Block(block) => {
                    let tile_pos = get_tile_coords(cursor.world_position);
                    block_place_event_writer.send(
                        BlockPlaceEvent { tile_pos, block, inventory_item_index: selected_item_index }
                    );
                },
            }
        }
    }
}

// TODO: Debug function, remove in feature
#[cfg(debug_assertions)]
pub fn set_sprite_index(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut TextureAtlasSprite, &WalkingAnimationData), With<PlayerBodySprite>>,
) {
    query.for_each_mut(|(mut sprite, animation_data)| {
        let anim_offset = animation_data.offset;

        let mut new_sprite_index = sprite.index;

        if input.just_pressed(KeyCode::J) {
            new_sprite_index = sprite.index.checked_sub(1).unwrap_or(0);
        }

        if input.just_pressed(KeyCode::L) {
            new_sprite_index = sprite.index + 1;
        }

        new_sprite_index = new_sprite_index.checked_sub(anim_offset).unwrap_or(0);

        sprite.index = anim_offset + (new_sprite_index % WALKING_ANIMATION_MAX_INDEX);
    });
}