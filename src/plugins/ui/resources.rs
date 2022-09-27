pub struct ToggleExtraUiEvent(pub bool);

#[derive(Clone, Copy, Default)]
pub struct ExtraUiVisibility(pub bool);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UiVisibility(pub bool);

impl Default for UiVisibility {
    fn default() -> Self {
        Self(true)
    }
}