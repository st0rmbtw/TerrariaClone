use bevy::{prelude::{Name, Color, default, Transform, Handle, ChildBuilder, Visibility, Component}, sprite::{SpriteSheetBundle, TextureAtlasSprite, TextureAtlas, SpriteBundle, Sprite, Anchor}};

use crate::plugins::{inventory::{UseItemAnimationData, ItemInHand}, world::WORLD_RENDER_LAYER, assets::PlayerAssets};

use super::{MovementAnimationBundle, WalkingAnimationData, FlyingAnimationData, IdleAnimationData};

#[derive(Component)]
pub(crate) struct ChangeFlip;

#[derive(Component)]
pub(crate) struct PlayerSpriteBody;

#[derive(Component)]
pub(super) struct PlayerSpriteFeet;

fn spawn_player_hair(commands: &mut ChildBuilder, sprite_bundle: SpriteSheetBundle) {
    commands.spawn((
        Name::new("Player hair"),
        ChangeFlip,
        PlayerSpriteBody,
        MovementAnimationBundle::default(),
        WORLD_RENDER_LAYER,
        sprite_bundle,
    ));
}

fn spawn_player_skull(commands: &mut ChildBuilder, sprite_bundle: SpriteSheetBundle) {
    commands.spawn((
        Name::new("Player head"),
        ChangeFlip,
        PlayerSpriteBody,
        MovementAnimationBundle::default(),
        WORLD_RENDER_LAYER,
        sprite_bundle
    ));
}

fn spawn_player_eyes(
    commands: &mut ChildBuilder,
    left_eye_sprite: SpriteSheetBundle,
    right_eye_sprite: SpriteSheetBundle
) {
    commands.spawn((
        Name::new("Player left eye"),
        ChangeFlip,
        PlayerSpriteBody,
        WORLD_RENDER_LAYER,
        MovementAnimationBundle {
            walking: WalkingAnimationData {
                offset: 6,
                count: 14,
            },
            ..default()
        },
        left_eye_sprite
    ));

    commands.spawn((
        Name::new("Player right eye"),
        ChangeFlip,
        PlayerSpriteBody,
        WORLD_RENDER_LAYER,
        MovementAnimationBundle {
            walking: WalkingAnimationData {
                offset: 6,
                count: 14,
            },
            ..default()
        },
        right_eye_sprite
    ));
}

fn spawn_player_left_hand(commands: &mut ChildBuilder, left_shoulder: Handle<TextureAtlas>, left_hand: Handle<TextureAtlas>, z: f32) {
    commands.spawn((
        Name::new("Player left shoulder"),
        ChangeFlip,
        PlayerSpriteBody,
        UseItemAnimationData(2),
        MovementAnimationBundle {
            walking: WalkingAnimationData {
                offset: 13,
                ..default()
            },
            flying: FlyingAnimationData(2),
            ..default()
        },
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: Color::rgb(0.58, 0.55, 0.47),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., z),
            texture_atlas: left_shoulder,
            ..default()
        },
        WORLD_RENDER_LAYER
    ));

    commands.spawn((
        Name::new("Player left hand"),
        ChangeFlip,
        PlayerSpriteBody,
        UseItemAnimationData(2),
        MovementAnimationBundle {
            walking: WalkingAnimationData {
                offset: 13,
                ..default()
            },
            flying: FlyingAnimationData(2),
            ..default()
        },
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: Color::rgb(0.92, 0.45, 0.32),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., z),
            texture_atlas: left_hand,
            ..default()
        },
        WORLD_RENDER_LAYER
    ));
}

fn spawn_player_right_hand(commands: &mut ChildBuilder, right_hand: Handle<TextureAtlas>, z: f32) {
    commands.spawn((
        Name::new("Player right hand"),
        ChangeFlip,
        PlayerSpriteBody,
        UseItemAnimationData(15),
        MovementAnimationBundle {
            idle: IdleAnimationData(14),
            flying: FlyingAnimationData(13),
            ..default()
        },
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: Color::rgb(0.92, 0.45, 0.32),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., z),
            texture_atlas: right_hand,
            ..default()
        },
        WORLD_RENDER_LAYER
    ));
}

