use bevy::prelude::Resource;

use crate::common::{BoolValue, Toggle};

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct InventoryUiVisibility(bool);

impl InventoryUiVisibility {
    pub(crate) const HIDDEN: Self = Self(false);
}

impl Toggle for InventoryUiVisibility {
    fn toggle(&mut self) {
        self.0 = !self.0;
    }
}

impl BoolValue for InventoryUiVisibility {
    fn value(&self) -> bool {
        self.0
    }
}

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UiVisibility(bool);

impl UiVisibility {
    pub(crate) const VISIBLE: Self = Self(true);
}

impl Toggle for UiVisibility {
    fn toggle(&mut self) {
        self.0 = !self.0
    }
}

impl BoolValue for UiVisibility {
    fn value(&self) -> bool {
        self.0
    }
}

impl Default for UiVisibility {
    fn default() -> Self { Self::VISIBLE }
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct SettingsMenuVisibility(pub(crate) bool);

impl Toggle for SettingsMenuVisibility {
    fn toggle(&mut self) {
        self.0 = !self.0;
    }
}

impl BoolValue for SettingsMenuVisibility {
    fn value(&self) -> bool {
        self.0
    }
}