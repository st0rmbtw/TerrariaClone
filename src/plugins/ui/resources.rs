use bevy::prelude::Resource;

use crate::common::IsVisible;

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ExtraUiVisibility(bool);

impl ExtraUiVisibility {
    pub(crate) const HIDDEN: Self = Self(false);

    pub(crate) fn toggle(&mut self) {
        self.0 = !self.0;
    }
}

impl IsVisible for ExtraUiVisibility {
    fn is_visible(&self) -> bool {
        self.0
    }
}

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UiVisibility(bool);

impl UiVisibility {
    pub(crate) const VISIBLE: Self = Self(true);

    pub(crate) fn toggle(&mut self) {
        self.0 = !self.0;
    }
}

impl IsVisible for UiVisibility {
    fn is_visible(&self) -> bool {
        self.0
    }
}

impl Default for UiVisibility {
    fn default() -> Self { Self::VISIBLE }
}