fn spawn_player_chest(commands: &mut ChildBuilder, chest: Handle<TextureAtlas>, z: f32) {
    commands.spawn((
        Name::new("Player chest"),
        ChangeFlip,
        PlayerSpriteBody,
        MovementAnimationBundle::default(),
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 0,
                color: Color::rgb(0.58, 0.55, 0.47),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., z),
            texture_atlas: chest,
            ..default()
        },
        WORLD_RENDER_LAYER
    ));
}

pub(super) fn spawn_player_feet(commands: &mut ChildBuilder, feet: Handle<TextureAtlas>, z: f32) {
    commands.spawn((
        Name::new("Player feet"),
        ChangeFlip,
        PlayerSpriteBody,
        PlayerSpriteFeet,
        MovementAnimationBundle {
            walking: WalkingAnimationData {
                offset: 6,
                ..default()
            },
            flying: FlyingAnimationData(5),
            ..default()
        },
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: Color::rgb_u8(190, 190, 156),
                ..default()
            },
            texture_atlas: feet,
            transform: Transform::from_xyz(0., 0., z),
            ..default()
        },
        WORLD_RENDER_LAYER
    ));
}

pub(super) fn spawn_player_item_in_hand(commands: &mut ChildBuilder, z: f32) {
    commands.spawn((
        Name::new("Item in hand"),
        ChangeFlip,
        PlayerSpriteBody,
        ItemInHand,
        SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(0., 0., z),
            ..default()
        },
        WORLD_RENDER_LAYER
    ));
}

#[inline]
pub(crate) fn player_hair_sprite(player_assets: &PlayerAssets, z: f32) -> SpriteSheetBundle {
    SpriteSheetBundle {
        sprite: TextureAtlasSprite {
            color: Color::rgb(0.55, 0.23, 0.14),
            ..default()
        },
        transform: Transform::from_xyz(0., 0., z),
        texture_atlas: player_assets.hair.clone_weak(),
        ..default()
    }
}

#[inline]
pub(crate) fn player_skull_sprite(player_assets: &PlayerAssets, z: f32) -> SpriteSheetBundle {
    SpriteSheetBundle {
        sprite: TextureAtlasSprite {
            color: Color::rgb(0.92, 0.45, 0.32),
            ..default()
        },
        texture_atlas: player_assets.head.clone_weak(),
        transform: Transform::from_xyz(0., 0., z),
        ..default()
    }
}

#[inline]
pub(crate) fn player_left_eye(player_assets: &PlayerAssets, z: f32) -> SpriteSheetBundle {
    SpriteSheetBundle {
        transform: Transform::from_xyz(0., 0., z),
        texture_atlas: player_assets.eyes_1.clone_weak(),
        ..default()
    }
}

#[inline]
pub(crate) fn player_right_eye(player_assets: &PlayerAssets, z: f32) -> SpriteSheetBundle {
    SpriteSheetBundle {
        sprite: TextureAtlasSprite {
            color: Color::rgb_u8(89, 76, 64),
            ..default()
        },
        transform: Transform::from_xyz(0., 0., z),
        texture_atlas: player_assets.eyes_2.clone_weak(),
        ..default()
    }
}

#[inline]
pub(super) fn spawn_player_head(parent: &mut ChildBuilder, player_assets: &PlayerAssets) {
    spawn_player_hair(parent, player_hair_sprite(player_assets, 0.5));
    spawn_player_skull(parent, player_skull_sprite(player_assets, 0.1));
    spawn_player_eyes(parent,
        player_left_eye(player_assets, 0.2),
        player_right_eye(player_assets, 0.2)
    );
}

#[inline]
pub(super) fn spawn_player_body(parent: &mut ChildBuilder, player_assets: &PlayerAssets) {
    spawn_player_left_hand(parent, player_assets.left_shoulder.clone_weak(), player_assets.left_hand.clone_weak(), 0.9);
    spawn_player_right_hand(parent, player_assets.right_arm.clone_weak(), 0.);
    spawn_player_chest(parent, player_assets.chest.clone_weak(), 0.1);
}