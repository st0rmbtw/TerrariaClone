use bevy::{prelude::{Plugin, AssetServer, Assets, Handle, App, Image, World}, sprite::TextureAtlas, math::Vec2, text::Font, asset::HandleUntyped};
use bevy_asset_loader::{AssetCollection, AssetCollectionApp};

pub const TILE_SIZE: f32 = 16.;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_collection::<BlockAssets>()
            .init_collection::<UiAssets>()
            .init_collection::<PlayerAssets>()
            .init_collection::<FontAssets>();
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
    pub iner_panel_background: Handle<Image>
}

#[derive(AssetCollection)]
pub struct PlayerAssets {
    #[asset(texture_atlas(tile_size_x = 37., tile_size_y = 53., columns = 1, rows = 16, padding_x = 0., padding_y = 3.))]
    #[asset(path = "sprites/npc_22.png")]
    pub main: Handle<TextureAtlas>
}

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/andyb.ttf")]
    pub andy_bold: Handle<Font>
}