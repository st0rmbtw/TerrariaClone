use bevy::{prelude::{Plugin, AssetServer, Assets, Handle, App, Image, World}, sprite::TextureAtlas, math::Vec2, text::Font, asset::HandleUntyped};
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use bevy::ecs::world::Mut;

use crate::block::BlockId;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_collection::<BlockAssets>()
            .init_collection::<UiAssets>()
            .init_collection::<PlayerAssets>()
            .init_collection::<FontAssets>()
            .init_collection::<ItemAssets>()
            .init_collection::<CursorAssets>();
    }
}

#[derive(AssetCollection)]
pub struct BlockAssets {
    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., rows = 15, columns = 16, padding_x = 2., padding_y = 2.))]
    #[asset(path = "sprites/tiles/Tiles_0.png")]
    pub dirt: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., rows = 15, columns = 16, padding_x = 2., padding_y = 2.))]
    #[asset(path = "sprites/tiles/Tiles_2.png")]
    pub grass: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., rows = 15, columns = 16, padding_x = 2., padding_y = 2.))]
    #[asset(path = "sprites/tiles/Tiles_1.png")]
    pub stone: Handle<TextureAtlas>,
}

#[derive(AssetCollection)]
pub struct UiAssets {
    #[asset(path = "sprites/ui/InnerPanelBackground.png")]
    pub iner_panel_background: Handle<Image>,

    #[asset(path = "sprites/ui/PlayerBackground.png")]
    pub player_background: Handle<Image>,

    #[asset(path = "sprites/Inventory_Back.png")]
    pub inventory_back: Handle<Image>,

    #[asset(path = "sprites/Inventory_Back14.png")]
    pub selected_inventory_back: Handle<Image>,
}

#[derive(AssetCollection)]
pub struct PlayerAssets {
    #[asset(texture_atlas(tile_size_x = 37., tile_size_y = 53., columns = 1, rows = 16, padding_x = 0., padding_y = 3.))]
    #[asset(path = "sprites/npc_22.png")]
    pub main: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 48., columns = 1, rows = 14, padding_x = 0., padding_y = 0.))]
    #[asset(path = "sprites/player/Player_0_0.png")]
    pub head: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 64., columns = 37, rows = 1, padding_x = 0., padding_y = 0.))]
    #[asset(path = "sprites/player/Player_Left_Hand2.png")]
    pub left_hand: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 64., columns = 18, rows = 1, padding_x = 0., padding_y = 0.))]
    #[asset(path = "sprites/player/Player_Right_Hand2.png")]
    pub right_hand: Handle<TextureAtlas>,
 
    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 48., columns = 1, rows = 14, padding_x = 0., padding_y = 0.))]
    #[asset(path = "sprites/player/Player_Hair_1.png")]
    pub hair: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 64., columns = 1, rows = 14, padding_x = 8., padding_y = 0.))]
    #[asset(path = "sprites/player/Player_Body.png")]
    pub chest: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 64., columns = 1, rows = 19, padding_x = 0., padding_y = 0.))]
    #[asset(path = "sprites/player/Player_0_11.png")]
    pub feet: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 64., columns = 1, rows = 20, padding_x = 0., padding_y = 0.))]
    #[asset(path = "sprites/player/Player_0_1.png")]
    pub eyes_1: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 64., columns = 1, rows = 20, padding_x = 0., padding_y = 0.))]
    #[asset(path = "sprites/player/Player_0_2.png")]
    pub eyes_2: Handle<TextureAtlas>
}

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/andy_bold.ttf")]
    pub andy_bold: Handle<Font>,
 
    #[asset(path = "fonts/andy_regular.otf")]
    pub andy_regular: Handle<Font>
}

#[derive(AssetCollection)]
pub struct ItemAssets {
    #[asset(path = "sprites/Item_0.png")]
    no_item: Handle<Image>,

    #[asset(path = "sprites/Item_3509.png")]
    pub copper_pickaxe: Handle<Image>
}

#[derive(AssetCollection)]
pub struct CursorAssets {
    #[asset(path = "sprites/Cursor_0.png")]
    pub cursor: Handle<Image>,

    #[asset(path = "sprites/Cursor_11.png")]
    pub cursor_background: Handle<Image>
}

impl ItemAssets {
    pub fn no_item(&self) -> Handle<Image> {
        self.no_item.clone()
    }

    pub fn get_by_id(&self, id: i32) -> Handle<Image> {
        match id {
            0 => self.no_item.clone(),
            3509 => self.copper_pickaxe.clone(),
            _ => self.no_item()
        }
    }
}

impl BlockAssets {
    
    pub fn get_by_id(&self, id: BlockId) -> Option<Handle<TextureAtlas>> {
        match id {
            0 => Some(self.dirt.clone()),
            1 => Some(self.stone.clone()),
            2 => Some(self.grass.clone()),
            _ => None
        }
    }
}