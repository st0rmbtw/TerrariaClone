use bevy::prelude::{Resource, AudioSource, OnExit};
use bevy::{
    math::Vec2,
    prelude::{App, AssetServer, Assets, Handle, Image, Plugin, Res, ResMut},
    render::texture::ImageSampler,
    sprite::TextureAtlas,
    text::Font,
};
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};
use rand::{RngCore, thread_rng};
use rand::seq::SliceRandom;

use crate::items::{Item, Pickaxe, ItemTool, Axe, ItemSeed, ItemBlock};
use crate::common::state::GameState;
use crate::world::block::BlockType;

use super::audio::{SoundType, MusicType};

macro_rules! handles {
    (
     $field_type:ty,
     // meta data about struct
     $(#[$meta:meta])*
     $vis:vis struct $struct_name:ident {
        $(
        // meta data about field
        $(#[$field_meta:meta])*
        $field_vis:vis $field_name:ident : $field_t:ty
        ),*$(,)+
    }
    ) => {
        $(#[$meta])*
        pub struct $struct_name {
            $(
            $(#[$field_meta])*
            pub $field_name : $field_type,
            )*
        }

        impl $struct_name {
            fn handles(&self) -> Vec<$field_type> {
                vec![$(self.$field_name.clone()),*]
            }
        }
    }
}

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Menu)
        );

        app.add_collection_to_loading_state::<_, BlockAssets>(GameState::AssetLoading);
        app.add_collection_to_loading_state::<_, WallAssets>(GameState::AssetLoading);
        app.add_collection_to_loading_state::<_, UiAssets>(GameState::AssetLoading);
        app.add_collection_to_loading_state::<_, SoundAssets>(GameState::AssetLoading);
        app.add_collection_to_loading_state::<_, MusicAssets>(GameState::AssetLoading);
        app.add_collection_to_loading_state::<_, PlayerAssets>(GameState::AssetLoading);
        app.add_collection_to_loading_state::<_, FontAssets>(GameState::AssetLoading);
        app.add_collection_to_loading_state::<_, ItemAssets>(GameState::AssetLoading);
        app.add_collection_to_loading_state::<_, CursorAssets>(GameState::AssetLoading);
        app.add_collection_to_loading_state::<_, BackgroundAssets>(GameState::AssetLoading);
        app.add_collection_to_loading_state::<_, CelestialBodyAssets>(GameState::AssetLoading);
        app.add_collection_to_loading_state::<_, ParticleAssets>(GameState::AssetLoading);
        
        app.add_systems(OnExit(GameState::AssetLoading), setup);
    }
}

fn setup(
    mut images: ResMut<Assets<Image>>,
    ui_assets: Res<UiAssets>,
    cursor_assets: Res<CursorAssets>,
    background_assets: Res<BackgroundAssets>,
) {
    let mut handles = ui_assets.handles();
    handles.append(&mut cursor_assets.handles());
    handles.append(&mut background_assets.handles());

    for handle in handles.iter() {
        let image = images.get_mut(handle).unwrap();

        image.sampler_descriptor = ImageSampler::linear();
    }
}

#[derive(Resource, AssetCollection)]
pub(crate) struct BlockAssets {
    #[asset(path = "sprites/tiles/Tiles.png")]
    pub(crate) tiles: Handle<Image>,

    #[asset(path = "sprites/tiles/TileCracks.png")]
    pub(crate) tile_cracks: Handle<Image>,

    #[asset(path = "sprites/tiles/Tiles_5.png")]
    pub(crate) trees: Handle<Image>,

    #[asset(path = "sprites/tiles/Tree_Branches_0.png")]
    pub(crate) tree_branches_forest: Handle<Image>,

    #[asset(path = "sprites/tiles/Tree_Tops_0.png")]
    pub(crate) tree_tops_forest: Handle<Image>,
}

