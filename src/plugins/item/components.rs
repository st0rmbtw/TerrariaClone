use bevy::{prelude::{Component, DerefMut, Deref}, time::Timer};

use crate::items::ItemStack;

#[derive(Component)]
pub(crate) struct DroppedItem {
    pub(crate) item_stack: ItemStack
}

#[derive(Component)]
pub(super) struct Following;

#[derive(Component, Deref, DerefMut)]
pub(super) struct GrabTimer(pub(super) Timer);