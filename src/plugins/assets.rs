use bevy::ecs::world::Mut;
use bevy::{
    asset::HandleUntyped,
    math::Vec2,
    prelude::{App, AssetServer, Assets, Handle, Image, Plugin, Res, ResMut, World},
    render::texture::ImageSampler,
    sprite::TextureAtlas,
    text::Font,
};
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};
use iyes_loopless::prelude::AppLooplessStateExt;

use crate::items::{Item, Pickaxe};
use crate::{block::Block, state::GameState, util::handles, wall::Wall};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::MainMenu)
                .with_collection::<BlockAssets>()
                .with_collection::<UiAssets>()
                .with_collection::<PlayerAssets>()
                .with_collection::<FontAssets>()
                .with_collection::<ItemAssets>()
                .with_collection::<CursorAssets>()
                .with_collection::<BackgroundAssets>()
                .with_collection::<WallAssets>(),
        )
        .add_exit_system(GameState::AssetLoading, setup);
    }
}

#[derive(AssetCollection)]
pub struct BlockAssets {
    #[asset(texture_atlas(
        tile_size_x = 16.,
        tile_size_y = 16.,
        rows = 15,
        columns = 16,
        padding_x = 2.,
        padding_y = 2.
    ))]
    #[asset(path = "sprites/tiles/Tiles_0.png")]
    pub dirt: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 16.,
        tile_size_y = 16.,
        rows = 15,
        columns = 16,
        padding_x = 2.,
        padding_y = 2.
    ))]
    #[asset(path = "sprites/tiles/Tiles_2.png")]
    pub grass: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 16.,
        tile_size_y = 16.,
        rows = 15,
        columns = 16,
        padding_x = 2.,
        padding_y = 2.
    ))]
    #[asset(path = "sprites/tiles/Tiles_1.png")]
    pub stone: Handle<TextureAtlas>,

    #[asset(path = "sprites/tiles/Tiles.png")]
    pub tiles: Handle<Image>
}

handles! {
    Handle<Image>,
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

        #[asset(path = "sprites/ui/Radial.png")]
        pub radial: Handle<Image>,
    }
}

#[derive(AssetCollection)]
pub struct PlayerAssets {
    #[asset(texture_atlas(
        tile_size_x = 40.,
        tile_size_y = 48.,
        columns = 1,
        rows = 14,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(path = "sprites/player/Player_0_0.png")]
    pub head: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 32.,
        tile_size_y = 64.,
        columns = 27,
        rows = 1,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(path = "sprites/player/Player_Left_Shoulder.png")]
    pub left_shoulder: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 32.,
        tile_size_y = 64.,
        columns = 27,
        rows = 1,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(path = "sprites/player/Player_Left_Hand.png")]
    pub left_hand: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 32.,
        tile_size_y = 80.,
        columns = 18,
        rows = 1,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(path = "sprites/player/Player_Right_Arm2.png")]
    pub right_arm: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 40.,
        tile_size_y = 64.,
        columns = 1,
        rows = 14,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(path = "sprites/player/Player_Hair_1_2.png")]
    pub hair: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 32.,
        tile_size_y = 64.,
        columns = 1,
        rows = 14,
        padding_x = 8.,
        padding_y = 0.
    ))]
    #[asset(path = "sprites/player/Player_Body.png")]
    pub chest: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 40.,
        tile_size_y = 64.,
        columns = 1,
        rows = 19,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(path = "sprites/player/Player_0_11.png")]
    pub feet: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 40.,
        tile_size_y = 64.,
        columns = 1,
        rows = 20,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(path = "sprites/player/Player_0_1.png")]
    pub eyes_1: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 40.,
        tile_size_y = 64.,
        columns = 1,
        rows = 20,
        padding_x = 0.,
        padding_y = 0.
    ))]
    #[asset(path = "sprites/player/Player_0_2.png")]
    pub eyes_2: Handle<TextureAtlas>,
}

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/andy_bold.ttf")]
    pub andy_bold: Handle<Font>,

    #[asset(path = "fonts/andy_regular.otf")]
    pub andy_regular: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct ItemAssets {
    #[asset(path = "sprites/items/Item_0.png")]
    no_item: Handle<Image>,

    #[asset(path = "sprites/items/Item_2.png")]
    pub dirt_block: Handle<Image>,

    #[asset(path = "sprites/items/Item_3.png")]
    pub stone_block: Handle<Image>,

    #[asset(path = "sprites/items/Item_3509.png")]
    pub copper_pickaxe: Handle<Image>,
}

