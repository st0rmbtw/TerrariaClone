use bevy::{prelude::{Plugin, AssetServer, Assets, Handle, App, Image, World}, sprite::TextureAtlas, math::Vec2, text::Font, asset::HandleUntyped};
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use bevy::ecs::world::Mut;

pub const TILE_SIZE: f32 = 16.;

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
    pub main: Handle<TextureAtlas>
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