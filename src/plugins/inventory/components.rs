use bevy::{prelude::{Component, Handle, Image, ReflectComponent}, reflect::Reflect};

#[derive(Component, Default)]
pub(super) struct InventoryUi;

#[derive(Component, Default)]
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
pub struct InventoryItemAmount(pub u16);

#[derive(Reflect, Component, Clone, Copy, Default)]
#[reflect(Component)]
pub struct UseItemAnimationData(pub usize);

#[derive(Component)]
pub struct UsedItem;