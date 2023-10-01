use bevy::prelude::{Component, Image, Handle};

#[derive(Component)]
pub(super) struct InventoryUi;

#[derive(Component)]
pub(super) struct InventoryUiContainer;

#[derive(Component)]
pub(super) struct HotbarUi;

#[derive(Component)]
pub(super) struct HotbarCell;

#[derive(Component)]
pub(super) struct InventoryCell;

#[derive(Component)]
pub(super) struct SelectedItemName;

#[derive(Component)]
pub(super) struct HotbarCellIndex;

#[derive(Component)]
pub(super) struct CellIndex(pub usize);

#[derive(Component, Default)]
pub(super) struct CellItemImage(pub Handle<Image>);

#[derive(Component, Default)]
pub(super) struct ItemAmount(pub u16);