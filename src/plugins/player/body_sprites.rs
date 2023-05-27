use bevy::{prelude::{Name, Color, default, Transform, Handle, ChildBuilder, Visibility, Component}, sprite::{SpriteSheetBundle, TextureAtlasSprite, TextureAtlas, SpriteBundle, Sprite, Anchor}};

use crate::plugins::inventory::{UseItemAnimationData, ItemInHand};

use super::{MovementAnimationBundle, WalkingAnimationData, FlyingAnimationData, IdleAnimationData};

#[derive(Component)]
pub(super) struct ChangeFlip;

#[derive(Component)]
pub(crate) struct PlayerSpriteBody;

#[derive(Component)]
pub(super) struct PlayerSpriteFeet;

pub(super) fn spawn_player_hair(commands: &mut ChildBuilder, hair: Handle<TextureAtlas>) {
    commands.spawn((
        Name::new("Player hair"),
        ChangeFlip,
        PlayerSpriteBody,
        MovementAnimationBundle::default(),
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: Color::rgb(0.55, 0.23, 0.14),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 0.1),
            texture_atlas: hair,
            ..default()
        }
    ));
}

pub(super) fn spawn_player_head(commands: &mut ChildBuilder, head: Handle<TextureAtlas>) {
    commands.spawn((
        Name::new("Player head"),
        ChangeFlip,
        PlayerSpriteBody,
        MovementAnimationBundle::default(),
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: Color::rgb(0.92, 0.45, 0.32),
                ..default()
            },
            texture_atlas: head,
            transform: Transform::from_xyz(0., 0., 0.003),
            ..default()
        }
    ));
}

pub(super) fn spawn_player_eyes(commands: &mut ChildBuilder, left_eye: Handle<TextureAtlas>, right_eye: Handle<TextureAtlas>) {
    commands.spawn((
        Name::new("Player left eye"),
        ChangeFlip,
        PlayerSpriteBody,
        MovementAnimationBundle {
            walking: WalkingAnimationData {
                offset: 6,
                count: 14,
            },
            ..default()
        },
        SpriteSheetBundle {
            transform: Transform::from_xyz(0., 0., 0.1),
            texture_atlas: left_eye,
            ..default()
        },
    ));

    commands.spawn((
        Name::new("Player right eye"),
        ChangeFlip,
        PlayerSpriteBody,
        MovementAnimationBundle {
            walking: WalkingAnimationData {
                offset: 6,
                count: 14,
            },
            ..default()
        },
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                color: Color::rgb_u8(89, 76, 64),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 0.01),
            texture_atlas: right_eye,
            ..default()
        },
    ));
}

pub(super) fn spawn_player_left_hand(commands: &mut ChildBuilder, left_shoulder: Handle<TextureAtlas>, left_hand: Handle<TextureAtlas>) {
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
            transform: Transform::from_xyz(0., 0., 0.2),
            texture_atlas: left_shoulder,
            ..default()
        },
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
            transform: Transform::from_xyz(0., 0., 0.2),
            texture_atlas: left_hand,
            ..default()
        },
    ));
}

pub(super) fn spawn_player_right_hand(commands: &mut ChildBuilder, right_hand: Handle<TextureAtlas>) {
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
            transform: Transform::from_xyz(0., 0., 0.001),
            texture_atlas: right_hand,
            ..default()
        },
    ));
}

pub(super) fn spawn_player_chest(commands: &mut ChildBuilder, chest: Handle<TextureAtlas>) {
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
            transform: Transform::from_xyz(0., 0., 0.002),
            texture_atlas: chest,
            ..default()
        },
    ));
}

pub(super) fn spawn_player_feet(commands: &mut ChildBuilder, feet: Handle<TextureAtlas>) {
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
            transform: Transform::from_xyz(0., 0., 0.15),
            ..default()
        }
    ));
}

pub(super) fn spawn_player_item_in_hand(commands: &mut ChildBuilder) {
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
            transform: Transform::from_xyz(0., 0., 0.15),
            ..default()
        }
    ));
}