handles! {
    Handle<Image>,
    #[derive(Resource, AssetCollection)]
    pub(crate) struct UiAssets {
        #[asset(path = "sprites/ui/InnerPanelBackground.png")]
        pub(crate) iner_panel_background: Handle<Image>,

        #[asset(path = "sprites/ui/PlayerBackground.png")]
        pub(crate) player_background: Handle<Image>,

        #[asset(path = "sprites/Inventory_Back.png")]
        pub(crate) inventory_background: Handle<Image>,

        #[asset(path = "sprites/Inventory_Back14.png")]
        pub(crate) selected_inventory_background: Handle<Image>,

        #[asset(path = "sprites/ui/Radial.png")]
        pub(crate) radial: Handle<Image>,

        #[asset(path = "sprites/ui/Logo.png")]
        pub(crate) logo: Handle<Image>,

        #[asset(path = "sprites/ui/SliderBorder.png")]
        pub(crate) slider_border: Handle<Image>,

        #[asset(path = "sprites/ui/SliderBackground.png")]
        pub(crate) slider_background: Handle<Image>,

        #[asset(path = "sprites/ui/SliderHandle.png")]
        pub(crate) slider_handle: Handle<Image>,
    }
}

#[derive(Resource, AssetCollection)]
pub(crate) struct PlayerAssets {
    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 48., columns = 1, rows = 14))]
    #[asset(path = "sprites/player/Player_0_0.png")]
    pub(crate) head: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 64., columns = 27, rows = 1))]
    #[asset(path = "sprites/player/Player_Left_Shoulder.png")]
    pub(crate) left_shoulder: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 64., columns = 27, rows = 1))]
    #[asset(path = "sprites/player/Player_Left_Hand.png")]
    pub(crate) left_hand: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 80., columns = 18, rows = 1))]
    #[asset(path = "sprites/player/Player_Right_Arm.png")]
    pub(crate) right_arm: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 64., columns = 1, rows = 14))]
    #[asset(path = "sprites/player/Player_Hair_1.png")]
    pub(crate) hair: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 64., columns = 1, rows = 14, padding_x = 8.))]
    #[asset(path = "sprites/player/Player_Body.png")]
    pub(crate) chest: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 64., columns = 1, rows = 19))]
    #[asset(path = "sprites/player/Player_0_11.png")]
    pub(crate) feet: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 64., columns = 1, rows = 20))]
    #[asset(path = "sprites/player/Player_0_1.png")]
    pub(crate) eyes_1: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 40., tile_size_y = 64., columns = 1, rows = 20))]
    #[asset(path = "sprites/player/Player_0_2.png")]
    pub(crate) eyes_2: Handle<TextureAtlas>,
}

#[derive(Resource, AssetCollection)]
pub(crate) struct FontAssets {
    #[asset(path = "fonts/andy_bold.ttf")]
    pub(crate) andy_bold: Handle<Font>,

    #[asset(path = "fonts/andy_regular.otf")]
    pub(crate) andy_regular: Handle<Font>,
}

#[derive(Resource, AssetCollection)]
pub(crate) struct ItemAssets {
    #[asset(path = "sprites/items/Item_2.png")]
    pub(crate) dirt_block: Handle<Image>,

    #[asset(path = "sprites/items/Item_3.png")]
    pub(crate) stone_block: Handle<Image>,

    #[asset(path = "sprites/items/Item_62.png")]
    pub(crate) grass_seed: Handle<Image>,

    #[asset(path = "sprites/items/Item_3509.png")]
    pub(crate) copper_pickaxe: Handle<Image>,

    #[asset(path = "sprites/items/Item_3506.png")]
    pub(crate) copper_axe: Handle<Image>,

    #[asset(path = "sprites/items/Item_9.png")]
    pub(crate) wood: Handle<Image>,
}