handles! {
    Handle<Image>,
    #[derive(AssetCollection)]
    pub struct CursorAssets {
        #[asset(path = "sprites/ui/Cursor_0.png")]
        pub cursor: Handle<Image>,

        #[asset(path = "sprites/ui/Cursor_11.png")]
        pub cursor_background: Handle<Image>,
    }
}

handles! {
    Handle<TextureAtlas>,
    #[derive(AssetCollection)]
    pub struct BackgroundAssets {
        #[asset(texture_atlas(tile_size_x = 48., tile_size_y = 1400., columns = 1, rows = 1))]
        #[asset(path = "sprites/backgrounds/Background_0.png")]
        pub background_0: Handle<TextureAtlas>,

        #[asset(texture_atlas(tile_size_x = 1024., tile_size_y = 600., columns = 1, rows = 1))]
        #[asset(path = "sprites/backgrounds/Background_7.png")]
        pub background_7: Handle<TextureAtlas>,

        #[asset(texture_atlas(tile_size_x = 1024., tile_size_y = 600., columns = 1, rows = 1))]
        #[asset(path = "sprites/backgrounds/Background_90.png")]
        pub background_90: Handle<TextureAtlas>,

        #[asset(texture_atlas(tile_size_x = 1024., tile_size_y = 600., columns = 1, rows = 1))]
        #[asset(path = "sprites/backgrounds/Background_91.png")]
        pub background_91: Handle<TextureAtlas>,

        #[asset(texture_atlas(tile_size_x = 1024., tile_size_y = 600., columns = 1, rows = 1))]
        #[asset(path = "sprites/backgrounds/Background_92.png")]
        pub background_92: Handle<TextureAtlas>,

        #[asset(texture_atlas(tile_size_x = 2048., tile_size_y = 434., columns = 1, rows = 1))]
        #[asset(path = "sprites/backgrounds/Background_112.png")]
        pub background_112: Handle<TextureAtlas>,

    }
}

#[derive(AssetCollection)]
pub struct WallAssets {
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32., columns = 13, rows = 5))]
    #[asset(path = "sprites/walls/Wall_2.png")]
    pub wall_2: Handle<TextureAtlas>,
}

impl WallAssets {
    pub fn get_by_wall(&self, id: Wall) -> Option<Handle<TextureAtlas>> {
        match id {
            Wall::DirtWall => Some(self.wall_2.clone()),
            _ => None,
        }
    }
}

impl ItemAssets {
    pub fn no_item(&self) -> Handle<Image> {
        self.no_item.clone()
    }

    pub fn get_by_item(&self, item: Item) -> Handle<Image> {
        match item {
            Item::Block(Block::Dirt) => self.dirt_block.clone(),
            Item::Block(Block::Stone) => self.stone_block.clone(),
            Item::Pickaxe(Pickaxe::CopperPickaxe) => self.copper_pickaxe.clone(),
            _ => self.no_item(),
        }
    }
}

fn setup(
    mut images: ResMut<Assets<Image>>,
    texture_atlasses: Res<Assets<TextureAtlas>>,
    background_assets: Res<BackgroundAssets>,
    ui_assets: Res<UiAssets>,
    cursor_assets: Res<CursorAssets>,
) {
    for handle in background_assets.handles() {
        let atlas = texture_atlasses.get(handle).unwrap();
        let mut image = images.get_mut(&atlas.texture).unwrap();

        image.sampler_descriptor = ImageSampler::linear();
    }

    let mut handles = ui_assets.handles();
    handles.append(&mut cursor_assets.handles());

    for handle in handles.iter() {
        let mut image = images.get_mut(&handle).unwrap();

        image.sampler_descriptor = ImageSampler::linear();
    }
}

impl BlockAssets {
    pub fn get_by_block(&self, block: Block) -> Option<Handle<TextureAtlas>> {
        match block {
            Block::Dirt => Some(self.dirt.clone()),
            Block::Stone => Some(self.stone.clone()),
            Block::Grass => Some(self.grass.clone()),
        }
    }
}