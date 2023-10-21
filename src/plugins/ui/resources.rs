use std::{marker::PhantomData, ops::{Deref, DerefMut, Not}};

use bevy::prelude::Resource;

use crate::common::{BoolValue, Toggle};

pub(crate) trait VisibilityMarker: Clone + Copy + Send + Sync + 'static {}

#[derive(Clone, Copy)]
pub(crate) struct Cursor;
impl VisibilityMarker for Cursor {}

#[derive(Clone, Copy)]
pub(crate) struct Ui;
impl VisibilityMarker for Ui {}

#[derive(Clone, Copy)]
pub(crate) struct SettingsMenu;
impl VisibilityMarker for SettingsMenu {}

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IsVisible<T: VisibilityMarker> {
    visible: bool,
    _phantom: PhantomData<T>
}

impl<T: VisibilityMarker> IsVisible<T> {
    pub(crate) fn visible() -> Self {
        IsVisible {
            visible: true,
            _phantom: PhantomData
        }
    }

    pub(crate) fn hidden() -> Self {
        IsVisible {
            visible: false,
            _phantom: PhantomData
        }
    }
}

impl<T: VisibilityMarker> Deref for IsVisible<T> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.visible
    }
}

impl<T: VisibilityMarker> DerefMut for IsVisible<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.visible
    }
}

impl<T: VisibilityMarker> Toggle for IsVisible<T> {
    fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

impl<T: VisibilityMarker> BoolValue for IsVisible<T> {
    fn value(&self) -> bool {
        self.visible
    }
}

impl<T: VisibilityMarker> Not for IsVisible<T> {
    type Output = bool;

    fn not(self) -> Self::Output {
        !self.visible
    }
}