impl ItemAssets {
    pub(crate) fn get_by_item(&self, item: Item) -> Handle<Image> {
        match item {
            Item::Block(block) => {
                match block {
                    ItemBlock::Dirt => self.dirt_block.clone_weak(),
                    ItemBlock::Stone => self.stone_block.clone_weak(),
                    ItemBlock::Wood => self.wood.clone_weak(),
                }
            }
            Item::Tool(ItemTool::Pickaxe(Pickaxe::CopperPickaxe)) => self.copper_pickaxe.clone_weak(),
            Item::Tool(ItemTool::Axe(Axe::CopperAxe)) => self.copper_axe.clone_weak(),
            Item::Seed(ItemSeed::Grass) => self.grass_seed.clone_weak()
        }
    }
}

handles! {
    Handle<Image>,
    #[derive(Resource, AssetCollection)]
    pub(crate) struct CursorAssets {
        #[asset(path = "sprites/ui/Cursor_0.png")]
        pub(crate) cursor: Handle<Image>,

        #[asset(path = "sprites/ui/Cursor_11.png")]
        pub(crate) cursor_background: Handle<Image>,
    }
}

handles! {
    Handle<Image>,
    #[derive(Resource, AssetCollection)]
    pub(crate) struct BackgroundAssets {
        #[asset(path = "sprites/backgrounds/Background_0.png")]
        pub(crate) background_0: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Background_7.png")]
        pub(crate) background_7: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Background_74.png")]
        pub(crate) background_74: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Background_77.png")]
        pub(crate) background_77: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Background_78.png")]
        pub(crate) background_78: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Background_55.png")]
        pub(crate) background_55: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Background_90.png")]
        pub(crate) background_90: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Background_91.png")]
        pub(crate) background_91: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Background_92.png")]
        pub(crate) background_92: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Background_93.png")]
        pub(crate) background_93: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Background_112.png")]
        pub(crate) background_112: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Background_114.png")]
        pub(crate) background_114: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Star_0.png")]
        pub(crate) star_0: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Star_1.png")]
        pub(crate) star_1: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Star_2.png")]
        pub(crate) star_2: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Star_3.png")]
        pub(crate) star_3: Handle<Image>,

        #[asset(path = "sprites/backgrounds/Star_4.png")]
        pub(crate) star_4: Handle<Image>,
    }
}

#[derive(Resource, AssetCollection)]
pub(crate) struct CelestialBodyAssets {
    #[asset(texture_atlas(tile_size_x = 114., tile_size_y = 114., columns = 1, rows = 1))]
    #[asset(path = "sprites/backgrounds/Sun.png")]
    pub(crate) sun: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 50., tile_size_y = 50., columns = 1, rows = 8))]
    #[asset(path = "sprites/backgrounds/Moon_0.png")]
    pub(crate) moon_0: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 50., tile_size_y = 50., columns = 1, rows = 8))]
    #[asset(path = "sprites/backgrounds/Moon_1.png")]
    pub(crate) moon_1: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 50., tile_size_y = 50., columns = 1, rows = 8))]
    #[asset(path = "sprites/backgrounds/Moon_2.png")]
    pub(crate) moon_2: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 50., tile_size_y = 50., columns = 1, rows = 8))]
    #[asset(path = "sprites/backgrounds/Moon_3.png")]
    pub(crate) moon_3: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 50., tile_size_y = 50., columns = 1, rows = 8))]
    #[asset(path = "sprites/backgrounds/Moon_4.png")]
    pub(crate) moon_4: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 50., tile_size_y = 50., columns = 1, rows = 8))]
    #[asset(path = "sprites/backgrounds/Moon_5.png")]
    pub(crate) moon_5: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 50., tile_size_y = 50., columns = 1, rows = 8))]
    #[asset(path = "sprites/backgrounds/Moon_6.png")]
    pub(crate) moon_6: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 50., tile_size_y = 50., columns = 1, rows = 8))]
    #[asset(path = "sprites/backgrounds/Moon_7.png")]
    pub(crate) moon_7: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 50., tile_size_y = 50., columns = 1, rows = 8))]
    #[asset(path = "sprites/backgrounds/Moon_8.png")]
    pub(crate) moon_8: Handle<TextureAtlas>,
}

