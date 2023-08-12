use bevy::prelude::{Plugin, App, PostUpdate, Event, IntoSystemConfigs, in_state, not};

use crate::{world::block::BlockType, items::Tool, common::state::GameState};

mod systems;

pub(crate) struct AudioPlugin;
impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySoundEvent>();
        app.add_systems(
            PostUpdate,
            systems::handle_play_sound_event.run_if(not(in_state(GameState::AssetLoading)))
        );
    }
}


#[derive(Clone, Copy)]
pub(crate) enum SoundType {
    MenuTick,
    MenuOpen,
    MenuClose,

    BlockHit(BlockType),
    BlockPlace(BlockType),

    PlayerToolSwing(Tool)
}

#[derive(Event)]
pub(crate) struct PlaySoundEvent(pub(crate) SoundType);