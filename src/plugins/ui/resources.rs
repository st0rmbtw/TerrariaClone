use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy, Default)]
pub(crate) struct ExtraUiVisibility(pub bool);

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UiVisibility(bool);

impl std::ops::Not for UiVisibility {
    type Output = UiVisibility;

    fn not(self) -> Self::Output {
        UiVisibility(!self.is_visible())
    }
}

impl UiVisibility {
    pub(crate) const VISIBLE: Self = Self(true);

    pub(crate) fn is_visible(&self) -> bool {
        self.0
    }
}

impl Default for UiVisibility {
    fn default() -> Self { Self::VISIBLE }
}