impl CelestialBodyAssets {
    pub(crate) const fn moons(&self) -> [&Handle<TextureAtlas>; 9] {
        [&self.moon_0, &self.moon_1, &self.moon_2, &self.moon_3, &self.moon_4, &self.moon_5, &self.moon_6, &self.moon_7, &self.moon_8]
    }
}

#[derive(Resource, AssetCollection)]
pub(crate) struct WallAssets {
    #[asset(path = "sprites/walls/Walls.png")]
    pub(crate) walls: Handle<Image>,
}

#[derive(Resource, AssetCollection)]
pub(crate) struct SoundAssets {
    #[asset(path = "sounds/Menu_Tick.wav")]
    pub(crate) menu_tick: Handle<AudioSource>,

    #[asset(path = "sounds/Menu_Open.wav")]
    pub(crate) menu_open: Handle<AudioSource>,

    #[asset(path = "sounds/Menu_Close.wav")]
    pub(crate) menu_close: Handle<AudioSource>,

    #[asset(paths("sounds/Swing_1.wav", "sounds/Swing_2.wav", "sounds/Swing_3.wav"), collection(typed))]
    pub(crate) swing: Vec<Handle<AudioSource>>,

    #[asset(paths("sounds/Dig_0.wav", "sounds/Dig_1.wav", "sounds/Dig_2.wav"), collection(typed))]
    pub(crate) dig: Vec<Handle<AudioSource>>,

    #[asset(paths("sounds/Tink_0.wav", "sounds/Tink_1.wav", "sounds/Tink_2.wav"), collection(typed))]
    pub(crate) tink: Vec<Handle<AudioSource>>,

    #[asset(path = "sounds/Grab.wav")]
    pub(crate) grab: Handle<AudioSource>
}

impl SoundAssets {
    pub(crate) fn get_handle_by_sound_type(&self, sound_type: SoundType) -> Handle<AudioSource> {
        match sound_type {
            SoundType::MenuTick => self.menu_tick.clone_weak(),
            SoundType::MenuOpen => self.menu_open.clone_weak(),
            SoundType::MenuClose => self.menu_close.clone_weak(),
            SoundType::BlockHit(block_type) => self.get_by_block(block_type, &mut thread_rng()),
            SoundType::BlockPlace(block_type) => self.get_by_block(block_type, &mut thread_rng()),
            SoundType::PlayerToolSwing(_tool) => self.swing.choose(&mut thread_rng()).unwrap().clone_weak(),
            SoundType::ItemGrab => self.grab.clone_weak()
        }
    }
    
    fn get_by_block<Rng: RngCore>(&self, block: BlockType, rng: &mut Rng) -> Handle<AudioSource> {
        match block {
            BlockType::Stone => self.tink.choose(rng).unwrap().clone_weak(),
            _ => self.dig.choose(rng).unwrap().clone_weak()
        }
    }
}

#[derive(Resource, AssetCollection)]
pub(crate) struct MusicAssets {
    #[asset(path = "music/TitleScreen.mp3")]
    pub(crate) title_screen: Handle<AudioSource>,

    #[asset(path = "music/OverworldDay.mp3")]
    pub(crate) overworld_day: Handle<AudioSource>,
}

impl MusicAssets {
    pub(crate) fn get_handle_by_music_type(&self, music_type: MusicType) -> Handle<AudioSource> {
        match music_type {
            MusicType::TitleScreen => self.title_screen.clone_weak(),
            MusicType::OverworldDay => self.overworld_day.clone_weak(),
        }
    }
}

#[derive(Resource, AssetCollection)]
pub(crate) struct ParticleAssets {
    #[asset(texture_atlas(tile_size_x = 8., tile_size_y = 8., columns = 100, rows = 12, padding_x = 2., padding_y = 2.))]
    #[asset(path = "sprites/Particles.png")]
    pub(crate) particles: Handle<TextureAtlas>,
}

impl ParticleAssets {
    pub(crate) const COLUMNS: usize = 100;
}