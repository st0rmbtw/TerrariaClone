use bevy::prelude::{Component, Handle, Image, Deref, DerefMut};
use bevy_inspector_egui::Inspectable;

use crate::items::ItemStack;

#[derive(Component, Default, Inspectable)]
pub struct InventoryUi;

#[derive(Component, Default)]
pub struct HotbarUi;
#[derive(Component)]
pub struct HotbarCellMarker;

#[derive(Component)]
pub struct SelectedItemNameMarker;

#[derive(Component)]
pub struct InventoryCellIndex(pub usize);

#[derive(Component, Default)]
pub struct InventoryCellItemImage(pub Handle<Image>);

#[derive(Component, Default)]
pub struct InventoryItemAmount(pub u16);

#[derive(Component, Default, Deref, DerefMut)]
pub struct SelectedItem(pub Option<ItemStack>);