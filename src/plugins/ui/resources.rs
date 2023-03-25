use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy, Default)]
pub(crate) struct ExtraUiVisibility(pub bool);

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UiVisibility(pub bool);

impl Default for UiVisibility {
    fn default() -> Self {
        Self(true)
    }
}