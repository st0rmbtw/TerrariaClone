use bevy::prelude::{Component, Image, Handle};

#[derive(Component)]
pub(super) struct InventoryUi;

#[derive(Component)]
pub(super) struct HotbarUi;

#[derive(Component)]
pub(super) struct HotbarCellMarker;

#[derive(Component)]
pub(super) struct SelectedItemNameMarker;

#[derive(Component)]
pub(super) struct InventoryCellIndex(pub usize);

#[derive(Component, Default)]
pub(super) struct InventoryCellItemImage(pub Handle<Image>);

#[derive(Component, Default)]
pub(super) struct InventoryItemAmount(pub u16);