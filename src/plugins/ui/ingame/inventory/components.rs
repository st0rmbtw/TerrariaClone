use bevy::prelude::{Component, Image, Handle};

#[derive(Component)]
pub(super) struct InventoryUi;

#[derive(Component)]
pub(super) struct InventoryUiContainer;

#[derive(Component)]
pub(super) struct HotbarUi;

#[derive(Component)]
pub(super) struct HotbarSlot;

#[derive(Component)]
pub(super) struct InventorySlot;

#[derive(Component)]
pub(super) struct SelectedItemName;

#[derive(Component)]
pub(super) struct HotbarSlotIndex;

#[derive(Component)]
pub(super) struct SlotIndex(pub usize);

#[derive(Component, Default, PartialEq, Eq)]
pub(super) struct SlotItemImage(pub Handle<Image>);

#[derive(Component, Default)]
pub(super) struct ItemAmount(pub u16);