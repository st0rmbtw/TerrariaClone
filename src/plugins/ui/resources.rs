use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy, Default)]
pub struct ExtraUiVisibility(pub bool);

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub struct UiVisibility(pub bool);

impl Default for UiVisibility {
    fn default() -> Self {
        Self(true)
    }
}